// Copyright 2025 Cowboy AI, LLC.

//! Infrastructure Domain Events
//!
//! All state changes in the Infrastructure domain are represented as immutable events.
//! Events follow event sourcing principles with correlation and causation tracking.

use super::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use uuid::Uuid;

// ============================================================================
// Domain Entities (Embedded in Events)
// ============================================================================

/// Compute resource entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeResource {
    pub id: ResourceId,
    pub resource_type: ComputeType,
    pub hostname: Hostname,
    pub system: SystemArchitecture,
    pub capabilities: ResourceCapabilities,
    pub interfaces: Vec<InterfaceId>,
    pub services: Vec<SoftwareId>,
    pub guests: Vec<ResourceId>,
}

/// Network interface entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub id: InterfaceId,
    pub resource_id: ResourceId,
    pub network_id: Option<NetworkId>,
    pub addresses: Vec<IpAddr>,
}

/// Network entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    pub id: NetworkId,
    pub name: String,
    pub cidr_v4: Option<Ipv4Network>,
    pub cidr_v6: Option<Ipv6Network>,
}

/// Physical connection between interfaces
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalConnection {
    pub from_resource: ResourceId,
    pub from_interface: InterfaceId,
    pub to_resource: ResourceId,
    pub to_interface: InterfaceId,
}

/// Software artifact entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareArtifact {
    pub id: SoftwareId,
    pub name: String,
    pub version: Version,
    pub derivation_path: Option<String>,
}

/// Software configuration entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareConfiguration {
    pub id: ConfigurationId,
    pub resource_id: ResourceId,
    pub software: SoftwareArtifact,
    pub configuration_data: serde_json::Value,
    pub dependencies: Vec<SoftwareId>,
}

/// Policy rule entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: PolicyId,
    pub name: String,
    pub scope: PolicyScope,
    pub rules: Vec<Rule>,
}

/// Individual rule within a policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub rule_type: RuleType,
    pub condition: String,
    pub action: String,
}

/// Type of policy rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    Security,
    Access,
    Compliance,
    Performance,
}

// ============================================================================
// Infrastructure Events
// ============================================================================

/// Domain events for Infrastructure aggregate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InfrastructureEvent {
    /// A compute resource was registered
    ComputeResourceRegistered {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        resource: ComputeResource,
    },

    /// A network interface was added to a resource
    InterfaceAdded {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        interface: NetworkInterface,
    },

    /// A network was defined
    NetworkDefined {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        network: Network,
    },

    /// Network topology was defined
    NetworkTopologyDefined {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        topology_id: TopologyId,
        networks: Vec<Network>,
        connections: Vec<PhysicalConnection>,
    },

    /// Resources were connected physically
    ResourcesConnected {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        connection: PhysicalConnection,
    },

    /// Software was configured on a resource
    SoftwareConfigured {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        configuration: SoftwareConfiguration,
    },

    /// A policy was applied
    PolicyApplied {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        policy: PolicyRule,
    },

    /// A resource was updated
    ResourceUpdated {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        resource_id: ResourceId,
        updates: ResourceUpdates,
    },

    /// A resource was removed
    ResourceRemoved {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        resource_id: ResourceId,
        reason: String,
    },
}

/// Updates that can be applied to a resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceUpdates {
    pub hostname: Option<Hostname>,
    pub capabilities: Option<ResourceCapabilities>,
    pub metadata: Option<HashMap<String, String>>,
}

impl InfrastructureEvent {
    /// Get the event ID
    pub fn event_id(&self) -> Uuid {
        match self {
            InfrastructureEvent::ComputeResourceRegistered { event_id, .. } => *event_id,
            InfrastructureEvent::InterfaceAdded { event_id, .. } => *event_id,
            InfrastructureEvent::NetworkDefined { event_id, .. } => *event_id,
            InfrastructureEvent::NetworkTopologyDefined { event_id, .. } => *event_id,
            InfrastructureEvent::ResourcesConnected { event_id, .. } => *event_id,
            InfrastructureEvent::SoftwareConfigured { event_id, .. } => *event_id,
            InfrastructureEvent::PolicyApplied { event_id, .. } => *event_id,
            InfrastructureEvent::ResourceUpdated { event_id, .. } => *event_id,
            InfrastructureEvent::ResourceRemoved { event_id, .. } => *event_id,
        }
    }

