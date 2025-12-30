use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response variant for a preset
#[derive(Debug, Serialize, Deserialize)]
pub struct Variant {
    /// Unique identifier for this variant within the preset
    pub id: String,
    /// HTTP status code for the response (100-599)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// Response headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Response body (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
    /// Path to file to serve as response body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// URL to proxy the request to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Delay in milliseconds before sending response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<u64>,
}
