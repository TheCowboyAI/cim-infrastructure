//! Event projections for building read models
//!
//! Provides infrastructure for projecting event streams into queryable read models:
//! - Aggregate-specific projections
//! - Multi-aggregate views
//! - NATS KV-backed persistence
//! - Incremental updates

use async_nats::jetstream::kv::{Store as KvStore, Config as KvConfig};
use async_nats::jetstream::Context as JetStreamContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

use cim_domain_infrastructure::{
    InfrastructureEvent, ResourceId, NetworkId,
};

use crate::event_store::StoredEvent;

/// Error types for projection operations
#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("JetStream error: {0}")]
    JetStream(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Projection error: {0}")]
    Projection(String),

    #[error("KV store error: {0}")]
    KvStore(String),
}

/// Result type for projection operations
pub type Result<T> = std::result::Result<T, ProjectionError>;

/// Read model for compute resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResourceView {
    pub id: ResourceId,
    pub hostname: String,
    pub system: String,
    pub resource_type: String,
    pub is_active: bool,
    pub last_updated: String,
}

/// Read model for networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkView {
    pub id: NetworkId,
    pub name: String,
    pub network_type: String,
    pub is_active: bool,
    pub last_updated: String,
}

/// Read model for connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionView {
    pub id: String,
    pub source_id: ResourceId,
    pub target_id: ResourceId,
    pub connection_type: String,
    pub is_active: bool,
    pub last_updated: String,
}

/// Infrastructure topology projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyView {
    pub compute_resources: HashMap<String, ComputeResourceView>,
    pub networks: HashMap<String, NetworkView>,
    pub connections: HashMap<String, ConnectionView>,
    pub last_event_sequence: u64,
}

impl TopologyView {
    pub fn new() -> Self {
        Self {
            compute_resources: HashMap::new(),
            networks: HashMap::new(),
            connections: HashMap::new(),
            last_event_sequence: 0,
        }
    }

    /// Project an event onto the topology view
    pub fn project(&mut self, event: &StoredEvent) -> Result<()> {
        use cim_domain_infrastructure::InfrastructureEvent::*;

        let domain_event: InfrastructureEvent = serde_json::from_value(event.data.clone())?;

        match domain_event {
            ComputeResourceRegistered { resource, .. } => {
                let view = ComputeResourceView {
                    id: resource.id.clone(),
                    hostname: resource.hostname.to_string(),
                    system: resource.system.to_string(),
                    resource_type: format!("{:?}", resource.resource_type),
                    is_active: true,
                    last_updated: event.timestamp.to_rfc3339(),
                };
                self.compute_resources.insert(resource.id.to_string(), view);
            }

            ResourceRemoved { resource_id, .. } => {
                if let Some(resource) = self.compute_resources.get_mut(&resource_id.to_string()) {
                    resource.is_active = false;
                    resource.last_updated = event.timestamp.to_rfc3339();
                }
            }

            NetworkDefined { network, .. } => {
                let network_type = if network.cidr_v4.is_some() && network.cidr_v6.is_some() {
                    "DualStack"
                } else if network.cidr_v4.is_some() {
                    "IPv4"
                } else if network.cidr_v6.is_some() {
                    "IPv6"
                } else {
                    "Unknown"
                };

                let view = NetworkView {
                    id: network.id.clone(),
                    name: network.name.to_string(),
                    network_type: network_type.to_string(),
                    is_active: true,
                    last_updated: event.timestamp.to_rfc3339(),
                };
                self.networks.insert(network.id.to_string(), view);
            }

            ResourcesConnected { connection, .. } => {
                // Create a synthetic connection ID from connection fields
                let conn_id = format!("{}:{}--{}:{}",
                    connection.from_resource,
                    connection.from_interface,
                    connection.to_resource,
                    connection.to_interface
                );
                let view = ConnectionView {
                    id: conn_id.clone(),
                    source_id: connection.from_resource.clone(),
                    target_id: connection.to_resource.clone(),
                    connection_type: "Physical".to_string(),
                    is_active: true,
                    last_updated: event.timestamp.to_rfc3339(),
                };
                self.connections.insert(conn_id, view);
            }

            _ => {
                // Other events don't affect the topology view
                debug!(event_type = %event.event_type, "Event not relevant for topology projection");
            }
        }

        self.last_event_sequence = event.sequence;
        Ok(())
    }

    /// Get all active compute resources
    pub fn active_compute_resources(&self) -> Vec<&ComputeResourceView> {
        self.compute_resources
            .values()
            .filter(|r| r.is_active)
            .collect()
    }

    /// Get all active networks
    pub fn active_networks(&self) -> Vec<&NetworkView> {
        self.networks.values().filter(|n| n.is_active).collect()
    }

