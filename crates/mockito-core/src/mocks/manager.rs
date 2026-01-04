//! Mocks manager for managing routes and collections.
//!
//! This module provides `MocksManager` which stores and resolves collections and routes.
//! It is used by `MocksController` for handling dynamic changes to mocked routes
//! from added collections/routes.

use crate::types::collection::Collection;
use crate::types::preset::Preset;
use crate::types::route::{Route, RouteReference};
use crate::types::variant::Variant;
use std::collections::{HashMap, HashSet};

/// Active route with selected preset and variant.
///
/// Represents a fully resolved route that can be used for mocking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveRoute {
    /// Base route definition
    pub route: Route,
    /// Selected preset from the route
    pub preset: Preset,
    /// Selected variant from the preset
    pub variant: Variant,
}

/// Manager for storing and resolving collections and routes.
///
/// `MocksManager` is responsible for:
/// - Storing collections and routes
/// - Resolving collections with inheritance support
/// - Detecting circular dependencies
/// - Merging routes (child collections override parent routes)
///
/// This manager is used by `MocksController` to handle dynamic changes
/// to mocked routes from added collections/routes.
#[derive(Debug, Clone)]
pub struct MocksManager {
    /// Map of collection ID to Collection
    collections: HashMap<String, Collection>,
    /// Map of route ID to Route
    routes: HashMap<String, Route>,
}

impl MocksManager {
    /// Create a new MocksManager
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
            routes: HashMap::new(),
        }
    }

    /// Add a collection to the manager
    pub fn add_collection(&mut self, collection: Collection) {
        self.collections.insert(collection.id.clone(), collection);
    }

    /// Add multiple collections to the manager
    pub fn add_collections(&mut self, collections: Vec<Collection>) {
        for collection in collections {
            self.add_collection(collection);
        }
    }

    /// Add a route to the manager
    pub fn add_route(&mut self, route: Route) {
        self.routes.insert(route.id.clone(), route);
    }

    /// Add multiple routes to the manager
    pub fn add_routes(&mut self, routes: Vec<Route>) {
        for route in routes {
            self.add_route(route);
        }
    }

    /// Resolve a collection by ID, returning all active routes.
    ///
    /// Supports inheritance via `from` field and detects circular dependencies.
    /// Child collections override parent routes with the same route_id.
    pub fn resolve_collection(
        &self,
        collection_id: &str,
    ) -> Result<Vec<ActiveRoute>, ResolveError> {
        let mut visited = HashSet::new();
        let mut route_map = HashMap::new(); // route_id -> ActiveRoute (for deduplication)

        self.resolve_collection_recursive(collection_id, &mut visited, &mut route_map)?;

        // Convert HashMap to Vec, preserving order from collections
        let mut result = Vec::new();
        let mut processed_routes = HashSet::new();

        // Process routes in order: first from parent, then from child
        self.collect_routes_in_order(
            collection_id,
            &mut processed_routes,
            &route_map,
            &mut result,
        )?;

        Ok(result)
    }

    /// Recursively resolve collection with inheritance support.
    ///
    /// Detects circular dependencies and resolves parent collections first.
    /// Child routes override parent routes with the same route_id.
    fn resolve_collection_recursive(
        &self,
        collection_id: &str,
        visited: &mut HashSet<String>,
        route_map: &mut HashMap<String, ActiveRoute>,
    ) -> Result<(), ResolveError> {
        // Detect circular dependency
        if visited.contains(collection_id) {
            return Err(ResolveError::CircularDependency {
                collection_id: collection_id.to_string(),
            });
        }

        // Get collection
        let collection = self.collections.get(collection_id).ok_or_else(|| {
            ResolveError::CollectionNotFound {
                collection_id: collection_id.to_string(),
            }
        })?;

        // Mark as visited
        visited.insert(collection_id.to_string());

        // First, resolve parent collection if exists
        if let Some(parent_id) = &collection.from {
            self.resolve_collection_recursive(parent_id, visited, route_map)?;
        }

        // Then, resolve current collection's routes (child overrides parent)
        for route_ref_str in &collection.routes {
            let route_ref = RouteReference::parse(route_ref_str).ok_or_else(|| {
                ResolveError::InvalidRouteReference {
                    reference: route_ref_str.clone(),
                }
            })?;

            // Get route
            let route = self.routes.get(&route_ref.route_id).ok_or_else(|| {
                ResolveError::RouteNotFound {
                    route_id: route_ref.route_id.clone(),
                }
            })?;

            // Get preset
            let preset = route
                .presets
                .iter()
                .find(|p| p.id == route_ref.preset_id)
                .ok_or_else(|| ResolveError::PresetNotFound {
                    route_id: route_ref.route_id.clone(),
                    preset_id: route_ref.preset_id.clone(),
                })?;

            // Get variant
            let variant = preset
                .variants
                .iter()
                .find(|v| v.id == route_ref.variant_id)
                .ok_or_else(|| ResolveError::VariantNotFound {
                    route_id: route_ref.route_id.clone(),
                    preset_id: route_ref.preset_id.clone(),
                    variant_id: route_ref.variant_id.clone(),
                })?;

            // Create active route (child routes override parent routes with same route_id)
            let active_route = ActiveRoute {
                route: route.clone(),
                preset: preset.clone(),
                variant: variant.clone(),
            };

            // Child routes override parent routes
            route_map.insert(route_ref.route_id.clone(), active_route);
        }

        // Remove from visited after processing (allows reuse in different branches)
        visited.remove(collection_id);

        Ok(())
    }

    /// Collect routes in order: parent first, then child.
    ///
    /// Child routes override parent routes with the same route_id.
    fn collect_routes_in_order(
        &self,
        collection_id: &str,
        processed: &mut HashSet<String>,
        route_map: &HashMap<String, ActiveRoute>,
        result: &mut Vec<ActiveRoute>,
    ) -> Result<(), ResolveError> {
        let collection = self.collections.get(collection_id).ok_or_else(|| {
            ResolveError::CollectionNotFound {
                collection_id: collection_id.to_string(),
            }
        })?;

        // First process parent
        if let Some(parent_id) = &collection.from {
            self.collect_routes_in_order(parent_id, processed, route_map, result)?;
        }

        // Then process current collection's routes
        for route_ref_str in &collection.routes {
            let route_ref = RouteReference::parse(route_ref_str).ok_or_else(|| {
                ResolveError::InvalidRouteReference {
                    reference: route_ref_str.clone(),
                }
            })?;

            // Add route if not already processed (child routes override parent)
            if !processed.contains(&route_ref.route_id) {
                if let Some(active_route) = route_map.get(&route_ref.route_id) {
                    result.push(active_route.clone());
                    processed.insert(route_ref.route_id.clone());
                }
            }
        }

        Ok(())
    }
}

