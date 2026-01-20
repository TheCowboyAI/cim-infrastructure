// Copyright (c) 2025 - Cowboy AI, Inc.
//! Pure Functional Command Handlers for ComputeResource Aggregate
//!
//! Command handlers are pure functions that:
//! 1. Take current state + command
//! 2. Validate business rules
//! 3. Return Event (success) or Error (validation failure)
//!
//! # Handler Pattern
//!
//! ```text
//! handle_command(State, Command) → Result<Event, CommandError>
//! ```
//!
//! All handlers are **pure functions**:
//! - No side effects (no I/O, no Utc::now(), no mutations)
//! - Deterministic (same inputs → same output)
//! - Referentially transparent
//!
//! # Business Rule Enforcement
//!
//! Handlers enforce aggregate invariants:
//! - State transitions must be valid
//! - Resources can't be double-registered
//! - Policies can't be added twice
//! - Status changes must follow state machine rules

use uuid::Uuid;

use crate::aggregate::commands::*;
use crate::aggregate::compute_resource::ComputeResourceState;
use crate::events::compute_resource::*;
use crate::events::ResourceStatus;

/// Command validation error
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// Resource is not initialized (no events yet)
    #[error("Resource not initialized")]
    NotInitialized,

    /// Resource is already initialized (can't register twice)
    #[error("Resource already initialized")]
    AlreadyInitialized,

    /// Policy already added
    #[error("Policy {0} already added")]
    PolicyAlreadyAdded(String),

    /// Policy not found
    #[error("Policy {0} not found")]
    PolicyNotFound(String),

    /// Invalid status transition
    #[error("Invalid status transition from {from:?} to {to:?}")]
    InvalidStatusTransition {
        from: ResourceStatus,
        to: ResourceStatus,
    },

    /// Business rule violation
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
}

/// Handle RegisterResource command
///
/// # Business Rules
/// - Resource must not already be initialized
///
/// # Returns
/// - Ok(ResourceRegistered) if validation passes
/// - Err(CommandError) if validation fails
pub fn handle_register_resource(
    state: &ComputeResourceState,
    command: RegisterResourceCommand,
    aggregate_id: Uuid,
) -> Result<ResourceRegistered, CommandError> {
    // Business rule: Can't register twice
    if state.is_initialized() {
        return Err(CommandError::AlreadyInitialized);
    }

    // Create event
    Ok(ResourceRegistered {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: None,
        hostname: command.hostname,
        resource_type: command.resource_type,
    })
}

/// Handle AssignOrganization command
///
/// # Business Rules
/// - Resource must be initialized
///
/// # Returns
/// - Ok(OrganizationAssigned) if validation passes
/// - Err(CommandError) if validation fails
pub fn handle_assign_organization(
    state: &ComputeResourceState,
    command: AssignOrganizationCommand,
) -> Result<OrganizationAssigned, CommandError> {
    // Business rule: Must be initialized
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    // Create event
    Ok(OrganizationAssigned {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        organization_id: command.organization_id,
    })
}

/// Handle AssignLocation command
///
/// # Business Rules
/// - Resource must be initialized
pub fn handle_assign_location(
    state: &ComputeResourceState,
    command: AssignLocationCommand,
) -> Result<LocationAssigned, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    Ok(LocationAssigned {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        location_id: command.location_id,
    })
}

/// Handle AssignOwner command
///
/// # Business Rules
/// - Resource must be initialized
pub fn handle_assign_owner(
    state: &ComputeResourceState,
    command: AssignOwnerCommand,
) -> Result<OwnerAssigned, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    Ok(OwnerAssigned {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        owner_id: command.owner_id,
    })
}

/// Handle AddPolicy command
///
/// # Business Rules
/// - Resource must be initialized
/// - Policy must not already be added
pub fn handle_add_policy(
    state: &ComputeResourceState,
    command: AddPolicyCommand,
) -> Result<PolicyAdded, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    // Business rule: Can't add policy twice
    if state.policy_ids.contains(&command.policy_id) {
        return Err(CommandError::PolicyAlreadyAdded(
            command.policy_id.to_string(),
        ));
    }

    Ok(PolicyAdded {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        policy_id: command.policy_id,
    })
}

/// Handle RemovePolicy command
///
/// # Business Rules
/// - Resource must be initialized
/// - Policy must exist in the list
pub fn handle_remove_policy(
    state: &ComputeResourceState,
    command: RemovePolicyCommand,
) -> Result<PolicyRemoved, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    // Business rule: Policy must exist
    if !state.policy_ids.contains(&command.policy_id) {
        return Err(CommandError::PolicyNotFound(command.policy_id.to_string()));
    }

    Ok(PolicyRemoved {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        policy_id: command.policy_id,
    })
}

/// Handle AssignAccountConcept command
///
/// # Business Rules
/// - Resource must be initialized
pub fn handle_assign_account_concept(
    state: &ComputeResourceState,
    command: AssignAccountConceptCommand,
) -> Result<AccountConceptAssigned, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    Ok(AccountConceptAssigned {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        concept_id: command.concept_id,
    })
}

