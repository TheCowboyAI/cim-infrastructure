// Copyright (c) 2025 - Cowboy AI, Inc.

//! Neo4j Projection Adapter
//!
//! Implements the ProjectionAdapter Functor for projecting infrastructure events
//! into a Neo4j graph database.
//!
//! # Graph Model
//!
//! The Neo4j projection represents infrastructure as a graph with:
//!
//! ## Nodes
//! - **ComputeResource**: Servers, VMs, containers
//! - **Network**: Network segments with CIDR ranges
//! - **Interface**: Network interfaces on compute resources
//! - **Software**: Software artifacts and configurations
//! - **Policy**: Security and compliance policies
//!
//! ## Relationships
//! - `(ComputeResource)-[:HAS_INTERFACE]->(Interface)`
//! - `(Interface)-[:CONNECTED_TO]->(Network)`
//! - `(Interface)-[:ROUTES_TO]->(Interface)` (for physical connections)
//! - `(ComputeResource)-[:RUNS]->(Software)`
//! - `(ComputeResource)-[:ENFORCES]->(Policy)`
//! - `(Network)-[:APPLIES]->(Policy)`
//!
//! # Functoriality
//!
//! This projection is a structure-preserving functor:
//!
//! ```text
//! F: InfrastructureEvents → Neo4jGraph
//!
//! F(ComputeRegistered) = CREATE (r:ComputeResource {...})
//! F(NetworkDefined) = CREATE (n:Network {...})
//! F(ConnectionEstablished) = CREATE (i1)-[:ROUTES_TO]->(i2)
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use cim_infrastructure::adapters::Neo4jProjectionAdapter;
//! use cim_infrastructure::projection::ProjectionAdapter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Neo4jConfig {
//!         uri: "bolt://localhost:7687".to_string(),
//!         username: "neo4j".to_string(),
//!         password: "password".to_string(),
//!         database: None,
//!     };
//!
//!     let mut projection = Neo4jProjectionAdapter::new(config).await?;
//!     projection.initialize().await?;
//!
//!     // Project events...
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use neo4rs::{Graph, Query};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::projection::{ProjectionAdapter, ProjectionError};

/// Configuration for Neo4j connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    /// Neo4j URI (e.g., "bolt://localhost:7687")
    pub uri: String,

    /// Username for authentication
    pub username: String,

    /// Password for authentication
    pub password: String,

    /// Optional database name (uses default if None)
    pub database: Option<String>,
}

impl Default for Neo4jConfig {
    fn default() -> Self {
        Self {
            uri: "bolt://localhost:7687".to_string(),
            username: "neo4j".to_string(),
            password: "password".to_string(),
            database: None,
        }
    }
}

/// Infrastructure event type for projection
///
/// This is a simplified event envelope for demonstration.
/// In production, use domain-specific event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureEvent {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Neo4j projection adapter implementing the Functor F: Events → Neo4jGraph
pub struct Neo4jProjectionAdapter {
    graph: Arc<Graph>,
    config: Neo4jConfig,
}

impl Neo4jProjectionAdapter {
    /// Create a new Neo4j projection adapter
    pub async fn new(config: Neo4jConfig) -> Result<Self, ProjectionError> {
        info!("Connecting to Neo4j at {}", config.uri);

        let graph = Graph::new(&config.uri, &config.username, &config.password)
            .await
            .map_err(|e| {
                ProjectionError::TargetUnavailable(format!(
                    "Failed to connect to Neo4j: {}",
                    e
                ))
            })?;

        Ok(Self {
            graph: Arc::new(graph),
            config,
        })
    }

    /// Project a compute resource registered event
    async fn project_compute_registered(
        &self,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        let id = data["id"].as_str().ok_or_else(|| {
            ProjectionError::InvalidEvent("Missing 'id' field in ComputeRegistered event".to_string())
        })?;

        let hostname = data["hostname"].as_str().unwrap_or("unknown");
        let resource_type = data["resource_type"].as_str().unwrap_or("unknown");

        let query = Query::new(
            r#"
            MERGE (r:ComputeResource {id: $id})
            SET r.hostname = $hostname,
                r.resource_type = $resource_type,
                r.updated_at = timestamp()
            RETURN r
            "#.to_string(),
        )
        .param("id", id)
        .param("hostname", hostname)
        .param("resource_type", resource_type);

        self.graph
            .run(query)
            .await
            .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;

        debug!("Projected ComputeRegistered for {}", id);
        Ok(())
    }

    /// Project a network defined event
    async fn project_network_defined(
        &self,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        let id = data["id"].as_str().ok_or_else(|| {
            ProjectionError::InvalidEvent("Missing 'id' field in NetworkDefined event".to_string())
        })?;

        let name = data["name"].as_str().unwrap_or("unknown");
        let cidr = data["cidr"].as_str();

        let mut query = Query::new(
            r#"
            MERGE (n:Network {id: $id})
            SET n.name = $name,
                n.updated_at = timestamp()
            "#.to_string(),
        )
        .param("id", id)
        .param("name", name);

        if let Some(cidr_value) = cidr {
            query = query.param("cidr", cidr_value);
        }

        self.graph
            .run(query)
            .await
            .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;

        debug!("Projected NetworkDefined for {}", id);
        Ok(())
    }

