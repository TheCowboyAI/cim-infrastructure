// Copyright (c) 2025 - Cowboy AI, Inc.
//! Pure Functional Commands for ComputeResource Aggregate
//!
//! Commands express user intent and can fail validation.
//! They contain all data needed for business rule enforcement.
//!
//! # Command Pattern
//!
//! ```text
//! Command → handle_command(State, Command) → Result<Event, Error>
//! ```
//!
//! Commands differ from Events:
//! - Commands express intent (what should happen)
//! - Events express facts (what did happen)
//! - Commands can be rejected by business rules
//! - Events cannot fail (they already happened)
//!
//! # Time Handling
//!
//! All commands include explicit `timestamp` parameter.
//! **NEVER call `Utc::now()` in domain logic**.
//! Time is passed from the application layer.

use cim_domain::EntityId;
use cim_domain_location::LocationMarker;
use cim_domain_organization::Organization;
use cim_domain_person::PersonId;
use cim_domain_policy::PolicyId;
use cim_domain_spaces::ConceptId;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{Hostname, ResourceType};
use crate::events::ResourceStatus;

/// Command to register a new compute resource
///
/// This is the initial command that creates the aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterResourceCommand {
    /// Hostname for the resource
    pub hostname: Hostname,

    /// Type of resource (physical server, VM, container, etc.)
    pub resource_type: ResourceType,

    /// Timestamp when command was issued (explicit time parameter)
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,
}

/// Command to assign organization ownership
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignOrganizationCommand {
    /// Organization to assign
    pub organization_id: EntityId<Organization>,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID (event that caused this command)
    pub causation_id: Option<Uuid>,
}

/// Command to assign physical location
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignLocationCommand {
    /// Location to assign
    pub location_id: EntityId<LocationMarker>,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to assign owner/primary contact
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignOwnerCommand {
    /// Person to assign as owner
    pub owner_id: PersonId,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to add a policy to the resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddPolicyCommand {
    /// Policy to add
    pub policy_id: PolicyId,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to remove a policy from the resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovePolicyCommand {
    /// Policy to remove
    pub policy_id: PolicyId,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to assign account concept for semantic classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignAccountConceptCommand {
    /// Concept to assign
    pub concept_id: ConceptId,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to clear account concept
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearAccountConceptCommand {
    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to set hardware details
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetHardwareDetailsCommand {
    /// Hardware manufacturer
    pub manufacturer: Option<String>,

    /// Hardware model
    pub model: Option<String>,

    /// Serial number
    pub serial_number: Option<String>,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to assign asset tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignAssetTagCommand {
    /// Asset tag to assign
    pub asset_tag: String,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to update custom metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMetadataCommand {
    /// Metadata key
    pub key: String,

    /// Metadata value
    pub value: String,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

/// Command to change resource status
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeStatusCommand {
    /// New status
    pub to_status: ResourceStatus,

    /// Timestamp when command was issued
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// Optional causation ID
    pub causation_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-19T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn test_register_resource_command() {
        let cmd = RegisterResourceCommand {
            hostname: Hostname::new("server01.example.com").unwrap(),
            resource_type: ResourceType::PhysicalServer,
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
        };

        assert_eq!(cmd.hostname.as_str(), "server01.example.com");
        assert_eq!(cmd.resource_type, ResourceType::PhysicalServer);
    }

    #[test]
    fn test_assign_organization_command() {
        let org_id = EntityId::new();
        let cmd = AssignOrganizationCommand {
            organization_id: org_id.clone(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        assert_eq!(cmd.organization_id, org_id);
        assert_eq!(cmd.timestamp, test_timestamp());
    }

    #[test]
    fn test_change_status_command() {
        let cmd = ChangeStatusCommand {
            to_status: ResourceStatus::Active,
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        assert_eq!(cmd.to_status, ResourceStatus::Active);
    }
}
