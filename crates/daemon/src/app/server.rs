use crate::configuration::Configuration;
use core::fmt::{self, Display};
use hyper::{
    body::{Body, Bytes, Frame, Incoming},
    Request as HyperRequest, Response as HyperResponse,
};
use std::{
    convert::Infallible,
    error::Error,
    fmt::Formatter,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;

#[derive(Clone)]
pub struct ServerHandler {
    configuration: Arc<Configuration>,
}

impl ServerHandler {
    pub fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }
}

impl Service<HyperRequest<Incoming>> for ServerHandler {
    type Response = HyperResponse<HandlerBody>;
    type Error = Infallible;
    type Future = ServerHandlerFuture;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {}

    fn call(&mut self, request: HyperRequest<Incoming>) -> Self::Future {}
}

struct ServerHandlerFuture {}

impl Future for ServerHandlerFuture {
    type Output = Result<HyperResponse<HandlerBody>, Infallible>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {}
}

enum HandlerBody {
    Hyper(Incoming),
}

impl Body for HandlerBody {
    type Data = Bytes;
    type Error = BodyError;

    fn poll_frame(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            HandlerBody::Hyper(incoming) => Pin::new(incoming)
                .poll_frame(context)
                .map(|o| o.map(|r| r.map_err(BodyError::Hyper))),
        }
    }
}

#[derive(Debug)]
pub enum BodyError {
    Hyper(hyper::Error),
}

impl Display for BodyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BodyError::Hyper(e) => format!("hyper body error: {e:?}"),
            }
        )
    }
}

impl Error for BodyError {}

/*
Ok(match request.headers().get("Host") {
            None => HyperResponse::builder()
                .header("Content-Type", "text/html")
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(format!(
                    "<h1>No Host header!</h1>\n"
                ))))
                .unwrap(),
            Some(host) => match host.to_str().unwrap() {
                "registry.jensmeindertsma.com" => HyperResponse::builder()
                    .header("Content-Type", "text/html")
                    .status(StatusCode::OK)
                    .body(Full::new(Bytes::from(format!(
                        "<h1>Registry {}</h1>\n",
                        request.uri()
                    ))))
                    .unwrap(),
                host => {
                    if let Some(application) = self.configuration.get().applications.get(host) {
                        HyperResponse::builder()
                            .header("Content-Type", "text/html")
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::from(format!(
                                "<h1>Application {} {}</h1>\n",
                                application.name,
                                request.uri()
                            ))))
                            .unwrap()
                    } else {
                        HyperResponse::builder()
                            .header("Content-Type", "text/html")
                            .status(StatusCode::BAD_REQUEST)
                            .body(Full::new(Bytes::from(format!(
                                "<h1>Unknown host {}!</h1>\n",
                                host
                            ))))
                            .unwrap()
                    }
                }
            },
        })
*/
