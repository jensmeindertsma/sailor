use axum::{extract::Host, response::Html, routing::get, Json, Router};
use http::StatusCode;
use sailor_core::proxy::ProxyError;

pub fn create_router() -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Html("<h1>Hello, World! Welcome to the interface!</h1>\n") }),
        )
        .route(
            "/proxy-error",
            get(
                |host: Option<Host>, error: Option<Json<ProxyError>>| async move {
                    if let Some(Json(err)) = error {
                        match err {
                            ProxyError::FetchError(fetch_error) => (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Html(format!("<h1>[sailor] failed to connect to target address</h1><pre><code>{:?}</code></pre>\n", fetch_error))
                            ),
                        }
                    } else {
                        (
                            StatusCode::NOT_FOUND,
                            Html(match host {
                                None => "<h1>[sailor] no host header?</h1>\n".to_string(),
                                Some(Host(host)) => {
                                    format!("<h1>[sailor] unknown host {host}</h1>\n")
                                }
                            }),
                        )
                    }
                },
            ),
        )
}
