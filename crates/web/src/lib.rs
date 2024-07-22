mod app;

pub use app::App;

#[cfg(feature = "ssr")]
mod service;

#[cfg(feature = "ssr")]
pub use service::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    leptos::mount_to_body(App);
}
