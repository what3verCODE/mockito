//! NAPI-RS bindings for Mockito core library.
//!
//! Exposes Rust core API to Node.js.

use napi_derive::napi;

mod config;
mod mocks;

pub use config::*;
pub use mocks::*;

/// Library version
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
