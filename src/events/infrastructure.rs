// Copyright (c) 2025 - Cowboy AI, Inc.
//! Infrastructure Domain Events
//!
//! Top-level event envelope for all infrastructure-related events.
//! This allows polymorphic handling of different aggregate types while
//! maintaining type safety.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::compute_resource::ComputeResourceEvent;

/// Infrastructure Domain Events
///
/// Polymorphic envelope for all infrastructure aggregate events.
/// Each variant represents events from a specific aggregate type.
///
/// # Design Rationale
/// - Allows NATS consumers to handle any infrastructure event
/// - Maintains type safety (each variant is strongly typed)
/// - Supports future aggregate types (Network, Storage, etc.)
/// - Enables polymorphic projections
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "aggregate_type", content = "event", rename_all = "snake_case")]
pub enum InfrastructureEvent {
    /// Events from ComputeResource aggregate
    ComputeResource(ComputeResourceEvent),

    // Future aggregate types:
    // Network(NetworkEvent) - routers, switches, VLANs
    // Storage(StorageEvent) - volumes, arrays, snapshots
    // Container(ContainerEvent) - pods, deployments, services
}

impl InfrastructureEvent {
    /// Extract aggregate ID from any event type
    pub fn aggregate_id(&self) -> Uuid {
        match self {
            InfrastructureEvent::ComputeResource(event) => event.aggregate_id(),
        }
    }

    /// Extract event timestamp from any event type
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            InfrastructureEvent::ComputeResource(event) => event.timestamp(),
        }
    }

    /// Extract correlation ID from any event type
    pub fn correlation_id(&self) -> Uuid {
        match self {
            InfrastructureEvent::ComputeResource(event) => event.correlation_id(),
        }
    }

    /// Extract causation ID from any event type
    pub fn causation_id(&self) -> Option<Uuid> {
        match self {
            InfrastructureEvent::ComputeResource(event) => event.causation_id(),
        }
    }

    /// Extract event version from any event type
    pub fn event_version(&self) -> u32 {
        match self {
            InfrastructureEvent::ComputeResource(event) => event.event_version(),
        }
    }

    /// Get human-readable event type name
    pub fn event_type_name(&self) -> &str {
        match self {
            InfrastructureEvent::ComputeResource(event) => event.event_type_name(),
        }
    }
}

impl ComputeResourceEvent {
    /// Extract aggregate ID from compute resource event
    pub fn aggregate_id(&self) -> Uuid {
        use super::compute_resource::ComputeResourceEvent::*;

        match self {
            ResourceRegistered(e) => e.aggregate_id,
            OrganizationAssigned(e) => e.aggregate_id,
            LocationAssigned(e) => e.aggregate_id,
            OwnerAssigned(e) => e.aggregate_id,
            PolicyAdded(e) => e.aggregate_id,
            PolicyRemoved(e) => e.aggregate_id,
            AccountConceptAssigned(e) => e.aggregate_id,
            AccountConceptCleared(e) => e.aggregate_id,
            HardwareDetailsSet(e) => e.aggregate_id,
            AssetTagAssigned(e) => e.aggregate_id,
            MetadataUpdated(e) => e.aggregate_id,
            StatusChanged(e) => e.aggregate_id,
        }
    }

    /// Extract timestamp from compute resource event
    pub fn timestamp(&self) -> DateTime<Utc> {
        use super::compute_resource::ComputeResourceEvent::*;

        match self {
            ResourceRegistered(e) => e.timestamp,
            OrganizationAssigned(e) => e.timestamp,
            LocationAssigned(e) => e.timestamp,
            OwnerAssigned(e) => e.timestamp,
            PolicyAdded(e) => e.timestamp,
            PolicyRemoved(e) => e.timestamp,
            AccountConceptAssigned(e) => e.timestamp,
            AccountConceptCleared(e) => e.timestamp,
            HardwareDetailsSet(e) => e.timestamp,
            AssetTagAssigned(e) => e.timestamp,
            MetadataUpdated(e) => e.timestamp,
            StatusChanged(e) => e.timestamp,
        }
    }

    /// Extract correlation ID from compute resource event
    pub fn correlation_id(&self) -> Uuid {
        use super::compute_resource::ComputeResourceEvent::*;

        match self {
            ResourceRegistered(e) => e.correlation_id,
            OrganizationAssigned(e) => e.correlation_id,
            LocationAssigned(e) => e.correlation_id,
            OwnerAssigned(e) => e.correlation_id,
            PolicyAdded(e) => e.correlation_id,
            PolicyRemoved(e) => e.correlation_id,
            AccountConceptAssigned(e) => e.correlation_id,
            AccountConceptCleared(e) => e.correlation_id,
            HardwareDetailsSet(e) => e.correlation_id,
            AssetTagAssigned(e) => e.correlation_id,
            MetadataUpdated(e) => e.correlation_id,
            StatusChanged(e) => e.correlation_id,
        }
    }

