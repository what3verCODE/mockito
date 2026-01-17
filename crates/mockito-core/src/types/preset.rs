//! Request matching preset types.

use crate::expression::is_expression;
use crate::types::variant::Variant;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

/// Query parameters value - either a map or an expression string
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOrExpression {
    Map(HashMap<String, String>),
    Expression(String),
}

impl Serialize for QueryOrExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            QueryOrExpression::Map(map) => map.serialize(serializer),
            QueryOrExpression::Expression(expr) => expr.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for QueryOrExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(s) if is_expression(&s) => {
                // Extract expression without ${}
                let expr = s
                    .strip_prefix("${")
                    .and_then(|s| s.strip_suffix('}'))
                    .unwrap_or(&s);
                Ok(QueryOrExpression::Expression(expr.to_string()))
            }
            Value::Object(map) => {
                let mut result = HashMap::new();
                for (k, v) in map {
                    if let Some(s) = v.as_str() {
                        result.insert(k, s.to_string());
                    }
                }
                Ok(QueryOrExpression::Map(result))
            }
            _ => Err(serde::de::Error::custom(
                "Query must be either an object or an expression string",
            )),
        }
    }
}

/// Headers value - either a map or an expression string
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadersOrExpression {
    Map(HashMap<String, String>),
    Expression(String),
}

impl Serialize for HeadersOrExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            HeadersOrExpression::Map(map) => map.serialize(serializer),
            HeadersOrExpression::Expression(expr) => expr.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for HeadersOrExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(s) if is_expression(&s) => {
                // Extract expression without ${}
                let expr = s
                    .strip_prefix("${")
                    .and_then(|s| s.strip_suffix('}'))
                    .unwrap_or(&s);
                Ok(HeadersOrExpression::Expression(expr.to_string()))
            }
            Value::Object(map) => {
                let mut result = HashMap::new();
                for (k, v) in map {
                    if let Some(s) = v.as_str() {
                        result.insert(k, s.to_string());
                    }
                }
                Ok(HeadersOrExpression::Map(result))
            }
            _ => Err(serde::de::Error::custom(
                "Headers must be either an object or an expression string",
            )),
        }
    }
}

/// Payload value - either a JSON value or an expression string
#[derive(Debug, Clone, PartialEq)]
pub enum PayloadOrExpression {
    Value(Value),
    Expression(String),
}

impl Serialize for PayloadOrExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PayloadOrExpression::Value(v) => v.serialize(serializer),
            PayloadOrExpression::Expression(expr) => {
                // Serialize as ${expr}
                format!("${{{}}}", expr).serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for PayloadOrExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match &value {
            Value::String(s) if is_expression(s) => {
                // Extract expression without ${}
                let expr = s
                    .strip_prefix("${")
                    .and_then(|s| s.strip_suffix('}'))
                    .unwrap_or(s);
                Ok(PayloadOrExpression::Expression(expr.to_string()))
            }
            _ => Ok(PayloadOrExpression::Value(value)),
        }
    }
}

/// Request matching preset with response variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    /// Unique identifier for this preset within the route
    pub id: String,
    /// URL path parameters to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
    /// Query parameters to match (can be a map or expression string like "${query.page == '1'}")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<QueryOrExpression>,
    /// Request headers to match (can be a map or expression string like "${headers.myheader == 1}")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HeadersOrExpression>,
    /// Request body to match (can be any JSON value or expression string like "${payload.items[0].id == 5}")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<PayloadOrExpression>,
    /// Response variants
    pub variants: Vec<Variant>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::variant::Variant;
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
            query: Some(QueryOrExpression::Map({
                let mut map = HashMap::new();
                map.insert("page".to_string(), "1".to_string());
                map
            })),
            headers: Some(HeadersOrExpression::Map({
                let mut map = HashMap::new();
                map.insert("Authorization".to_string(), "Bearer token".to_string());
                map
            })),
            payload: Some(PayloadOrExpression::Value(json!({"name": "John"}))),
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
