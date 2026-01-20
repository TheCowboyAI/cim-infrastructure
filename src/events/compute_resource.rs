// Copyright (c) 2025 - Cowboy AI, Inc.
//! Compute Resource Domain Events
//!
//! All state changes to ComputeResource aggregates are represented as immutable events.
//! Events follow event sourcing best practices:
//! - Immutable (no setters, only getters)
//! - Past tense naming (OrganizationAssigned, not AssignOrganization)
//! - Include correlation_id and causation_id for traceability
//! - Versioned for schema evolution
//! - Serializable for persistence

use cim_domain::EntityId;
use cim_domain_location::LocationMarker;
use cim_domain_organization::Organization;
use cim_domain_person::PersonId;
use cim_domain_policy::PolicyId;
use cim_domain_spaces::ConceptId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Hostname, ResourceType};

/// Compute Resource Domain Events
///
/// All events are immutable and represent facts that have occurred.
/// Each event type corresponds to a specific state change in the ComputeResource aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComputeResourceEvent {
    /// Resource was registered/created
    ResourceRegistered(ResourceRegistered),

    /// Organization ownership was assigned
    OrganizationAssigned(OrganizationAssigned),

    /// Physical location was set
    LocationAssigned(LocationAssigned),

    /// Owner/primary contact was assigned
    OwnerAssigned(OwnerAssigned),

    /// Policy was added to resource
    PolicyAdded(PolicyAdded),

    /// Policy was removed from resource
    PolicyRemoved(PolicyRemoved),

    /// Account concept was associated
    AccountConceptAssigned(AccountConceptAssigned),

    /// Account concept association was removed
    AccountConceptCleared(AccountConceptCleared),

    /// Hardware details were set
    HardwareDetailsSet(HardwareDetailsSet),

    /// Asset tag was assigned
    AssetTagAssigned(AssetTagAssigned),

    /// Metadata entry was added or updated
    MetadataUpdated(MetadataUpdated),

    /// Resource status changed (provisioning, active, maintenance, decommissioned)
    StatusChanged(StatusChanged),
}

/// Resource was initially registered in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRegistered {
    /// Event version for schema evolution
    pub event_version: u32,

    /// Unique event identifier (UUID v7 for time ordering)
    pub event_id: Uuid,

    /// Resource aggregate ID
    pub aggregate_id: Uuid,

    /// When this event occurred
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for request tracing
    pub correlation_id: Uuid,

    /// Causation ID (event that caused this event)
    pub causation_id: Option<Uuid>,

    /// Hostname (validated DNS name)
    pub hostname: Hostname,

    /// Resource type (server, router, etc.)
    pub resource_type: ResourceType,
}

/// Organization ownership was assigned to resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationAssigned {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Organization that now owns this resource
    pub organization_id: EntityId<Organization>,
}

/// Physical location was assigned to resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationAssigned {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Physical location of resource
    pub location_id: EntityId<LocationMarker>,
}

/// Owner/primary contact was assigned
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerAssigned {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Person who owns/is responsible for this resource
    pub owner_id: PersonId,
}

/// Policy was added to resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyAdded {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Policy that was applied
    pub policy_id: PolicyId,
}

/// Policy was removed from resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRemoved {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Policy that was removed
    pub policy_id: PolicyId,
}

/// Account concept was associated with resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountConceptAssigned {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Concept ID in conceptual space
    pub concept_id: ConceptId,
}

/// Account concept association was removed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountConceptCleared {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Hardware details were set
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareDetailsSet {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Hardware manufacturer
    pub manufacturer: Option<String>,

    /// Hardware model
    pub model: Option<String>,

    /// Serial number
    pub serial_number: Option<String>,
}

/// Asset tag was assigned
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetTagAssigned {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Asset tag identifier
    pub asset_tag: String,
}

/// Metadata entry was added or updated
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataUpdated {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Metadata key
    pub key: String,

    /// Metadata value
    pub value: String,
}

