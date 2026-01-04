//! NAPI bindings for mocks operations.

use crate::config::{Preset, Route, Variant};
use mockito_core::config::parser;
use mockito_core::mocks::manager::{
    ActiveRoute as CoreActiveRoute, MocksManager as CoreMocksManager,
};
use mockito_core::types::{
    preset::Preset as CorePreset, route::Route as CoreRoute, variant::Variant as CoreVariant,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{Arc, Mutex};

#[napi(object)]
pub struct ActiveRoute {
    pub route: Route,
    pub preset: Preset,
    pub variant: Variant,
}

impl From<CoreActiveRoute> for ActiveRoute {
    fn from(a: CoreActiveRoute) -> Self {
        Self {
            route: Route::from(a.route),
            preset: Preset::from(a.preset),
            variant: Variant::from(a.variant),
        }
    }
}

impl From<&ActiveRoute> for CoreActiveRoute {
    fn from(a: &ActiveRoute) -> Self {
        Self {
            route: CoreRoute::from(&a.route),
            preset: CorePreset::from(&a.preset),
            variant: CoreVariant::from(&a.variant),
        }
    }
}

impl From<ActiveRoute> for CoreActiveRoute {
    fn from(a: ActiveRoute) -> Self {
        Self {
            route: CoreRoute::from(a.route),
            preset: CorePreset::from(a.preset),
            variant: CoreVariant::from(a.variant),
        }
    }
}

/// Mocks Manager class
#[napi]
pub struct MocksManager {
    inner: Arc<Mutex<CoreMocksManager>>,
}

#[napi]
impl MocksManager {
    /// Create a new mocks manager
    ///
    /// @param collectionsPath - Path or glob pattern to collections file(s)
    /// @param routesPath - Path or glob pattern to routes file(s)
    /// @param basePath - Optional base directory for resolving relative paths
    #[napi(constructor)]
    pub fn new(collections_path: String, routes_path: String) -> Result<Self> {
        // Load routes and collections
        let routes = parser::load_routes(&routes_path)
            .map_err(|e| Error::from_reason(format!("Failed to load routes: {e}")))?;
        let collections = parser::load_collections(&collections_path)
            .map_err(|e| Error::from_reason(format!("Failed to load collections: {e}")))?;

        // Create manager and add data
        let mut manager = CoreMocksManager::new();
        manager.add_routes(routes);
        manager.add_collections(collections);

        Ok(Self {
            inner: Arc::new(Mutex::new(manager)),
        })
    }

    /// Resolve collection with inheritance and return active routes
    #[napi]
    pub fn resolve_collection(&self, collection_id: String) -> Result<Vec<ActiveRoute>> {
        let manager = self.inner.lock().unwrap();
        let active_routes = manager
            .resolve_collection(&collection_id)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(active_routes
            .into_iter()
            .map(|a| ActiveRoute {
                route: Route::from(&a.route),
                preset: Preset::from(&a.preset),
                variant: Variant::from(&a.variant),
            })
            .collect())
    }
}
