//! Core route configuration types.

use crate::config::preset::Preset;
use serde::{Deserialize, Serialize};

/// Transport type for route matching
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Transport {
    Http,
    WebSocket,
}

/// HTTP method for route matching
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

/// Mock route definition
#[derive(Serialize, Deserialize)]
pub struct Route {
    /// Unique identifier for this route
    pub id: String,
    /// URL pattern (supports {param} placeholders)
    pub url: String,
    /// Transport type (HTTP or WebSocket)
    pub transport: Transport,
    /// HTTP method (for HTTP routes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<HttpMethod>,
    /// Request matching presets
    pub presets: Vec<Preset>,
}
