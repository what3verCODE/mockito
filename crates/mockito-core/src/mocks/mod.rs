//! Mocks management module.
//!
//! This module provides functionality for managing mock routes and collections:
//! - [`MocksManager`]: Stores and resolves collections and routes with inheritance support
//! - [`MocksController`]: Manages active routes and provides fast route lookup by request matching

pub mod controller;
pub mod manager;