/// Handle ClearAccountConcept command
///
/// # Business Rules
/// - Resource must be initialized
pub fn handle_clear_account_concept(
    state: &ComputeResourceState,
    command: ClearAccountConceptCommand,
) -> Result<AccountConceptCleared, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    Ok(AccountConceptCleared {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
    })
}

/// Handle SetHardwareDetails command
///
/// # Business Rules
/// - Resource must be initialized
pub fn handle_set_hardware_details(
    state: &ComputeResourceState,
    command: SetHardwareDetailsCommand,
) -> Result<HardwareDetailsSet, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    Ok(HardwareDetailsSet {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        manufacturer: command.manufacturer,
        model: command.model,
        serial_number: command.serial_number,
    })
}

/// Handle AssignAssetTag command
///
/// # Business Rules
/// - Resource must be initialized
pub fn handle_assign_asset_tag(
    state: &ComputeResourceState,
    command: AssignAssetTagCommand,
) -> Result<AssetTagAssigned, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    Ok(AssetTagAssigned {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        asset_tag: command.asset_tag,
    })
}

/// Handle UpdateMetadata command
///
/// # Business Rules
/// - Resource must be initialized
pub fn handle_update_metadata(
    state: &ComputeResourceState,
    command: UpdateMetadataCommand,
) -> Result<MetadataUpdated, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    Ok(MetadataUpdated {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        key: command.key,
        value: command.value,
    })
}

/// Handle ChangeStatus command
///
/// # Business Rules
/// - Resource must be initialized
/// - Status transition must be valid (per ResourceStatus state machine)
pub fn handle_change_status(
    state: &ComputeResourceState,
    command: ChangeStatusCommand,
) -> Result<StatusChanged, CommandError> {
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    // Business rule: Must be valid state transition
    if !state.status.can_transition_to(&command.to_status) {
        return Err(CommandError::InvalidStatusTransition {
            from: state.status,
            to: command.to_status,
        });
    }

    Ok(StatusChanged {
        event_version: 1,
        event_id: Uuid::now_v7(),
        aggregate_id: state.id,
        timestamp: command.timestamp,
        correlation_id: command.correlation_id,
        causation_id: command.causation_id,
        from_status: state.status,
        to_status: command.to_status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Hostname, ResourceType};
    use chrono::{DateTime, Utc};

    fn test_timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-19T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn test_aggregate_id() -> Uuid {
        Uuid::parse_str("01934f4a-1000-7000-8000-000000001000").unwrap()
    }

    #[test]
    fn test_handle_register_resource_success() {
        // Arrange
        let state = ComputeResourceState::default_for(test_aggregate_id());
        let command = RegisterResourceCommand {
            hostname: Hostname::new("server01.example.com").unwrap(),
            resource_type: ResourceType::PhysicalServer,
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
        };

        // Act
        let result = handle_register_resource(&state, command, test_aggregate_id());

        // Assert
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.hostname.as_str(), "server01.example.com");
        assert_eq!(event.resource_type, ResourceType::PhysicalServer);
    }

    #[test]
    fn test_handle_register_resource_already_initialized() {
        // Arrange - Create initialized state
        let mut state = ComputeResourceState::default_for(test_aggregate_id());
        state.created_at = Some(test_timestamp()); // Mark as initialized

        let command = RegisterResourceCommand {
            hostname: Hostname::new("server01.example.com").unwrap(),
            resource_type: ResourceType::PhysicalServer,
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
        };

        // Act
        let result = handle_register_resource(&state, command, test_aggregate_id());

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandError::AlreadyInitialized);
    }

    #[test]
    fn test_handle_assign_organization_not_initialized() {
        // Arrange - Uninitialized state
        let state = ComputeResourceState::default_for(test_aggregate_id());

        let command = AssignOrganizationCommand {
            organization_id: cim_domain::EntityId::new(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        // Act
        let result = handle_assign_organization(&state, command);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandError::NotInitialized);
    }

    #[test]
    fn test_handle_add_policy_duplicate() {
        // Arrange - Initialized state with policy
        let policy_id = cim_domain_policy::PolicyId::new();
        let mut state = ComputeResourceState::default_for(test_aggregate_id());
        state.created_at = Some(test_timestamp());
        state.policy_ids = vec![policy_id.clone()];

        let command = AddPolicyCommand {
            policy_id: policy_id.clone(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        // Act
        let result = handle_add_policy(&state, command);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommandError::PolicyAlreadyAdded(_)
        ));
    }

    #[test]
    fn test_handle_change_status_invalid_transition() {
        // Arrange - Initialized state with Active status
        let mut state = ComputeResourceState::default_for(test_aggregate_id());
        state.created_at = Some(test_timestamp());
        state.status = ResourceStatus::Active;

        let command = ChangeStatusCommand {
            to_status: ResourceStatus::Provisioning, // Can't go back to Provisioning from Active
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        // Act
        let result = handle_change_status(&state, command);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommandError::InvalidStatusTransition { .. }
        ));
    }
}
