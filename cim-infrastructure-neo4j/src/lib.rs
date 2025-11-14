// Copyright 2025 Cowboy AI, LLC.

//! Neo4j Graph Projection for CIM Infrastructure
//!
//! This module provides a Neo4j graph database projection for infrastructure events.
//! It transforms event-sourced infrastructure data into a queryable graph structure
//! that visualizes relationships between compute resources, networks, connections,
//! and policies.
//!
//! ## Graph Model
//!
//! The Neo4j graph uses the following node types:
//!
//! - **ComputeResource**: Physical servers, VMs, containers
//! - **Network**: Network segments with CIDR ranges
//! - **Interface**: Network interfaces on compute resources
//! - **Software**: Software artifacts and configurations
//! - **Policy**: Security and compliance policies
//!
//! Relationships include:
//!
//! - `(ComputeResource)-[:HAS_INTERFACE]->(Interface)`
//! - `(Interface)-[:CONNECTED_TO]->(Network)`
//! - `(Interface)-[:ROUTES_TO]->(Interface)` (for connections)
//! - `(ComputeResource)-[:RUNS]->(Software)`
//! - `(ComputeResource)-[:ENFORCES]->(Policy)`
//! - `(Network)-[:APPLIES]->(Policy)`
//!
//! ## Usage
//!
//! ```rust,no_run
//! use cim_infrastructure_neo4j::{Neo4jProjection, Neo4jConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Neo4jConfig {
//!         uri: "bolt://localhost:7687".to_string(),
//!         user: "".to_string(),  // No auth
//!         password: "".to_string(),
//!         database: None,
//!     };
//!
//!     let mut projection = Neo4jProjection::connect(config).await?;
//!
//!     // Initialize schema
//!     projection.initialize_schema().await?;
//!
//!     // Start event subscriber (subscribes to infrastructure.> NATS subjects)
//!     projection.start_event_subscriber("nats://localhost:4222").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod graph_model;
pub mod projection;
pub mod queries;
pub mod subscriber;

pub use config::Neo4jConfig;
pub use error::{Neo4jError, Result};
pub use graph_model::{GraphModel, NodeType, RelationshipType};
pub use projection::Neo4jProjection;
pub use queries::InfrastructureQueries;
