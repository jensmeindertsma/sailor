mod body;
mod fetcher;

use axum::routing::future::RouteFuture;
use body::Body;
use fetcher::FetchError;
use http_body_util::{Empty, Full};
use hyper::{
    body::{Bytes, Incoming},
    Request, Response,
};
use pin_project::pin_project;
use sailor_config::Configurable;
use sailor_web::WebInterface;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;
use tracing::{error, info};

const WEB_HOSTNAME: &str = "cabin.jensmeindertsma.com";

pub struct Proxy<C> {
    configuration: Arc<C>,
    web: WebInterface<C>,
}

impl<C> Clone for Proxy<C> {
    fn clone(&self) -> Self {
        Self {
            configuration: self.configuration.clone(),
            web: self.web.clone(),
        }
    }
}

impl<C> Proxy<C>
where
    C: Configurable,
{
    pub fn new(configuration: Arc<C>) -> Self {
        Self {
            web: WebInterface::new(configuration.clone()),
            configuration,
        }
    }
}

impl<C> Service<Request<Incoming>> for Proxy<C>
where
    C: Configurable,
{
    type Future = ProxyFuture<C>;
    type Response = Response<Body>;
    type Error = Infallible;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <WebInterface<C> as Service<Request<Incoming>>>::poll_ready(&mut self.web, context)
    }

    fn call(&mut self, request: Request<Incoming>) -> Self::Future {
        let host_header = request
            .headers()
            .get("Host")
            .and_then(|host| host.to_str().ok().map(|s| s.to_string()));

        info!("Host: {host_header:?}");

        match host_header {
            Some(host) if host.as_str() == WEB_HOSTNAME => {
                info!("request is to web interface");
                ProxyFuture::Web(self.web.call(request))
            }
            Some(host) => {
                if let Some(address) =
                    self.configuration
                        .get()
                        .applications
                        .iter()
                        .find_map(|app| {
                            if app.hostname == host {
                                Some(app.address)
                            } else {
                                None
                            }
                        })
                {
                    info!("request is to proxied application");

                    ProxyFuture::Forwarded {
                        future: Box::pin(fetcher::fetch(address, request)),
                        web: self.web.clone(),
                    }
                } else {
                    info!("request is to unknown proxy address");

                    ProxyFuture::Web(
                        self.web.call(
                            Request::builder()
                                .uri("/proxy-error")
                                .header("Host", host)
                                .body(Empty::new())
                                .expect("constructing error page request should succeed"),
                        ),
                    )
                }
            }
            None => {
                info!("request has no Host header");

                ProxyFuture::Web(
                    self.web.call(
                        Request::builder()
                            .uri("/proxy-error")
                            .body(Empty::new())
                            .expect("constructing error page request should succeed"),
                    ),
                )
            }
        }
    }
}

#[pin_project(project = Enum)]
pub enum ProxyFuture<C> {
    Forwarded {
        #[pin]
        future:
            Pin<Box<dyn Future<Output = Result<Response<Incoming>, FetchError>> + Send + 'static>>,
        web: WebInterface<C>,
    },
    Web(#[pin] RouteFuture<Infallible>),
}

impl<C> Future for ProxyFuture<C>
where
    C: Configurable,
{
    type Output = Result<Response<Body>, Infallible>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();

        enum Outcome<C> {
            Poll(Poll<Result<Response<Body>, Infallible>>),
            Mutate(ProxyFuture<C>),
        }

        info!("polling proxy future");

        let outcome: Outcome<C> = match this {
            Enum::Forwarded { mut future, web } => match future.as_mut().poll(context) {
                Poll::Ready(result) => match result {
                    Ok(response) => Outcome::Poll(Poll::Ready(Ok(response.map(Body::Hyper)))),
                    Err(fetch_error) => {
                        error!("fetcher returned error: {:?}", fetch_error);
                        let error = serde_json::to_string(&ProxyError::FetchError(fetch_error))
                            .expect("serialization of proxy error should succeed")
                            .as_bytes()
                            .to_vec();

                        let web_future = web.call(
                            Request::builder()
                                .uri("/proxy-error")
                                .body(Full::new(Bytes::from(error)))
                                .expect("constructing error page request should succeed"),
                        );

                        Outcome::Mutate(ProxyFuture::Web(web_future))
                    }
                },
                Poll::Pending => Outcome::Poll(Poll::Pending),
            },
            Enum::Web(f) => Outcome::Poll(
                f.poll(context)
                    .map(|result| result.map(|response| response.map(Body::Axum))),
            ),
        };

        match outcome {
            Outcome::Poll(poll) => {
                match poll {
                    Poll::Pending => {
                        info!("proxy futured polled pending");
                    }
                    Poll::Ready(_) => {
                        info!("proxy futured polled ready");
                    }
                }
                poll
            }
            Outcome::Mutate(value) => {
                *self = value;
                info!("mutated proxy future");
                Poll::Pending
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum ProxyError {
    FetchError(fetcher::FetchError),
}
