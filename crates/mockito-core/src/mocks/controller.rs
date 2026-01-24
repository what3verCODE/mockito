//! Controller for managing active routes and switching between collections.
//!
//! This module provides `MocksController` which manages active routes from collections
//! and provides fast route lookup by request matching.

use crate::matching::{
    headers_matches, parse_query_string, payload_matches, query_matches, url_matches,
};
use crate::mocks::manager::{ActiveRoute, MocksManager, ResolveError};
use crate::types::preset::Preset;
use crate::types::route::{HttpMethod, Transport};
use serde_json::Value;
use std::collections::HashMap;

/// HTTP request for route matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// Request URL (path + query string)
    pub url: String,
    /// HTTP method (required for HTTP routes, `None` for WebSocket)
    pub method: Option<HttpMethod>,
    /// Transport type
    pub transport: Transport,
    /// Request headers
    pub headers: Option<HashMap<String, String>>,
    /// Query parameters (parsed from URL if `None`)
    pub query: Option<HashMap<String, String>>,
    /// Request body/payload
    pub payload: Option<Value>,
}

/// Manager for controlling active routes and collection switching.
///
/// `MocksController` provides:
/// - Collection activation via `use_collection()`
/// - Fast route lookup via `find_route()`
/// - Cached active routes for performance
/// - Request matching against route presets
#[derive(Debug, Clone)]
pub struct MocksController {
    /// Mocks manager for storing and resolving collections/routes
    mocks_manager: MocksManager,
    /// Currently active collection ID
    active_collection_id: Option<String>,
    /// Cached active routes from the current collection
    cached_active_routes: Vec<ActiveRoute>,
}

impl MocksController {
    /// Create a new MocksController with MocksManager.
    ///
    /// The controller consumes the manager and uses its data as the source for route resolution.
    /// Data from the manager is read-only - routes and collections should be added to MocksManager
    /// before passing it to the controller.
    pub fn new(mocks_manager: MocksManager) -> Self {
        Self {
            mocks_manager,
            active_collection_id: None,
            cached_active_routes: Vec::new(),
        }
    }

    /// Activate a collection by ID.
    ///
    /// This resolves the collection and caches the active routes for fast lookup.
    /// Returns error if collection not found or resolution fails.
    pub fn use_collection(&mut self, collection_id: &str) -> Result<(), ResolveError> {
        let active_routes = self.mocks_manager.resolve_collection(collection_id)?;
        self.active_collection_id = Some(collection_id.to_string());
        self.cached_active_routes = active_routes;
        Ok(())
    }

    /// Apply specific HTTP routes without changing the entire collection.
    ///
    /// This method allows dynamic route switching by:
    /// - Resolving provided route references (`route:preset:variant`)
    /// - Merging them with existing active routes
    /// - Overriding routes with the same route ID
    ///
    /// # Arguments
    /// * `routes` - Array of route reference strings in format `route_id:preset_id:variant_id`
    ///
    /// # Errors
    /// Returns error if:
    /// - Route, preset, or variant not found
    /// - Route is a WebSocket route (use `use_socket` instead)
    ///
    /// # Example
    /// ```ignore
    /// controller.use_collection("base")?;
    /// controller.use_routes(&["users-api:error:not-found"])?;
    /// ```
    pub fn use_routes(&mut self, routes: &[String]) -> Result<(), ResolveError> {
        // Resolve all new routes first (fail fast if any route is invalid)
        let mut new_routes: Vec<ActiveRoute> = Vec::with_capacity(routes.len());
        for route_ref in routes {
            let active_route = self.mocks_manager.resolve_http_route_reference(route_ref)?;
            new_routes.push(active_route);
        }

        // Build a set of new route IDs for quick lookup
        let new_route_ids: std::collections::HashSet<&str> =
            new_routes.iter().map(|r| r.route.id.as_str()).collect();

        // Merge: keep existing routes that are not overridden, then add new routes
        let mut merged_routes: Vec<ActiveRoute> = self
            .cached_active_routes
            .iter()
            .filter(|existing| !new_route_ids.contains(existing.route.id.as_str()))
            .cloned()
            .collect();

        merged_routes.extend(new_routes);

        self.cached_active_routes = merged_routes;
        Ok(())
    }

