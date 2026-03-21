pub mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod grpc;
pub mod ui;
pub mod util;
pub mod views;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::WebHandle;
