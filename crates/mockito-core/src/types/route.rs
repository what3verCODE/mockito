//! Core route types.

use crate::types::preset::Preset;
use serde::{Deserialize, Serialize};

/// Transport type for route matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Transport {
    Http,
    WebSocket,
}

/// HTTP method for route matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Parsed route reference in format `route_id:preset_id:variant_id`.
pub struct RouteReference {
    pub route_id: String,
    pub preset_id: String,
    pub variant_id: String,
}

impl RouteReference {
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return None;
        }

        // Safe to unwrap because we checked parts.len() == 3
        let route_id = parts[0];
        let preset_id = parts[1];
        let variant_id = parts[2];

        if route_id.is_empty() || preset_id.is_empty() || variant_id.is_empty() {
            return None;
        }

        Some(Self {
            route_id: route_id.to_owned(),
            preset_id: preset_id.to_owned(),
            variant_id: variant_id.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("route1:preset1:variant1", Some(("route1", "preset1", "variant1")))]
    #[case("a:b:c", Some(("a", "b", "c")))]
    #[case("route-id:preset-id:variant-id", Some(("route-id", "preset-id", "variant-id")))]
    fn test_route_reference_parse_valid(
        #[case] input: &str,
        #[case] expected: Option<(&str, &str, &str)>,
    ) {
        let result = RouteReference::parse(input);
        if let Some(expected) = expected {
            let parsed = result.expect("Should parse successfully");
            assert_eq!(parsed.route_id, expected.0);
            assert_eq!(parsed.preset_id, expected.1);
            assert_eq!(parsed.variant_id, expected.2);
        } else {
            assert!(result.is_none());
        }
    }

    #[rstest]
    #[case("")]
    #[case("route1")]
    #[case("route1:preset1")]
    #[case("route1:preset1:variant1:extra")]
    #[case(":preset1:variant1")]
    #[case("route1::variant1")]
    #[case("route1:preset1:")]
    #[case("::")]
    fn test_route_reference_parse_invalid(#[case] input: &str) {
        assert!(RouteReference::parse(input).is_none());
    }

    #[rstest]
    #[case(Transport::Http)]
    #[case(Transport::WebSocket)]
    fn test_transport_roundtrip(#[case] transport: Transport) {
        let json = serde_json::to_string(&transport).expect("Should serialize");
        let deserialized: Transport = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized, transport);
    }

    #[rstest]
    #[case(HttpMethod::Get)]
    #[case(HttpMethod::Post)]
    #[case(HttpMethod::Put)]
    #[case(HttpMethod::Patch)]
    #[case(HttpMethod::Delete)]
    #[case(HttpMethod::Head)]
    #[case(HttpMethod::Options)]
    fn test_http_method_roundtrip(#[case] method: HttpMethod) {
        let json = serde_json::to_string(&method).expect("Should serialize");
        let deserialized: HttpMethod = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized, method);
    }

    #[rstest]
    #[case(
        "test-route",
        "/api/users/{id}",
        Transport::Http,
        Some(HttpMethod::Get),
        true
    )]
    #[case("websocket-route", "/ws", Transport::WebSocket, None, false)]
    #[case(
        "post-route",
        "/api/posts",
        Transport::Http,
        Some(HttpMethod::Post),
        true
    )]
    fn test_route_serialize_deserialize(
        #[case] id: &str,
        #[case] url: &str,
        #[case] transport: Transport,
        #[case] method: Option<HttpMethod>,
        #[case] should_have_method: bool,
    ) {
        let route = Route {
            id: id.to_string(),
            url: url.to_string(),
            transport,
            method,
            presets: vec![],
        };

        let json = serde_json::to_string(&route).expect("Should serialize");

        if !should_have_method {
            assert!(!json.contains("method"));
        }

        let deserialized: Route = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.id, route.id);
        assert_eq!(deserialized.url, route.url);
        assert_eq!(deserialized.transport, route.transport);
        assert_eq!(deserialized.method, route.method);
        assert_eq!(deserialized.presets.len(), 0);
    }
}