    /// Apply specific WebSocket routes without changing the entire collection.
    ///
    /// This method allows dynamic WebSocket route switching by:
    /// - Resolving provided route references (`route:preset:variant`)
    /// - Merging them with existing active routes
    /// - Overriding routes with the same route ID
    ///
    /// # Arguments
    /// * `routes` - Array of route reference strings in format `route_id:preset_id:variant_id`
    ///
    /// # Errors
    /// Returns error if:
    /// - Route, preset, or variant not found
    /// - Route is not a WebSocket route (use `use_routes` instead)
    ///
    /// # Example
    /// ```ignore
    /// controller.use_collection("base")?;
    /// controller.use_socket(&["ws-notifications:default:message"])?;
    /// ```
    pub fn use_socket(&mut self, routes: &[String]) -> Result<(), ResolveError> {
        // Resolve all new routes first (fail fast if any route is invalid)
        let mut new_routes: Vec<ActiveRoute> = Vec::with_capacity(routes.len());
        for route_ref in routes {
            let active_route = self.mocks_manager.resolve_websocket_route_reference(route_ref)?;
            new_routes.push(active_route);
        }

        // Build a set of new route IDs for quick lookup
        let new_route_ids: std::collections::HashSet<&str> =
            new_routes.iter().map(|r| r.route.id.as_str()).collect();

        // Merge: keep existing routes that are not overridden, then add new routes
        let mut merged_routes: Vec<ActiveRoute> = self
            .cached_active_routes
            .iter()
            .filter(|existing| !new_route_ids.contains(existing.route.id.as_str()))
            .cloned()
            .collect();

        merged_routes.extend(new_routes);

        self.cached_active_routes = merged_routes;
        Ok(())
    }

    /// Get all currently active routes.
    ///
    /// Returns cached active routes from the current collection.
    pub fn get_active_routes(&self) -> &[ActiveRoute] {
        &self.cached_active_routes
    }

    /// Get currently active collection ID.
    ///
    /// Returns `None` if no collection is currently active.
    pub fn active_collection_id(&self) -> Option<&str> {
        self.active_collection_id.as_deref()
    }

    /// Find a route that matches the given request.
    ///
    /// Searches through cached active routes and returns the first matching route.
    /// Matching is performed in order: URL, method, transport, headers, query, payload.
    ///
    /// Returns `None` if no matching route is found.
    pub fn find_route(&self, request: &Request) -> Option<&ActiveRoute> {
        self.cached_active_routes
            .iter()
            .find(|active_route| self.route_matches_request(active_route, request))
    }