    /// Get the correlation ID
    pub fn correlation_id(&self) -> Uuid {
        match self {
            InfrastructureEvent::ComputeResourceRegistered { correlation_id, .. } => *correlation_id,
            InfrastructureEvent::InterfaceAdded { correlation_id, .. } => *correlation_id,
            InfrastructureEvent::NetworkDefined { correlation_id, .. } => *correlation_id,
            InfrastructureEvent::NetworkTopologyDefined { correlation_id, .. } => *correlation_id,
            InfrastructureEvent::ResourcesConnected { correlation_id, .. } => *correlation_id,
            InfrastructureEvent::SoftwareConfigured { correlation_id, .. } => *correlation_id,
            InfrastructureEvent::PolicyApplied { correlation_id, .. } => *correlation_id,
            InfrastructureEvent::ResourceUpdated { correlation_id, .. } => *correlation_id,
            InfrastructureEvent::ResourceRemoved { correlation_id, .. } => *correlation_id,
        }
    }

    /// Get the causation ID (if any)
    pub fn causation_id(&self) -> Option<Uuid> {
        match self {
            InfrastructureEvent::ComputeResourceRegistered { causation_id, .. } => *causation_id,
            InfrastructureEvent::InterfaceAdded { causation_id, .. } => *causation_id,
            InfrastructureEvent::NetworkDefined { causation_id, .. } => *causation_id,
            InfrastructureEvent::NetworkTopologyDefined { causation_id, .. } => *causation_id,
            InfrastructureEvent::ResourcesConnected { causation_id, .. } => *causation_id,
            InfrastructureEvent::SoftwareConfigured { causation_id, .. } => *causation_id,
            InfrastructureEvent::PolicyApplied { causation_id, .. } => *causation_id,
            InfrastructureEvent::ResourceUpdated { causation_id, .. } => *causation_id,
            InfrastructureEvent::ResourceRemoved { causation_id, .. } => *causation_id,
        }
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            InfrastructureEvent::ComputeResourceRegistered { timestamp, .. } => *timestamp,
            InfrastructureEvent::InterfaceAdded { timestamp, .. } => *timestamp,
            InfrastructureEvent::NetworkDefined { timestamp, .. } => *timestamp,
            InfrastructureEvent::NetworkTopologyDefined { timestamp, .. } => *timestamp,
            InfrastructureEvent::ResourcesConnected { timestamp, .. } => *timestamp,
            InfrastructureEvent::SoftwareConfigured { timestamp, .. } => *timestamp,
            InfrastructureEvent::PolicyApplied { timestamp, .. } => *timestamp,
            InfrastructureEvent::ResourceUpdated { timestamp, .. } => *timestamp,
            InfrastructureEvent::ResourceRemoved { timestamp, .. } => *timestamp,
        }
    }

    /// Get event type as string
    pub fn event_type(&self) -> &'static str {
        match self {
            InfrastructureEvent::ComputeResourceRegistered { .. } => "ComputeResourceRegistered",
            InfrastructureEvent::InterfaceAdded { .. } => "InterfaceAdded",
            InfrastructureEvent::NetworkDefined { .. } => "NetworkDefined",
            InfrastructureEvent::NetworkTopologyDefined { .. } => "NetworkTopologyDefined",
            InfrastructureEvent::ResourcesConnected { .. } => "ResourcesConnected",
            InfrastructureEvent::SoftwareConfigured { .. } => "SoftwareConfigured",
            InfrastructureEvent::PolicyApplied { .. } => "PolicyApplied",
            InfrastructureEvent::ResourceUpdated { .. } => "ResourceUpdated",
            InfrastructureEvent::ResourceRemoved { .. } => "ResourceRemoved",
        }
    }
}

