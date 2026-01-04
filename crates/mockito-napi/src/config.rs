//! Config parsing bindings for Node.js.

use mockito_core::types::{
    collection::Collection as CoreCollection,
    preset::Preset as CorePreset,
    route::{HttpMethod as CoreHttpMethod, Route as CoreRoute, Transport as CoreTransport},
    variant::Variant as CoreVariant,
};
use napi_derive::napi;
use std::collections::HashMap;

/// Transport type for route matching
#[napi]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Http,
    WebSocket,
}

impl From<CoreTransport> for Transport {
    fn from(t: CoreTransport) -> Self {
        match t {
            CoreTransport::Http => Transport::Http,
            CoreTransport::WebSocket => Transport::WebSocket,
        }
    }
}

impl From<Transport> for CoreTransport {
    fn from(t: Transport) -> Self {
        match t {
            Transport::Http => CoreTransport::Http,
            Transport::WebSocket => CoreTransport::WebSocket,
        }
    }
}

/// HTTP method for route matching
#[napi]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl From<CoreHttpMethod> for HttpMethod {
    fn from(m: CoreHttpMethod) -> Self {
        match m {
            CoreHttpMethod::Get => HttpMethod::Get,
            CoreHttpMethod::Post => HttpMethod::Post,
            CoreHttpMethod::Put => HttpMethod::Put,
            CoreHttpMethod::Patch => HttpMethod::Patch,
            CoreHttpMethod::Delete => HttpMethod::Delete,
            CoreHttpMethod::Head => HttpMethod::Head,
            CoreHttpMethod::Options => HttpMethod::Options,
        }
    }
}

impl From<HttpMethod> for CoreHttpMethod {
    fn from(m: HttpMethod) -> Self {
        match m {
            HttpMethod::Get => CoreHttpMethod::Get,
            HttpMethod::Post => CoreHttpMethod::Post,
            HttpMethod::Put => CoreHttpMethod::Put,
            HttpMethod::Patch => CoreHttpMethod::Patch,
            HttpMethod::Delete => CoreHttpMethod::Delete,
            HttpMethod::Head => CoreHttpMethod::Head,
            HttpMethod::Options => CoreHttpMethod::Options,
        }
    }
}

/// Response variant
#[napi(object)]
#[derive(Clone)]
pub struct Variant {
    pub id: String,
    pub status: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
}

impl From<CoreVariant> for Variant {
    fn from(v: CoreVariant) -> Self {
        Self {
            id: v.id,
            status: v.status.map(|s| s as u32),
            headers: v.headers,
            body: v.body,
        }
    }
}

impl From<&CoreVariant> for Variant {
    fn from(v: &CoreVariant) -> Self {
        Self {
            id: v.id.clone(),
            status: v.status.map(|s| s as u32),
            headers: v.headers.clone(),
            body: v.body.clone(),
        }
    }
}

/// Request matching preset
#[napi(object)]
#[derive(Clone)]
pub struct Preset {
    pub id: String,
    pub variants: Vec<Variant>,
    pub headers: Option<HashMap<String, String>>,
    pub query: Option<HashMap<String, String>>,
    pub query_expr: Option<String>,
    pub params: Option<HashMap<String, String>>,
    pub payload: Option<serde_json::Value>,
    pub payload_expr: Option<String>,
}

impl From<CorePreset> for Preset {
    fn from(p: CorePreset) -> Self {
        Self {
            id: p.id,
            variants: p.variants.into_iter().map(Variant::from).collect(),
            headers: p.headers,
            query: p.query,
            query_expr: p.query_expr,
            params: p.params,
            payload: p
                .payload
                .map(|h| serde_json::to_value(h).unwrap_or_default()),
            payload_expr: p.payload_expr,
        }
    }
}

impl From<&CorePreset> for Preset {
    fn from(p: &CorePreset) -> Self {
        Self {
            id: p.id.clone(),
            variants: p.variants.iter().map(Variant::from).collect(),
            headers: p.headers.clone(),
            query: p.query.clone(),
            query_expr: p.query_expr.clone(),
            params: p.params.clone(),
            payload: p
                .payload
                .as_ref()
                .map(|h| serde_json::to_value(h).unwrap_or_default()),
            payload_expr: p.payload_expr.clone(),
        }
    }
}