impl Default for MocksManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during collection resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// Collection not found
    CollectionNotFound { collection_id: String },
    /// Route not found
    RouteNotFound { route_id: String },
    /// Preset not found in route
    PresetNotFound { route_id: String, preset_id: String },
    /// Variant not found in preset
    VariantNotFound {
        route_id: String,
        preset_id: String,
        variant_id: String,
    },
    /// Invalid route reference format
    InvalidRouteReference { reference: String },
    /// Circular dependency detected
    CircularDependency { collection_id: String },
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::CollectionNotFound { collection_id } => {
                write!(f, "Collection not found: {}", collection_id)
            }
            ResolveError::RouteNotFound { route_id } => {
                write!(f, "Route not found: {}", route_id)
            }
            ResolveError::PresetNotFound {
                route_id,
                preset_id,
            } => {
                write!(
                    f,
                    "Preset '{}' not found in route '{}'",
                    preset_id, route_id
                )
            }
            ResolveError::VariantNotFound {
                route_id,
                preset_id,
                variant_id,
            } => {
                write!(
                    f,
                    "Variant '{}' not found in preset '{}' of route '{}'",
                    variant_id, preset_id, route_id
                )
            }
            ResolveError::InvalidRouteReference { reference } => {
                write!(f, "Invalid route reference format: {}", reference)
            }
            ResolveError::CircularDependency { collection_id } => {
                write!(
                    f,
                    "Circular dependency detected involving collection: {}",
                    collection_id
                )
            }
        }
    }
}