    /// Extract causation ID from compute resource event
    pub fn causation_id(&self) -> Option<Uuid> {
        use super::compute_resource::ComputeResourceEvent::*;

        match self {
            ResourceRegistered(e) => e.causation_id,
            OrganizationAssigned(e) => e.causation_id,
            LocationAssigned(e) => e.causation_id,
            OwnerAssigned(e) => e.causation_id,
            PolicyAdded(e) => e.causation_id,
            PolicyRemoved(e) => e.causation_id,
            AccountConceptAssigned(e) => e.causation_id,
            AccountConceptCleared(e) => e.causation_id,
            HardwareDetailsSet(e) => e.causation_id,
            AssetTagAssigned(e) => e.causation_id,
            MetadataUpdated(e) => e.causation_id,
            StatusChanged(e) => e.causation_id,
        }
    }

    /// Extract event version from compute resource event
    pub fn event_version(&self) -> u32 {
        use super::compute_resource::ComputeResourceEvent::*;

        match self {
            ResourceRegistered(e) => e.event_version,
            OrganizationAssigned(e) => e.event_version,
            LocationAssigned(e) => e.event_version,
            OwnerAssigned(e) => e.event_version,
            PolicyAdded(e) => e.event_version,
            PolicyRemoved(e) => e.event_version,
            AccountConceptAssigned(e) => e.event_version,
            AccountConceptCleared(e) => e.event_version,
            HardwareDetailsSet(e) => e.event_version,
            AssetTagAssigned(e) => e.event_version,
            MetadataUpdated(e) => e.event_version,
            StatusChanged(e) => e.event_version,
        }
    }

    /// Get human-readable event type name
    pub fn event_type_name(&self) -> &str {
        use super::compute_resource::ComputeResourceEvent::*;

        match self {
            ResourceRegistered(_) => "ResourceRegistered",
            OrganizationAssigned(_) => "OrganizationAssigned",
            LocationAssigned(_) => "LocationAssigned",
            OwnerAssigned(_) => "OwnerAssigned",
            PolicyAdded(_) => "PolicyAdded",
            PolicyRemoved(_) => "PolicyRemoved",
            AccountConceptAssigned(_) => "AccountConceptAssigned",
            AccountConceptCleared(_) => "AccountConceptCleared",
            HardwareDetailsSet(_) => "HardwareDetailsSet",
            AssetTagAssigned(_) => "AssetTagAssigned",
            MetadataUpdated(_) => "MetadataUpdated",
            StatusChanged(_) => "StatusChanged",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Hostname, ResourceType};
    use crate::events::compute_resource::ResourceRegistered;

    #[test]
    fn test_infrastructure_event_polymorphism() {
        let compute_event = ComputeResourceEvent::ResourceRegistered(ResourceRegistered {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: Uuid::now_v7(),
            timestamp: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            hostname: Hostname::new("test.example.com").unwrap(),
            resource_type: ResourceType::PhysicalServer,
        });

        let infra_event = InfrastructureEvent::ComputeResource(compute_event.clone());

        // Should extract metadata correctly
        assert_eq!(infra_event.aggregate_id(), compute_event.aggregate_id());
        assert_eq!(infra_event.correlation_id(), compute_event.correlation_id());
        assert_eq!(infra_event.event_version(), 1);
        assert_eq!(infra_event.event_type_name(), "ResourceRegistered");
    }

    #[test]
    fn test_infrastructure_event_serialization() {
        let compute_event = ComputeResourceEvent::ResourceRegistered(ResourceRegistered {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: Uuid::now_v7(),
            timestamp: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            hostname: Hostname::new("server01.example.com").unwrap(),
            resource_type: ResourceType::VirtualMachine,
        });

        let infra_event = InfrastructureEvent::ComputeResource(compute_event);

        // Should serialize
        let json = serde_json::to_string(&infra_event).expect("Failed to serialize");
        assert!(json.contains("server01.example.com"));
        assert!(json.contains("compute_resource")); // aggregate_type tag

        // Should deserialize
        let deserialized: InfrastructureEvent =
            serde_json::from_str(&json).expect("Failed to deserialize");

        match deserialized {
            InfrastructureEvent::ComputeResource(ComputeResourceEvent::ResourceRegistered(e)) => {
                assert_eq!(e.hostname.as_str(), "server01.example.com");
            }
            _ => panic!("Wrong event type after deserialization"),
        }
    }
}
