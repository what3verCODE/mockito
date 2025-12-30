//! Request matching utilities.

mod headers;
mod url;

pub use headers::headers_intersects;
pub use url::{url_matches, UrlMatchResult};