    /// Project a connection established event
    async fn project_connection_established(
        &self,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        let from_interface = data["from_interface"].as_str().ok_or_else(|| {
            ProjectionError::InvalidEvent(
                "Missing 'from_interface' in ConnectionEstablished event".to_string(),
            )
        })?;

        let to_interface = data["to_interface"].as_str().ok_or_else(|| {
            ProjectionError::InvalidEvent(
                "Missing 'to_interface' in ConnectionEstablished event".to_string(),
            )
        })?;

        let query = Query::new(
            r#"
            MATCH (i1:Interface {id: $from_id})
            MATCH (i2:Interface {id: $to_id})
            MERGE (i1)-[:ROUTES_TO {established_at: timestamp()}]->(i2)
            "#.to_string(),
        )
        .param("from_id", from_interface)
        .param("to_id", to_interface);

        self.graph
            .run(query)
            .await
            .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;

        debug!(
            "Projected ConnectionEstablished: {} -> {}",
            from_interface, to_interface
        );
        Ok(())
    }
}

#[async_trait]
impl ProjectionAdapter for Neo4jProjectionAdapter {
    type Event = InfrastructureEvent;
    type Error = ProjectionError;

    async fn project(&mut self, event: Self::Event) -> Result<(), Self::Error> {
        debug!("Projecting event: {} ({})", event.event_type, event.event_id);

        // Route events to specific projection handlers based on event type
        match event.event_type.as_str() {
            "ComputeRegistered" | "compute.registered" => {
                self.project_compute_registered(&event.data).await?
            }
            "NetworkDefined" | "network.defined" => {
                self.project_network_defined(&event.data).await?
            }
            "ConnectionEstablished" | "connection.established" => {
                self.project_connection_established(&event.data).await?
            }
            unknown => {
                warn!("Unknown event type: {}", unknown);
                // Don't fail on unknown events - allows for graceful evolution
            }
        }

        Ok(())
    }

    async fn initialize(&mut self) -> Result<(), Self::Error> {
        info!("Initializing Neo4j schema for infrastructure projection");

        // Create uniqueness constraints
        let constraints = vec![
            "CREATE CONSTRAINT compute_resource_id IF NOT EXISTS FOR (r:ComputeResource) REQUIRE r.id IS UNIQUE",
            "CREATE CONSTRAINT network_id IF NOT EXISTS FOR (n:Network) REQUIRE n.id IS UNIQUE",
            "CREATE CONSTRAINT interface_id IF NOT EXISTS FOR (i:Interface) REQUIRE i.id IS UNIQUE",
            "CREATE CONSTRAINT software_id IF NOT EXISTS FOR (s:Software) REQUIRE s.id IS UNIQUE",
            "CREATE CONSTRAINT policy_id IF NOT EXISTS FOR (p:Policy) REQUIRE p.id IS UNIQUE",
        ];

        for constraint in constraints {
            self.graph
                .run(Query::new(constraint.to_string()))
                .await
                .map_err(|e| ProjectionError::InitializationFailed(e.to_string()))?;
        }

        // Create indexes for common queries
        let indexes = vec![
            "CREATE INDEX compute_hostname IF NOT EXISTS FOR (r:ComputeResource) ON (r.hostname)",
            "CREATE INDEX network_name IF NOT EXISTS FOR (n:Network) ON (n.name)",
            "CREATE INDEX network_cidr IF NOT EXISTS FOR (n:Network) ON (n.cidr)",
        ];

        for index in indexes {
            self.graph
                .run(Query::new(index.to_string()))
                .await
                .map_err(|e| ProjectionError::InitializationFailed(e.to_string()))?;
        }

        info!("Neo4j schema initialization complete");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Self::Error> {
        // Simple connectivity check
        self.graph
            .run(Query::new("RETURN 1".to_string()))
            .await
            .map_err(|e| {
                ProjectionError::TargetUnavailable(format!(
                    "Neo4j health check failed: {}",
                    e
                ))
            })?;

        debug!("Neo4j health check passed");
        Ok(())
    }

    async fn reset(&mut self) -> Result<(), Self::Error> {
        warn!("Resetting Neo4j projection - ALL DATA WILL BE DELETED");

        // Delete all nodes and relationships
        self.graph
            .run(Query::new("MATCH (n) DETACH DELETE n".to_string()))
            .await
            .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;

        info!("Neo4j projection reset complete");
        Ok(())
    }

    fn name(&self) -> &str {
        "neo4j-infrastructure-projection"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Neo4jConfig::default();
        assert_eq!(config.uri, "bolt://localhost:7687");
        assert_eq!(config.username, "neo4j");
    }

    #[test]
    fn test_infrastructure_event_creation() {
        let event = InfrastructureEvent {
            event_id: Uuid::now_v7(),
            aggregate_id: Uuid::now_v7(),
            event_type: "ComputeRegistered".to_string(),
            data: serde_json::json!({
                "id": "server-1",
                "hostname": "web01.example.com",
                "resource_type": "physical_server"
            }),
        };

        assert_eq!(event.event_type, "ComputeRegistered");
        assert!(event.data["hostname"].is_string());
    }
}
