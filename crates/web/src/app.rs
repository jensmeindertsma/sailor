use axum::{extract::Host, response::Html, routing::get, Router};
use http::StatusCode;

pub fn create_router() -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Html("<h1>Hello, World! Welcome to the interface!</h1>\n") }),
        )
        .route(
            "/proxy-error",
            get(|host: Option<Host>| async move {
                (
                    StatusCode::NOT_FOUND,
                    Html(match host {
                        None => "<h1>[sailor] no host header?</h1>\n".to_string(),
                        Some(Host(host)) => {
                            format!("<h1>[sailor] unknown host {host}</h1>\n")
                        }
                    }),
                )
            }),
        )
}