/// Route definition
#[napi(object)]
#[derive(Clone)]
pub struct Route {
    pub id: String,
    pub url: String,
    pub transport: Transport,
    pub method: Option<HttpMethod>,
    pub presets: Vec<Preset>,
}

impl From<CoreRoute> for Route {
    fn from(r: CoreRoute) -> Self {
        Self {
            id: r.id,
            url: r.url,
            transport: r.transport.into(),
            method: r.method.map(|m| m.into()),
            presets: r.presets.into_iter().map(Preset::from).collect(),
        }
    }
}

impl From<&CoreRoute> for Route {
    fn from(r: &CoreRoute) -> Self {
        Self {
            id: r.id.clone(),
            url: r.url.clone(),
            transport: r.transport.clone().into(),
            method: r.method.clone().map(|m| m.into()),
            presets: r.presets.iter().map(Preset::from).collect(),
        }
    }
}

// =============================================================================
// Reverse conversions: Napi -> Core types
// =============================================================================

impl From<Variant> for CoreVariant {
    fn from(v: Variant) -> Self {
        Self {
            id: v.id,
            status: v.status.map(|s| s as u16),
            headers: v.headers,
            body: v.body,
        }
    }
}

impl From<&Variant> for CoreVariant {
    fn from(v: &Variant) -> Self {
        Self {
            id: v.id.clone(),
            status: v.status.map(|s| s as u16),
            headers: v.headers.clone(),
            body: v.body.clone(),
        }
    }
}

impl From<Preset> for CorePreset {
    fn from(p: Preset) -> Self {
        Self {
            id: p.id,
            variants: p.variants.into_iter().map(CoreVariant::from).collect(),
            headers: p.headers,
            query: p.query,
            query_expr: p.query_expr,
            params: p.params,
            payload: p.payload.and_then(|v| serde_json::from_value(v).ok()),
            payload_expr: p.payload_expr,
        }
    }
}

impl From<&Preset> for CorePreset {
    fn from(p: &Preset) -> Self {
        Self {
            id: p.id.clone(),
            variants: p.variants.iter().map(CoreVariant::from).collect(),
            headers: p.headers.clone(),
            query: p.query.clone(),
            query_expr: p.query_expr.clone(),
            params: p.params.clone(),
            payload: p
                .payload
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            payload_expr: p.payload_expr.clone(),
        }
    }
}

impl From<Route> for CoreRoute {
    fn from(r: Route) -> Self {
        Self {
            id: r.id,
            url: r.url,
            transport: r.transport.into(),
            method: r.method.map(|m| m.into()),
            presets: r.presets.into_iter().map(CorePreset::from).collect(),
        }
    }
}

impl From<&Route> for CoreRoute {
    fn from(r: &Route) -> Self {
        Self {
            id: r.id.clone(),
            url: r.url.clone(),
            transport: r.transport.into(),
            method: r.method.map(|m| m.into()),
            presets: r.presets.iter().map(CorePreset::from).collect(),
        }
    }
}

/// Collection of routes
#[napi(object)]
#[derive(Clone)]
pub struct Collection {
    pub id: String,
    pub from: Option<String>,
    pub routes: Vec<String>,
}

impl From<CoreCollection> for Collection {
    fn from(c: CoreCollection) -> Self {
        Self {
            id: c.id,
            from: c.from,
            routes: c.routes,
        }
    }
}

impl From<&CoreCollection> for Collection {
    fn from(c: &CoreCollection) -> Self {
        Self {
            id: c.id.clone(),
            from: c.from.clone(),
            routes: c.routes.clone(),
        }
    }
}

impl From<Collection> for CoreCollection {
    fn from(c: Collection) -> Self {
        Self {
            id: c.id,
            from: c.from,
            routes: c.routes,
        }
    }
}

impl From<&Collection> for CoreCollection {
    fn from(c: &Collection) -> Self {
        Self {
            id: c.id.clone(),
            from: c.from.clone(),
            routes: c.routes.clone(),
        }
    }
}