    /// Check if an active route matches the given request.
    ///
    /// Matches transport, method, URL, headers, query, and payload.
    /// Supports JMESPath expressions for query and payload matching.
    fn route_matches_request(&self, active_route: &ActiveRoute, request: &Request) -> bool {
        let route = &active_route.route;
        let preset = &active_route.preset;

        // Check transport
        if route.transport != request.transport {
            return false;
        }

        // Check HTTP method (for HTTP routes)
        if route.transport == Transport::Http {
            if let Some(route_method) = &route.method {
                if let Some(request_method) = &request.method {
                    if route_method != request_method {
                        return false;
                    }
                } else {
                    return false; // Route requires method but request doesn't have it
                }
            }
        }

        // Check URL pattern
        let url_result = url_matches(&route.url, &request.url);
        if !url_result.matched {
            return false;
        }

        // Check URL path parameters (from preset.params)
        if let Some(expected_params) = &preset.params {
            // URL params are extracted from URL pattern matching
            // Check if all expected params are present in matched params
            for (key, expected_value) in expected_params {
                if let Some(actual_value) = url_result.params.get(key) {
                    if actual_value != expected_value {
                        return false;
                    }
                } else {
                    return false; // Expected param not found
                }
            }
        }

        // Check headers
        let empty_headers = HashMap::new();
        let request_headers = request.headers.as_ref().unwrap_or(&empty_headers);
        if !headers_matches(preset.headers.as_ref(), request_headers) {
            return false;
        }

        // Check query parameters
        let request_query = if let Some(query) = request.query.as_ref() {
            query
        } else {
            // Parse query from URL if not provided separately
            let parsed_query = if let Some(query_str) = request.url.split('?').nth(1) {
                parse_query_string(query_str)
            } else {
                HashMap::new()
            };
            // Use helper method to avoid lifetime issues with temporary
            if !self.check_query_with_parsed(preset, Some(&parsed_query)) {
                return false;
            }
            // Continue to payload check
            return self.check_payload(preset, &request.payload);
        };

        if !query_matches(preset.query.as_ref(), request_query) {
            return false;
        }

        // Check payload/body
        self.check_payload(preset, &request.payload)
    }

    /// Check query parameters with parsed query from URL.
    fn check_query_with_parsed(
        &self,
        preset: &Preset,
        parsed_query: Option<&HashMap<String, String>>,
    ) -> bool {
        let empty_query = HashMap::new();
        query_matches(preset.query.as_ref(), parsed_query.unwrap_or(&empty_query))
    }

    /// Check request payload/body.
    ///
    /// Returns `false` if preset expects payload but request doesn't have it.
    fn check_payload(&self, preset: &Preset, request_payload: &Option<Value>) -> bool {
        if let Some(request_payload) = request_payload {
            payload_matches(preset.payload.as_ref(), request_payload)
        } else if preset.payload.is_some() {
            // Preset expects payload but request doesn't have it
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::collection::Collection;
    use crate::types::preset::{
        HeadersOrExpression, PayloadOrExpression, Preset, QueryOrExpression,
    };
    use crate::types::route::{HttpMethod, Route, Transport};
    use crate::types::variant::Variant;
    use rstest::rstest;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_route(id: &str, url: &str) -> Route {
        Route {
            id: id.to_string(),
            url: url.to_string(),
            transport: Transport::Http,
            method: Some(HttpMethod::Get),
            presets: vec![],
        }
    }

    fn create_test_preset(id: &str) -> Preset {
        Preset {
            id: id.to_string(),
            params: None,
            query: None,
            headers: None,
            payload: None,
            variants: vec![],
        }
    }

    fn create_test_variant(id: &str) -> Variant {
        Variant {
            id: id.to_string(),
            status: Some(200),
            headers: None,
            body: None,
        }
    }

    #[rstest]
    fn test_controller_manager_new() {
        let manager = MocksManager::new();
        let controller = MocksController::new(manager);
        assert_eq!(controller.active_collection_id(), None);
        assert_eq!(controller.get_active_routes().len(), 0);
    }

    #[rstest]
    fn test_use_collection() {
        // Create manager and add routes/collections
        let mut manager = MocksManager::new();
        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        // Create controller with manager
        let mut controller = MocksController::new(manager);

        // Activate collection
        let result = controller.use_collection("collection1");
        assert!(result.is_ok());
        assert_eq!(controller.active_collection_id(), Some("collection1"));
        assert_eq!(controller.get_active_routes().len(), 1);
    }

    #[rstest]
    fn test_use_collection_not_found() {
        let manager = MocksManager::new();
        let mut controller = MocksController::new(manager);
        let result = controller.use_collection("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::CollectionNotFound { .. }
        ));
    }

    #[rstest]
    fn test_get_active_routes() {
        let mut manager = MocksManager::new();

        // Create routes
        let mut route1 = create_test_route("route1", "/api/users");
        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("variant1"));
        route1.presets.push(preset1);
        manager.add_route(route1);

        let mut route2 = create_test_route("route2", "/api/posts");
        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("variant2"));
        route2.presets.push(preset2);
        manager.add_route(route2);