// ============================================================================
// Event Constructors
// ============================================================================

impl InfrastructureEvent {
    /// Create a ComputeResourceRegistered event
    pub fn compute_resource_registered(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        resource: ComputeResource,
    ) -> Self {
        Self::ComputeResourceRegistered {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            resource,
        }
    }

    /// Create an InterfaceAdded event
    pub fn interface_added(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        interface: NetworkInterface,
    ) -> Self {
        Self::InterfaceAdded {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            interface,
        }
    }

    /// Create a NetworkDefined event
    pub fn network_defined(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        network: Network,
    ) -> Self {
        Self::NetworkDefined {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            network,
        }
    }

    /// Create a NetworkTopologyDefined event
    pub fn network_topology_defined(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        topology_id: TopologyId,
        networks: Vec<Network>,
        connections: Vec<PhysicalConnection>,
    ) -> Self {
        Self::NetworkTopologyDefined {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            topology_id,
            networks,
            connections,
        }
    }

    /// Create a ResourcesConnected event
    pub fn resources_connected(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        connection: PhysicalConnection,
    ) -> Self {
        Self::ResourcesConnected {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            connection,
        }
    }

    /// Create a SoftwareConfigured event
    pub fn software_configured(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        configuration: SoftwareConfiguration,
    ) -> Self {
        Self::SoftwareConfigured {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            configuration,
        }
    }

    /// Create a PolicyApplied event
    pub fn policy_applied(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        policy: PolicyRule,
    ) -> Self {
        Self::PolicyApplied {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            policy,
        }
    }

    /// Create a ResourceUpdated event
    pub fn resource_updated(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        resource_id: ResourceId,
        updates: ResourceUpdates,
    ) -> Self {
        Self::ResourceUpdated {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            resource_id,
            updates,
        }
    }

    /// Create a ResourceRemoved event
    pub fn resource_removed(
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        resource_id: ResourceId,
        reason: String,
    ) -> Self {
        Self::ResourceRemoved {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            resource_id,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let correlation_id = Uuid::now_v7();
        let resource = ComputeResource {
            id: ResourceId::new("server01").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("server01.example.com").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::new(),
            interfaces: vec![],
            services: vec![],
            guests: vec![],
        };

        let event = InfrastructureEvent::compute_resource_registered(
            correlation_id,
            None,
            resource.clone(),
        );

        assert_eq!(event.correlation_id(), correlation_id);
        assert!(event.causation_id().is_none());
        assert_eq!(event.event_type(), "ComputeResourceRegistered");

        if let InfrastructureEvent::ComputeResourceRegistered { resource: r, .. } = event {
            assert_eq!(r.id.as_str(), "server01");
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_event_id_is_v7() {
        let event = InfrastructureEvent::network_defined(
            Uuid::now_v7(),
            None,
            Network {
                id: NetworkId::new("lan").unwrap(),
                name: "LAN".into(),
                cidr_v4: Some("10.0.1.0/24".parse().unwrap()),
                cidr_v6: None,
            },
        );

        assert_eq!(event.event_id().get_version_num(), 7);
    }

    #[test]
    fn test_causation_chain() {
        let correlation_id = Uuid::now_v7();
        let event1_id = Uuid::now_v7();

        let event2 = InfrastructureEvent::interface_added(
            correlation_id,
            Some(event1_id),
            NetworkInterface {
                id: InterfaceId::new("eth0").unwrap(),
                resource_id: ResourceId::new("server01").unwrap(),
                network_id: None,
                addresses: vec![],
            },
        );

        assert_eq!(event2.causation_id(), Some(event1_id));
        assert_eq!(event2.correlation_id(), correlation_id);
    }

    #[test]
    fn test_serialization() {
        let event = InfrastructureEvent::policy_applied(
            Uuid::now_v7(),
            None,
            PolicyRule {
                id: PolicyId::new(),
                name: "Security Policy".into(),
                scope: PolicyScope::Global,
                rules: vec![],
            },
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: InfrastructureEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_id(), deserialized.event_id());
    }
}
