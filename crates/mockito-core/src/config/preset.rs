use crate::config::variant::Variant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request matching preset with response variants
#[derive(Debug, Serialize, Deserialize)]
pub struct Preset {
    /// Unique identifier for this preset within the route
    pub id: String,
    /// URL path parameters to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
    /// Query parameters to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<HashMap<String, String>>,
    /// Request headers to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Request body to match (for POST/PUT/PATCH)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<HashMap<String, serde_json::Value>>,
    /// Response variants
    pub variants: Vec<Variant>,
}