        // Create collection
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec![
                "route1:preset1:variant1".to_string(),
                "route2:preset2:variant2".to_string(),
            ],
        };
        manager.add_collection(collection);

        // Activate collection
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Get active routes
        let active_routes = controller.get_active_routes();
        assert_eq!(active_routes.len(), 2);
        assert_eq!(active_routes[0].route.id, "route1");
        assert_eq!(active_routes[1].route.id, "route2");
    }

    #[rstest]
    fn test_find_route_by_url() {
        let mut manager = MocksManager::new();

        // Create route
        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        // Create collection
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        // Activate collection
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Find route
        let request = Request {
            url: "/api/users".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_some());
        assert_eq!(found.unwrap().route.id, "route1");
    }

    #[rstest]
    fn test_find_route_with_url_params() {
        let mut manager = MocksManager::new();

        // Create route with URL params
        let mut route = create_test_route("route1", "/api/users/{id}");
        let mut preset = create_test_preset("preset1");
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        preset.params = Some(params);
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        // Create collection
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        // Activate collection
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Find route with matching params
        let request = Request {
            url: "/api/users/123".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_some());

        // Find route with non-matching params
        let request = Request {
            url: "/api/users/456".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_find_route_with_headers() {
        let mut manager = MocksManager::new();

        // Create route with headers
        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        preset.headers = Some(HeadersOrExpression::Map(headers));
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        // Create collection
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        // Activate collection
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Find route with matching headers
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        let request = Request {
            url: "/api/users".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: Some(headers),
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_some());

        // Find route with non-matching headers
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer wrong".to_string());
        let request = Request {
            url: "/api/users".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: Some(headers),
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_find_route_with_query() {
        let mut manager = MocksManager::new();

        // Create route with query
        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        let mut query = HashMap::new();
        query.insert("page".to_string(), "1".to_string());
        preset.query = Some(QueryOrExpression::Map(query));
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        // Create collection
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        // Activate collection
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Find route with matching query
        let mut query = HashMap::new();
        query.insert("page".to_string(), "1".to_string());
        let request = Request {
            url: "/api/users?page=1".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: None,
            query: None, // Will be parsed from URL
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_some());

        // Find route with non-matching query
        let request = Request {
            url: "/api/users?page=2".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_find_route_with_payload() {
        let mut manager = MocksManager::new();

        // Create route with payload
        let mut route = create_test_route("route1", "/api/users");
        route.method = Some(HttpMethod::Post);
        let mut preset = create_test_preset("preset1");
        preset.payload = Some(PayloadOrExpression::Value(json!({"name": "John"})));
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        // Create collection
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        // Activate collection
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Find route with matching payload
        let request = Request {
            url: "/api/users".to_string(),
            method: Some(HttpMethod::Post),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: Some(json!({"name": "John"})),
        };

        let found = controller.find_route(&request);
        assert!(found.is_some());

        // Find route with non-matching payload
        let request = Request {
            url: "/api/users".to_string(),
            method: Some(HttpMethod::Post),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: Some(json!({"name": "Jane"})),
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_find_route_not_found() {
        let mut manager = MocksManager::new();

        // Create route
        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        // Create collection
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        // Activate collection
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Find route that doesn't exist
        let request = Request {
            url: "/api/posts".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_switch_collections() {
        let mut manager = MocksManager::new();

        // Create routes
        let mut route1 = create_test_route("route1", "/api/users");
        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("variant1"));
        route1.presets.push(preset1);
        manager.add_route(route1);

        let mut route2 = create_test_route("route2", "/api/posts");
        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("variant2"));
        route2.presets.push(preset2);
        manager.add_route(route2);

        // Create collections
        let collection1 = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection1);

        let collection2 = Collection {
            id: "collection2".to_string(),
            from: None,
            routes: vec!["route2:preset2:variant2".to_string()],
        };
        manager.add_collection(collection2);

        // Activate first collection
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].route.id, "route1");

        // Switch to second collection
        controller.use_collection("collection2").unwrap();
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].route.id, "route2");
    }

    #[rstest]
    fn test_controller_manager_with_manager() {
        let mut manager = MocksManager::new();
        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        assert_eq!(controller.active_collection_id(), None);
        assert_eq!(controller.get_active_routes().len(), 0);

        // Activate collection to verify manager data is used
        controller.use_collection("collection1").unwrap();
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].route.id, "route1");
    }

    #[rstest]
    fn test_find_route_transport_mismatch() {
        let mut manager = MocksManager::new();

        // Create WebSocket route
        let mut route = Route {
            id: "route1".to_string(),
            url: "/ws".to_string(),
            transport: Transport::WebSocket,
            method: None,
            presets: vec![],
        };
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Try to find with HTTP transport
        let request = Request {
            url: "/ws".to_string(),
            method: None,
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_find_route_method_required_but_missing() {
        let mut manager = MocksManager::new();

        // Create route with required method
        let mut route = create_test_route("route1", "/api/users");
        route.method = Some(HttpMethod::Post);
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Request without method
        let request = Request {
            url: "/api/users".to_string(),
            method: None,
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_find_route_method_mismatch() {
        let mut manager = MocksManager::new();

        // Create POST route
        let mut route = create_test_route("route1", "/api/users");
        route.method = Some(HttpMethod::Post);
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Request with GET method
        let request = Request {
            url: "/api/users".to_string(),
            method: Some(HttpMethod::Get),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_find_route_payload_required_but_missing() {
        let mut manager = MocksManager::new();

        // Create route with required payload
        let mut route = create_test_route("route1", "/api/users");
        route.method = Some(HttpMethod::Post);
        let mut preset = create_test_preset("preset1");
        preset.payload = Some(PayloadOrExpression::Value(json!({"name": "John"})));
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Request without payload
        let request = Request {
            url: "/api/users".to_string(),
            method: Some(HttpMethod::Post),
            transport: Transport::Http,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_none());
    }

    #[rstest]
    fn test_find_route_websocket() {
        let mut manager = MocksManager::new();

        // Create WebSocket route
        let mut route = Route {
            id: "route1".to_string(),
            url: "/ws".to_string(),
            transport: Transport::WebSocket,
            method: None,
            presets: vec![],
        };
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);
        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Find WebSocket route
        let request = Request {
            url: "/ws".to_string(),
            method: None,
            transport: Transport::WebSocket,
            headers: None,
            query: None,
            payload: None,
        };

        let found = controller.find_route(&request);
        assert!(found.is_some());
        assert_eq!(found.unwrap().route.id, "route1");
    }

    // ============ use_routes tests ============

    #[rstest]
    fn test_use_routes_switches_variant() {
        let mut manager = MocksManager::new();

        // Create route with two variants
        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        preset.variants.push(create_test_variant("variant2"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Initial state
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].variant.id, "variant1");

        // Switch to variant2 using use_routes
        controller
            .use_routes(&["route1:preset1:variant2".to_string()])
            .unwrap();

        // Should still have 1 route but with variant2
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].variant.id, "variant2");
    }

    #[rstest]
    fn test_use_routes_merges_with_existing() {
        let mut manager = MocksManager::new();

        // Create two routes
        let mut route1 = create_test_route("route1", "/api/users");
        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("variant1"));
        route1.presets.push(preset1);
        manager.add_route(route1);

        let mut route2 = create_test_route("route2", "/api/posts");
        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("variant2"));
        route2.presets.push(preset2);
        manager.add_route(route2);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Initial state: only route1
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].route.id, "route1");

        // Add route2 using use_routes
        controller
            .use_routes(&["route2:preset2:variant2".to_string()])
            .unwrap();

        // Should now have 2 routes
        assert_eq!(controller.get_active_routes().len(), 2);
        let route_ids: Vec<&str> = controller
            .get_active_routes()
            .iter()
            .map(|r| r.route.id.as_str())
            .collect();
        assert!(route_ids.contains(&"route1"));
        assert!(route_ids.contains(&"route2"));
    }

    #[rstest]
    fn test_use_routes_overrides_existing() {
        let mut manager = MocksManager::new();

        // Create route with two presets
        let mut route = create_test_route("route1", "/api/users");

        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("variant1"));

        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("variant2"));

        route.presets.push(preset1);
        route.presets.push(preset2);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Initial: preset1
        assert_eq!(controller.get_active_routes()[0].preset.id, "preset1");

        // Override with preset2
        controller
            .use_routes(&["route1:preset2:variant2".to_string()])
            .unwrap();

        // Should have 1 route with preset2 (not 2 routes)
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].preset.id, "preset2");
    }

    #[rstest]
    fn test_use_routes_without_collection() {
        let mut manager = MocksManager::new();

        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let mut controller = MocksController::new(manager);

        // No collection selected, but use_routes should still work
        assert_eq!(controller.get_active_routes().len(), 0);

        controller
            .use_routes(&["route1:preset1:variant1".to_string()])
            .unwrap();

        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].route.id, "route1");
    }

    #[rstest]
    fn test_use_routes_route_not_found() {
        let manager = MocksManager::new();
        let mut controller = MocksController::new(manager);

        let result = controller.use_routes(&["nonexistent:preset1:variant1".to_string()]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::RouteNotFound { .. }
        ));
    }

    #[rstest]
    fn test_use_routes_preset_not_found() {
        let mut manager = MocksManager::new();

        let route = create_test_route("route1", "/api/users");
        manager.add_route(route);

        let mut controller = MocksController::new(manager);

        let result = controller.use_routes(&["route1:nonexistent:variant1".to_string()]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::PresetNotFound { .. }
        ));
    }

    #[rstest]
    fn test_use_routes_variant_not_found() {
        let mut manager = MocksManager::new();

        let mut route = create_test_route("route1", "/api/users");
        let preset = create_test_preset("preset1");
        // No variants
        route.presets.push(preset);
        manager.add_route(route);

        let mut controller = MocksController::new(manager);

        let result = controller.use_routes(&["route1:preset1:nonexistent".to_string()]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::VariantNotFound { .. }
        ));
    }

    #[rstest]
    fn test_use_routes_invalid_reference_format() {
        let manager = MocksManager::new();
        let mut controller = MocksController::new(manager);

        let result = controller.use_routes(&["invalid-format".to_string()]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::InvalidRouteReference { .. }
        ));
    }

    #[rstest]
    fn test_use_routes_multiple_routes() {
        let mut manager = MocksManager::new();

        // Create three routes
        let mut route1 = create_test_route("route1", "/api/users");
        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("v1"));
        preset1.variants.push(create_test_variant("v2"));
        route1.presets.push(preset1);
        manager.add_route(route1);

        let mut route2 = create_test_route("route2", "/api/posts");
        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("v1"));
        route2.presets.push(preset2);
        manager.add_route(route2);

        let mut route3 = create_test_route("route3", "/api/comments");
        let mut preset3 = create_test_preset("preset3");
        preset3.variants.push(create_test_variant("v1"));
        route3.presets.push(preset3);
        manager.add_route(route3);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec![
                "route1:preset1:v1".to_string(),
                "route2:preset2:v1".to_string(),
            ],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Override route1 and add route3
        controller
            .use_routes(&[
                "route1:preset1:v2".to_string(),
                "route3:preset3:v1".to_string(),
            ])
            .unwrap();

        // Should have 3 routes: route2 (original), route1 (overridden), route3 (new)
        assert_eq!(controller.get_active_routes().len(), 3);

        let routes = controller.get_active_routes();
        let route1 = routes.iter().find(|r| r.route.id == "route1").unwrap();
        let route2 = routes.iter().find(|r| r.route.id == "route2").unwrap();
        let route3 = routes.iter().find(|r| r.route.id == "route3").unwrap();

        assert_eq!(route1.variant.id, "v2"); // Overridden
        assert_eq!(route2.variant.id, "v1"); // Original
        assert_eq!(route3.variant.id, "v1"); // New
    }

    #[rstest]
    fn test_use_routes_fail_fast_on_invalid() {
        let mut manager = MocksManager::new();

        let mut route = create_test_route("route1", "/api/users");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Try to use valid + invalid routes
        let result = controller.use_routes(&[
            "route1:preset1:variant1".to_string(),
            "nonexistent:preset:variant".to_string(),
        ]);

        // Should fail
        assert!(result.is_err());

        // Original routes should remain unchanged (fail fast)
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].route.id, "route1");
    }

    #[rstest]
    fn test_use_routes_rejects_websocket_route() {
        let mut manager = MocksManager::new();

        // Create WebSocket route
        let mut ws_route = Route {
            id: "ws-route".to_string(),
            url: "/ws".to_string(),
            transport: Transport::WebSocket,
            method: None,
            presets: vec![],
        };
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        ws_route.presets.push(preset);
        manager.add_route(ws_route);

        let mut controller = MocksController::new(manager);

        // Try to use WebSocket route with use_routes (should fail)
        let result = controller.use_routes(&["ws-route:preset1:variant1".to_string()]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::TransportMismatch { .. }
        ));
    }

    // ============ use_socket tests ============

    fn create_test_ws_route(id: &str, url: &str) -> Route {
        Route {
            id: id.to_string(),
            url: url.to_string(),
            transport: Transport::WebSocket,
            method: None,
            presets: vec![],
        }
    }

    #[rstest]
    fn test_use_socket_basic() {
        let mut manager = MocksManager::new();

        let mut ws_route = create_test_ws_route("ws-route", "/ws/notifications");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        ws_route.presets.push(preset);
        manager.add_route(ws_route);

        let mut controller = MocksController::new(manager);

        // Use socket route
        controller
            .use_socket(&["ws-route:preset1:variant1".to_string()])
            .unwrap();

        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].route.id, "ws-route");
        assert_eq!(
            controller.get_active_routes()[0].route.transport,
            Transport::WebSocket
        );
    }

    #[rstest]
    fn test_use_socket_switches_variant() {
        let mut manager = MocksManager::new();

        // Create WebSocket route with two variants
        let mut ws_route = create_test_ws_route("ws-route", "/ws");
        let mut preset = create_test_preset("default");
        preset.variants.push(create_test_variant("message"));
        preset.variants.push(create_test_variant("error"));
        ws_route.presets.push(preset);
        manager.add_route(ws_route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["ws-route:default:message".to_string()],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Initial state
        assert_eq!(controller.get_active_routes()[0].variant.id, "message");

        // Switch to error variant
        controller
            .use_socket(&["ws-route:default:error".to_string()])
            .unwrap();

        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].variant.id, "error");
    }

    #[rstest]
    fn test_use_socket_merges_with_existing() {
        let mut manager = MocksManager::new();

        // Create two WS routes
        let mut ws_route1 = create_test_ws_route("ws-route1", "/ws/1");
        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("variant1"));
        ws_route1.presets.push(preset1);
        manager.add_route(ws_route1);

        let mut ws_route2 = create_test_ws_route("ws-route2", "/ws/2");
        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("variant2"));
        ws_route2.presets.push(preset2);
        manager.add_route(ws_route2);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["ws-route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Add second WS route
        controller
            .use_socket(&["ws-route2:preset2:variant2".to_string()])
            .unwrap();

        // Should have 2 routes
        assert_eq!(controller.get_active_routes().len(), 2);
        let route_ids: Vec<&str> = controller
            .get_active_routes()
            .iter()
            .map(|r| r.route.id.as_str())
            .collect();
        assert!(route_ids.contains(&"ws-route1"));
        assert!(route_ids.contains(&"ws-route2"));
    }

    #[rstest]
    fn test_use_socket_rejects_http_route() {
        let mut manager = MocksManager::new();

        // Create HTTP route
        let mut http_route = create_test_route("http-route", "/api/users");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        http_route.presets.push(preset);
        manager.add_route(http_route);

        let mut controller = MocksController::new(manager);

        // Try to use HTTP route with use_socket (should fail)
        let result = controller.use_socket(&["http-route:preset1:variant1".to_string()]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ResolveError::TransportMismatch { .. }));

        // Check error message contains suggestion
        let error_msg = error.to_string();
        assert!(error_msg.contains("Use 'useRoutes' instead"));
    }

    #[rstest]
    fn test_use_socket_route_not_found() {
        let manager = MocksManager::new();
        let mut controller = MocksController::new(manager);

        let result = controller.use_socket(&["nonexistent:preset1:variant1".to_string()]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::RouteNotFound { .. }
        ));
    }

    #[rstest]
    fn test_use_socket_preset_not_found() {
        let mut manager = MocksManager::new();

        let ws_route = create_test_ws_route("ws-route", "/ws");
        manager.add_route(ws_route);

        let mut controller = MocksController::new(manager);

        let result = controller.use_socket(&["ws-route:nonexistent:variant1".to_string()]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::PresetNotFound { .. }
        ));
    }

    #[rstest]
    fn test_use_socket_variant_not_found() {
        let mut manager = MocksManager::new();

        let mut ws_route = create_test_ws_route("ws-route", "/ws");
        let preset = create_test_preset("preset1");
        // No variants
        ws_route.presets.push(preset);
        manager.add_route(ws_route);

        let mut controller = MocksController::new(manager);

        let result = controller.use_socket(&["ws-route:preset1:nonexistent".to_string()]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::VariantNotFound { .. }
        ));
    }

    #[rstest]
    fn test_use_socket_fail_fast_on_invalid() {
        let mut manager = MocksManager::new();

        let mut ws_route = create_test_ws_route("ws-route", "/ws");
        let mut preset = create_test_preset("preset1");
        preset.variants.push(create_test_variant("variant1"));
        ws_route.presets.push(preset);
        manager.add_route(ws_route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["ws-route:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let mut controller = MocksController::new(manager);
        controller.use_collection("collection1").unwrap();

        // Try to use valid + invalid routes
        let result = controller.use_socket(&[
            "ws-route:preset1:variant1".to_string(),
            "nonexistent:preset:variant".to_string(),
        ]);

        // Should fail
        assert!(result.is_err());

        // Original routes should remain unchanged
        assert_eq!(controller.get_active_routes().len(), 1);
        assert_eq!(controller.get_active_routes()[0].route.id, "ws-route");
    }

    #[rstest]
    fn test_use_socket_multiple_routes() {
        let mut manager = MocksManager::new();

        // Create two WS routes
        let mut ws_route1 = create_test_ws_route("ws-route1", "/ws/1");
        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("v1"));
        ws_route1.presets.push(preset1);
        manager.add_route(ws_route1);

        let mut ws_route2 = create_test_ws_route("ws-route2", "/ws/2");
        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("v1"));
        ws_route2.presets.push(preset2);
        manager.add_route(ws_route2);

        let mut controller = MocksController::new(manager);

        // Add multiple routes at once
        controller
            .use_socket(&[
                "ws-route1:preset1:v1".to_string(),
                "ws-route2:preset2:v1".to_string(),
            ])
            .unwrap();

        assert_eq!(controller.get_active_routes().len(), 2);
    }
}
