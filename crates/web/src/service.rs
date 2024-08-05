use axum::{
    body::{Body, Bytes, HttpBody},
    response::{Html, IntoResponse},
    routing::future::RouteFuture,
    BoxError, Json, Router,
};
use http::Uri;
use hyper::{Request, Response};
use sail_config::Configurable;
use std::{
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;
use tracing::info;

#[derive(Debug, Default)]
pub struct WebInterface<C> {
    router: Router,
    configuration: Arc<C>,
}

impl<C> Clone for WebInterface<C> {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            configuration: self.configuration.clone(),
        }
    }
}

async fn handle_request(uri: Uri, Json(json): Json<String>) -> impl IntoResponse {
    Html(format!("<h1>Hey `{uri}`<h1><p>{json}</p>"))
}

impl<C> WebInterface<C> {
    pub fn new(configuration: Arc<C>) -> Self {
        Self {
            router: Router::new().fallback(handle_request),

            configuration,
        }
    }
}

impl<B, C> Service<Request<B>> for WebInterface<C>
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
    C: Configurable,
{
    type Error = Infallible;
    type Response = Response<Body>;
    type Future = RouteFuture<Infallible>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <Router as Service<Request<B>>>::poll_ready(&mut self.router, cx)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        info!("[web] routing request to {}", request.uri());
        self.router.call(request)
    }
}
