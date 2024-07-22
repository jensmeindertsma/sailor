mod files;

use crate::App;
use axum::{
    body::{Body, Bytes, HttpBody},
    routing::future::RouteFuture,
    BoxError, Router,
};
use hyper::{Request, Response};
use leptos::{get_configuration, LeptosOptions};
use leptos_axum::{generate_route_list, LeptosRoutes};
use sailor_config::Configurable;
use std::{
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;
use tracing::info;

pub type WebOptions = LeptosOptions;

pub async fn load_web_options() -> WebOptions {
    let conf = get_configuration(None).await.unwrap();
    conf.leptos_options
}

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
    pub fn new(configuration: Arc<C>, leptos_options: LeptosOptions) -> Self {
        let routes = generate_route_list(App);

        Self {
            router: Router::new()
                .leptos_routes(&leptos_options, routes, App)
                .fallback(files::file_and_error_handler)
                .with_state(leptos_options),

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
