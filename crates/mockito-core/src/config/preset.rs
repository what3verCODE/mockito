use crate::config::variant::Variant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request matching preset with response variants
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::variant::Variant;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    fn test_preset_serialize_deserialize() {
        let preset = Preset {
            id: "test-preset".to_string(),
            params: Some({
                let mut map = HashMap::new();
                map.insert("id".to_string(), "123".to_string());
                map
            }),
            query: Some({
                let mut map = HashMap::new();
                map.insert("page".to_string(), "1".to_string());
                map
            }),
            headers: Some({
                let mut map = HashMap::new();
                map.insert("Authorization".to_string(), "Bearer token".to_string());
                map
            }),
            payload: Some({
                let mut map = HashMap::new();
                map.insert("name".to_string(), json!("John"));
                map
            }),
            variants: vec![],
        };

        let json = serde_json::to_string(&preset).expect("Should serialize");
        let deserialized: Preset = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.id, preset.id);
        assert_eq!(deserialized.params, preset.params);
        assert_eq!(deserialized.query, preset.query);
        assert_eq!(deserialized.headers, preset.headers);
        assert_eq!(deserialized.payload, preset.payload);
    }

    #[rstest]
    #[case("params")]
    #[case("query")]
    #[case("headers")]
    #[case("payload")]
    fn test_preset_optional_fields_omitted_when_none(#[case] field: &str) {
        let preset = Preset {
            id: "minimal-preset".to_string(),
            params: None,
            query: None,
            headers: None,
            payload: None,
            variants: vec![],
        };

        let json = serde_json::to_string(&preset).expect("Should serialize");
        assert!(
            !json.contains(field),
            "Field '{}' should be omitted when None",
            field
        );

        let deserialized: Preset = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.id, preset.id);
        assert_eq!(deserialized.params, None);
        assert_eq!(deserialized.query, None);
        assert_eq!(deserialized.headers, None);
        assert_eq!(deserialized.payload, None);
    }

    #[rstest]
    #[case("variant1", Some(200))]
    #[case("variant2", Some(404))]
    #[case("variant3", None)]
    fn test_preset_with_variants(#[case] variant_id: &str, #[case] status: Option<u16>) {
        let variant = Variant {
            id: variant_id.to_string(),
            status,
            headers: None,
            body: None,
        };

        let preset = Preset {
            id: "preset-with-variants".to_string(),
            params: None,
            query: None,
            headers: None,
            payload: None,
            variants: vec![variant],
        };

        let json = serde_json::to_string(&preset).expect("Should serialize");
        let deserialized: Preset = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.variants.len(), 1);
        assert_eq!(deserialized.variants[0].id, variant_id);
        assert_eq!(deserialized.variants[0].status, status);
    }

    #[rstest]
    #[case("id", "value")]
    #[case("userId", "456")]
    #[case("param-name", "param-value")]
    fn test_preset_params_matching(#[case] param_key: &str, #[case] param_value: &str) {
        let mut params = HashMap::new();
        params.insert(param_key.to_string(), param_value.to_string());

        let preset = Preset {
            id: "test".to_string(),
            params: Some(params.clone()),
            query: None,
            headers: None,
            payload: None,
            variants: vec![],
        };

        let json = serde_json::to_string(&preset).expect("Should serialize");
        let deserialized: Preset = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.params, Some(params));
    }
}
