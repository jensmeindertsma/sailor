mod app;

use axum::{
    body::{Body, Bytes, HttpBody},
    routing::future::RouteFuture,
    BoxError, Router,
};
use hyper::{Request, Response};
use sailor_config::Configurable;
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

impl<C> WebInterface<C> {
    pub fn new(configuration: Arc<C>) -> Self {
        Self {
            router: app::create_router(),
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
