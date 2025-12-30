use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response variant for a preset
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    fn test_variant_serialize_deserialize() {
        let variant = Variant {
            id: "test-variant".to_string(),
            status: Some(200),
            headers: Some({
                let mut map = HashMap::new();
                map.insert("Content-Type".to_string(), "application/json".to_string());
                map
            }),
            body: Some(json!({"message": "success"})),
        };

        let json = serde_json::to_string(&variant).expect("Should serialize");
        let deserialized: Variant = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.id, variant.id);
        assert_eq!(deserialized.status, variant.status);
        assert_eq!(deserialized.headers, variant.headers);
        assert_eq!(deserialized.body, variant.body);
    }

    #[rstest]
    #[case("status")]
    #[case("headers")]
    #[case("body")]
    fn test_variant_optional_fields_omitted_when_none(#[case] field: &str) {
        let variant = Variant {
            id: "minimal-variant".to_string(),
            status: None,
            headers: None,
            body: None,
        };

        let json = serde_json::to_string(&variant).expect("Should serialize");
        assert!(
            !json.contains(field),
            "Field '{}' should be omitted when None",
            field
        );

        let deserialized: Variant = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.id, variant.id);
        assert_eq!(deserialized.status, None);
        assert_eq!(deserialized.headers, None);
        assert_eq!(deserialized.body, None);
    }

    #[rstest]
    #[case(200)]
    #[case(201)]
    #[case(400)]
    #[case(404)]
    #[case(500)]
    #[case(503)]
    fn test_variant_status_codes(#[case] status: u16) {
        let variant = Variant {
            id: "test".to_string(),
            status: Some(status),
            headers: None,
            body: None,
        };

        let json = serde_json::to_string(&variant).expect("Should serialize");
        let deserialized: Variant = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized, variant);
    }
}