impl std::error::Error for ResolveError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::route::{HttpMethod, Transport};
    use rstest::rstest;

    fn create_test_route(id: &str) -> Route {
        Route {
            id: id.to_string(),
            url: format!("/api/{}", id),
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
            query_expr: None,
            headers: None,
            payload: None,
            payload_expr: None,
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
    fn test_resolve_simple_collection() {
        let mut manager = MocksManager::new();

        // Create route with preset and variant
        let mut route = create_test_route("route1");
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

        // Resolve
        let result = manager.resolve_collection("collection1").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].route.id, "route1");
        assert_eq!(result[0].preset.id, "preset1");
        assert_eq!(result[0].variant.id, "variant1");
    }

    #[rstest]
    fn test_resolve_collection_with_inheritance() {
        let mut manager = MocksManager::new();

        // Create routes
        let mut route1 = create_test_route("route1");
        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("variant1"));
        route1.presets.push(preset1);
        manager.add_route(route1);

        let mut route2 = create_test_route("route2");
        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("variant2"));
        route2.presets.push(preset2);
        manager.add_route(route2);

        // Create parent collection
        let parent = Collection {
            id: "parent".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(parent);

        // Create child collection
        let child = Collection {
            id: "child".to_string(),
            from: Some("parent".to_string()),
            routes: vec!["route2:preset2:variant2".to_string()],
        };
        manager.add_collection(child);

        // Resolve child collection
        let result = manager.resolve_collection("child").unwrap();
        assert_eq!(result.len(), 2);
        // Parent routes first
        assert_eq!(result[0].route.id, "route1");
        // Then child routes
        assert_eq!(result[1].route.id, "route2");
    }

    #[rstest]
    fn test_resolve_collection_child_overrides_parent() {
        let mut manager = MocksManager::new();

        // Create route with two presets
        let mut route = create_test_route("route1");

        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("variant1"));

        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("variant2"));

        route.presets.push(preset1);
        route.presets.push(preset2);
        manager.add_route(route);

        // Create parent collection
        let parent = Collection {
            id: "parent".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(parent);

        // Create child collection with same route but different preset
        let child = Collection {
            id: "child".to_string(),
            from: Some("parent".to_string()),
            routes: vec!["route1:preset2:variant2".to_string()],
        };
        manager.add_collection(child);

        // Resolve child collection
        let result = manager.resolve_collection("child").unwrap();
        // Child should override parent, so only one route with preset2
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].route.id, "route1");
        assert_eq!(result[0].preset.id, "preset2");
        assert_eq!(result[0].variant.id, "variant2");
    }

    #[rstest]
    fn test_resolve_collection_circular_dependency() {
        let mut manager = MocksManager::new();

        // Create circular dependency: A -> B -> A
        let collection_a = Collection {
            id: "A".to_string(),
            from: Some("B".to_string()),
            routes: vec![],
        };
        let collection_b = Collection {
            id: "B".to_string(),
            from: Some("A".to_string()),
            routes: vec![],
        };

        manager.add_collection(collection_a);
        manager.add_collection(collection_b);

        // Should detect circular dependency
        let result = manager.resolve_collection("A");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::CircularDependency { .. }
        ));
    }

    #[rstest]
    fn test_resolve_collection_not_found() {
        let manager = MocksManager::new();
        let result = manager.resolve_collection("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::CollectionNotFound { .. }
        ));
    }

    #[rstest]
    fn test_resolve_collection_route_not_found() {
        let mut manager = MocksManager::new();
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["nonexistent:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let result = manager.resolve_collection("collection1");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::RouteNotFound { .. }
        ));
    }

    #[rstest]
    fn test_resolve_collection_invalid_reference() {
        let mut manager = MocksManager::new();
        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["invalid-format".to_string()],
        };
        manager.add_collection(collection);

        let result = manager.resolve_collection("collection1");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::InvalidRouteReference { .. }
        ));
    }

    #[rstest]
    fn test_resolve_collection_multiple_levels() {
        let mut manager = MocksManager::new();

        // Create routes
        let mut route1 = create_test_route("route1");
        let mut preset1 = create_test_preset("preset1");
        preset1.variants.push(create_test_variant("variant1"));
        route1.presets.push(preset1);
        manager.add_route(route1);

        let mut route2 = create_test_route("route2");
        let mut preset2 = create_test_preset("preset2");
        preset2.variants.push(create_test_variant("variant2"));
        route2.presets.push(preset2);
        manager.add_route(route2);

        let mut route3 = create_test_route("route3");
        let mut preset3 = create_test_preset("preset3");
        preset3.variants.push(create_test_variant("variant3"));
        route3.presets.push(preset3);
        manager.add_route(route3);

        // Create grandparent
        let grandparent = Collection {
            id: "grandparent".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(grandparent);

        // Create parent
        let parent = Collection {
            id: "parent".to_string(),
            from: Some("grandparent".to_string()),
            routes: vec!["route2:preset2:variant2".to_string()],
        };
        manager.add_collection(parent);

        // Create child
        let child = Collection {
            id: "child".to_string(),
            from: Some("parent".to_string()),
            routes: vec!["route3:preset3:variant3".to_string()],
        };
        manager.add_collection(child);

        // Resolve child collection
        let result = manager.resolve_collection("child").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].route.id, "route1"); // grandparent
        assert_eq!(result[1].route.id, "route2"); // parent
        assert_eq!(result[2].route.id, "route3"); // child
    }

    #[rstest]
    fn test_add_collections() {
        let mut manager = MocksManager::new();
        let collections = vec![
            Collection {
                id: "collection1".to_string(),
                from: None,
                routes: vec![],
            },
            Collection {
                id: "collection2".to_string(),
                from: None,
                routes: vec![],
            },
        ];
        manager.add_collections(collections);
        assert_eq!(manager.collections.len(), 2);
    }

    #[rstest]
    fn test_add_routes() {
        let mut manager = MocksManager::new();
        let routes = vec![create_test_route("route1"), create_test_route("route2")];
        manager.add_routes(routes);
        assert_eq!(manager.routes.len(), 2);
    }

    #[rstest]
    fn test_resolve_collection_preset_not_found() {
        let mut manager = MocksManager::new();
        let route = create_test_route("route1");
        // Route has no presets
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let result = manager.resolve_collection("collection1");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::PresetNotFound { .. }
        ));
    }

    #[rstest]
    fn test_resolve_collection_variant_not_found() {
        let mut manager = MocksManager::new();
        let mut route = create_test_route("route1");
        let preset = create_test_preset("preset1");
        // Preset has no variants
        route.presets.push(preset);
        manager.add_route(route);

        let collection = Collection {
            id: "collection1".to_string(),
            from: None,
            routes: vec!["route1:preset1:variant1".to_string()],
        };
        manager.add_collection(collection);

        let result = manager.resolve_collection("collection1");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::VariantNotFound { .. }
        ));
    }

    #[rstest]
    fn test_resolve_error_display() {
        let error = ResolveError::CollectionNotFound {
            collection_id: "test".to_string(),
        };
        assert!(error.to_string().contains("Collection not found"));
        assert!(error.to_string().contains("test"));

        let error = ResolveError::RouteNotFound {
            route_id: "route1".to_string(),
        };
        assert!(error.to_string().contains("Route not found"));
        assert!(error.to_string().contains("route1"));

        let error = ResolveError::PresetNotFound {
            route_id: "route1".to_string(),
            preset_id: "preset1".to_string(),
        };
        assert!(error.to_string().contains("Preset"));
        assert!(error.to_string().contains("route1"));
        assert!(error.to_string().contains("preset1"));

        let error = ResolveError::VariantNotFound {
            route_id: "route1".to_string(),
            preset_id: "preset1".to_string(),
            variant_id: "variant1".to_string(),
        };
        assert!(error.to_string().contains("Variant"));
        assert!(error.to_string().contains("route1"));
        assert!(error.to_string().contains("preset1"));
        assert!(error.to_string().contains("variant1"));

        let error = ResolveError::InvalidRouteReference {
            reference: "invalid".to_string(),
        };
        assert!(error.to_string().contains("Invalid route reference"));
        assert!(error.to_string().contains("invalid"));

        let error = ResolveError::CircularDependency {
            collection_id: "A".to_string(),
        };
        assert!(error.to_string().contains("Circular dependency"));
        assert!(error.to_string().contains("A"));
    }

    #[rstest]
    fn test_mocks_manager_default() {
        let manager = MocksManager::default();
        assert_eq!(manager.collections.len(), 0);
        assert_eq!(manager.routes.len(), 0);
    }
}