/// Resource status changed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusChanged {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,

    /// Previous status
    pub from_status: ResourceStatus,

    /// New status
    pub to_status: ResourceStatus,
}

/// Resource lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    /// Resource is being provisioned
    Provisioning,

    /// Resource is active and operational
    Active,

    /// Resource is under maintenance
    Maintenance,

    /// Resource has been decommissioned
    Decommissioned,
}

impl ResourceStatus {
    /// Check if transition to another status is valid
    pub fn can_transition_to(&self, target: &ResourceStatus) -> bool {
        use ResourceStatus::*;

        // Same status is always valid (idempotent)
        if self == target {
            return true;
        }

        match (self, target) {
            // Provisioning can go to Active or Decommissioned (failed provision)
            (Provisioning, Active) => true,
            (Provisioning, Decommissioned) => true,

            // Active can go to Maintenance or Decommissioned
            (Active, Maintenance) => true,
            (Active, Decommissioned) => true,

            // Maintenance can return to Active or be Decommissioned
            (Maintenance, Active) => true,
            (Maintenance, Decommissioned) => true,

            // Decommissioned is terminal (no transitions out except to itself, handled above)
            (Decommissioned, _) => false,

            // All other transitions are invalid
            _ => false,
        }
    }
}

/// Event version constants
impl ResourceRegistered {
    pub const CURRENT_VERSION: u32 = 1;
}

impl OrganizationAssigned {
    pub const CURRENT_VERSION: u32 = 1;
}

impl LocationAssigned {
    pub const CURRENT_VERSION: u32 = 1;
}

impl OwnerAssigned {
    pub const CURRENT_VERSION: u32 = 1;
}

impl PolicyAdded {
    pub const CURRENT_VERSION: u32 = 1;
}

impl PolicyRemoved {
    pub const CURRENT_VERSION: u32 = 1;
}

impl AccountConceptAssigned {
    pub const CURRENT_VERSION: u32 = 1;
}

impl AccountConceptCleared {
    pub const CURRENT_VERSION: u32 = 1;
}

impl HardwareDetailsSet {
    pub const CURRENT_VERSION: u32 = 1;
}

impl AssetTagAssigned {
    pub const CURRENT_VERSION: u32 = 1;
}

impl MetadataUpdated {
    pub const CURRENT_VERSION: u32 = 1;
}

impl StatusChanged {
    pub const CURRENT_VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_status_transitions() {
        use ResourceStatus::*;

        // Valid transitions from Provisioning
        assert!(Provisioning.can_transition_to(&Active));
        assert!(Provisioning.can_transition_to(&Decommissioned));
        assert!(!Provisioning.can_transition_to(&Maintenance));

        // Valid transitions from Active
        assert!(Active.can_transition_to(&Maintenance));
        assert!(Active.can_transition_to(&Decommissioned));
        assert!(!Active.can_transition_to(&Provisioning));

        // Valid transitions from Maintenance
        assert!(Maintenance.can_transition_to(&Active));
        assert!(Maintenance.can_transition_to(&Decommissioned));
        assert!(!Maintenance.can_transition_to(&Provisioning));

        // Decommissioned is terminal
        assert!(!Decommissioned.can_transition_to(&Provisioning));
        assert!(!Decommissioned.can_transition_to(&Active));
        assert!(!Decommissioned.can_transition_to(&Maintenance));

        // Idempotent (same status)
        assert!(Provisioning.can_transition_to(&Provisioning));
        assert!(Active.can_transition_to(&Active));
        assert!(Maintenance.can_transition_to(&Maintenance));
        assert!(Decommissioned.can_transition_to(&Decommissioned));
    }

    #[test]
    fn test_event_serialization() {
        let event = ResourceRegistered {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: Uuid::now_v7(),
            timestamp: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            hostname: Hostname::new("test.example.com").unwrap(),
            resource_type: ResourceType::PhysicalServer,
        };

        // Should serialize to JSON
        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains("test.example.com"));

        // Should deserialize back
        let deserialized: ResourceRegistered =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.hostname.as_str(), "test.example.com");
    }
}
