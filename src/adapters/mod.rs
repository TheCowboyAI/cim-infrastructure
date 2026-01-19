// Copyright (c) 2025 - Cowboy AI, Inc.

//! Projection adapter implementations
//!
//! This module contains concrete implementations of the ProjectionAdapter trait
//! for various target databases and systems.

#[cfg(feature = "neo4j")]
pub mod neo4j;

#[cfg(feature = "neo4j")]
pub use neo4j::Neo4jProjectionAdapter;

#[cfg(feature = "netbox")]
pub mod netbox;

#[cfg(feature = "netbox")]
pub use netbox::{InfrastructureEvent, NetBoxConfig, NetBoxProjectionAdapter};
