//! NAPI bindings for controller utilities.

use crate::config::{Preset, Route, Variant};
use crate::mocks::manager::ActiveRoute;
use mockito_core::config::parser;
use mockito_core::mocks::{
    controller::MocksController as CoreMocksController, manager::MocksManager,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{Arc, Mutex};

#[napi]
pub struct MocksController {
    inner: Arc<Mutex<CoreMocksController>>,
}

#[napi]
impl MocksController {
    /// Create a new controller manager
    ///
    /// @param collectionsPath - Path or glob pattern to collections file(s)
    /// @param routesPath - Path or glob pattern to routes file(s)
    /// @param defaultCollection - Optional default collection ID
    #[napi(constructor)]
    pub fn new(
        collections_path: String,
        routes_path: String,
        default_collection: Option<String>,
    ) -> Result<Self> {
        // Load routes and collections
        let routes = parser::load_routes(&routes_path)
            .map_err(|e| Error::from_reason(format!("Failed to load routes: {e}")))?;
        let collections = parser::load_collections(&collections_path)
            .map_err(|e| Error::from_reason(format!("Failed to load collections: {e}")))?;

        // Create manager and add data
        let mut manager = MocksManager::new();
        manager.add_routes(routes);
        manager.add_collections(collections);

        // Create controller
        let controller = CoreMocksController::new(manager);

        // Activate default collection if provided
        let result = Self {
            inner: Arc::new(Mutex::new(controller)),
        };

        if let Some(collection_id) = default_collection {
            result.use_collection(collection_id)?;
        }

        Ok(result)
    }

    /// Apply a collection by ID
    #[napi]
    pub fn use_collection(&self, collection_id: String) -> Result<()> {
        let mut controller = self.inner.lock().unwrap();
        controller
            .use_collection(&collection_id)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Get current collection ID
    #[napi(getter)]
    pub fn current_collection(&self) -> Option<String> {
        let controller = self.inner.lock().unwrap();
        controller.active_collection_id().map(String::from)
    }

    /// Get all active routes (HTTP + WS)
    #[napi]
    pub fn get_active_routes(&self) -> Vec<ActiveRoute> {
        let controller = self.inner.lock().unwrap();
        controller
            .get_active_routes()
            .iter()
            .map(|a| ActiveRoute {
                route: Route::from(&a.route),
                preset: Preset::from(&a.preset),
                variant: Variant::from(&a.variant),
            })
            .collect()
    }
}
