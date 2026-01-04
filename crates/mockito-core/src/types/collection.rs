//! Collection types.

use serde::{Deserialize, Serialize};

/// Collection of routes for a specific scenario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    /// Unique identifier for this collection
    pub id: String,
    /// ID of parent collection to inherit routes from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// List of route references in format 'routeId:presetId:variantId'
    pub routes: Vec<String>,
}