    /// Get all active connections
    pub fn active_connections(&self) -> Vec<&ConnectionView> {
        self.connections.values().filter(|c| c.is_active).collect()
    }
}

impl Default for TopologyView {
    fn default() -> Self {
        Self::new()
    }
}

/// Projection manager with KV store persistence
pub struct ProjectionManager {
    kv_store: Option<KvStore>,
    topology: Arc<RwLock<TopologyView>>,
}

impl ProjectionManager {
    /// Create a new projection manager
    pub fn new() -> Self {
        Self {
            kv_store: None,
            topology: Arc::new(RwLock::new(TopologyView::new())),
        }
    }

    /// Initialize with KV store for persistence
    pub async fn with_kv_store(
        jetstream: JetStreamContext,
        bucket_name: &str,
    ) -> Result<Self> {
        let kv_store = jetstream
            .create_key_value(KvConfig {
                bucket: bucket_name.to_string(),
                ..Default::default()
            })
            .await
            .map_err(|e| ProjectionError::JetStream(e.to_string()))?;

        let mut manager = Self::new();
        manager.kv_store = Some(kv_store);

        // Load existing state if available
        manager.load_state().await?;

        Ok(manager)
    }

    /// Project an event onto all projections
    pub async fn project(&self, event: &StoredEvent) -> Result<()> {
        let mut topology = self.topology.write().await;
        topology.project(event)?;

        // Persist to KV store if available
        if let Some(ref kv) = self.kv_store {
            self.save_state_to_kv(kv, &topology).await?;
        }

        info!(
            event_type = %event.event_type,
            sequence = event.sequence,
            "Event projected successfully"
        );

        Ok(())
    }

    /// Get the current topology view
    pub async fn get_topology(&self) -> TopologyView {
        self.topology.read().await.clone()
    }

    /// Load state from KV store
    async fn load_state(&mut self) -> Result<()> {
        if let Some(ref kv) = self.kv_store {
            match kv.get("topology").await {
                Ok(Some(entry)) => {
                    let topology: TopologyView = serde_json::from_slice(&entry)?;
                    *self.topology.write().await = topology;
                    info!("Loaded projection state from KV store");
                }
                Ok(None) => {
                    debug!("No existing projection state in KV store");
                }
                Err(e) => {
                    return Err(ProjectionError::KvStore(e.to_string()));
                }
            }
        }
        Ok(())
    }

    /// Save state to KV store
    async fn save_state_to_kv(&self, kv: &KvStore, topology: &TopologyView) -> Result<()> {
        let data = serde_json::to_vec(topology)?;
        kv.put("topology", data.into())
            .await
            .map_err(|e| ProjectionError::KvStore(e.to_string()))?;
        Ok(())
    }

    /// Rebuild projections from event stream
    pub async fn rebuild(&self, events: Vec<StoredEvent>) -> Result<()> {
        info!(event_count = events.len(), "Rebuilding projections");

        let mut topology = self.topology.write().await;
        *topology = TopologyView::new();

        for event in events {
            topology.project(&event)?;
        }

        // Persist final state
        if let Some(ref kv) = self.kv_store {
            self.save_state_to_kv(kv, &topology).await?;
        }

        info!("Projection rebuild complete");
        Ok(())
    }
}

impl Default for ProjectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use cim_domain_infrastructure::{
        Hostname, SystemArchitecture, ResourceCapabilities,
    };

    #[test]
    fn test_topology_view_compute_projection() {
        let mut topology = TopologyView::new();

        // Create actual domain event
        use cim_domain_infrastructure::{InfrastructureEvent, ComputeResource, ComputeType};

        let resource = ComputeResource {
            id: ResourceId::new("test-host").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("test-host").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::default(),
            interfaces: vec![],
            guests: vec![],
            services: vec![],
        };

        let domain_event = InfrastructureEvent::ComputeResourceRegistered {
            event_id: Uuid::now_v7(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            timestamp: chrono::Utc::now(),
            resource,
        };

        let event = StoredEvent {
            event_id: Uuid::now_v7(),
            aggregate_id: Uuid::now_v7(),
            sequence: 1,
            timestamp: chrono::Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            event_type: "ComputeResourceRegistered".to_string(),
            data: serde_json::to_value(&domain_event).unwrap(),
            metadata: None,
        };

        topology.project(&event).unwrap();

        assert_eq!(topology.compute_resources.len(), 1);
        assert_eq!(topology.last_event_sequence, 1);
    }

    #[tokio::test]
    async fn test_projection_manager() {
        let manager = ProjectionManager::new();

        let topology = manager.get_topology().await;
        assert_eq!(topology.compute_resources.len(), 0);
    }
}
