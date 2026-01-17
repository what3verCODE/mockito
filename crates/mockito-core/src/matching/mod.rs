//! Request matching utilities.

mod headers;
mod intersection;
mod payload;
mod query;
mod url;

pub use headers::{headers_intersects, headers_matches};
pub use intersection::{hashmap_intersects, object_intersects};
pub use payload::payload_matches;
pub use query::{parse_query_string, query_matches};
pub use url::{url_matches, UrlMatchResult};
