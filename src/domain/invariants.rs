// Copyright (c) 2025 - Cowboy AI, Inc.
//! Pure Validation Functions - Domain Invariants
//!
//! This module contains all business rule validation functions for ComputeResource.
//! All functions are pure (no side effects) and return detailed validation results.
//!
//! # Invariant Categories
//!
//! 1. **Structural Invariants**: Basic data validity
//! 2. **State Invariants**: Valid combinations of fields
//! 3. **Transition Invariants**: Valid state changes
//! 4. **Business Rules**: Domain-specific constraints
//!
//! # Design Principles
//!
//! - **Pure Functions**: No I/O, no mutations, deterministic
//! - **Explicit Errors**: Return detailed validation failures
//! - **Composable**: Small functions that combine
//! - **Type-Safe**: Use the type system to prevent invalid states

use crate::domain::Hostname;
use crate::events::ResourceStatus;

/// Validation result with detailed error information
pub type ValidationResult = Result<(), ValidationError>;

/// Validation error with context
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// Hostname validation failed
    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    /// State transition not allowed
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: ResourceStatus,
        to: ResourceStatus,
    },

    /// Resource not in required state
    #[error("Resource must be in {required:?} state, currently in {actual:?}")]
    InvalidState {
        required: ResourceStatus,
        actual: ResourceStatus,
    },

    /// Organization required for certain operations
    #[error("Organization assignment required for this operation")]
    OrganizationRequired,

    /// Location required for certain operations
    #[error("Location assignment required before activation")]
    LocationRequired,

    /// Policy constraint violation
    #[error("Policy constraint violated: {0}")]
    PolicyViolation(String),

    /// Hardware details incomplete
    #[error("Hardware details incomplete: missing {field}")]
    IncompleteHardwareDetails { field: String },

    /// Business rule violation
    #[error("Business rule violated: {0}")]
    BusinessRule(String),
}

/// Validate hostname conforms to domain rules
///
/// # Rules
/// - Must be valid DNS hostname
/// - Must not be empty
/// - Must match pattern from Hostname type
pub fn validate_hostname(hostname: &Hostname) -> ValidationResult {
    // Hostname type already validates DNS rules, so if we have a Hostname, it's valid
    if hostname.as_str().is_empty() {
        return Err(ValidationError::InvalidHostname(
            "Hostname cannot be empty".to_string(),
        ));
    }
    Ok(())
}

/// Validate state transition is allowed
///
/// # Rules
/// - Provisioning → Active, Decommissioned
/// - Active → Maintenance, Decommissioned
/// - Maintenance → Active, Decommissioned
/// - Decommissioned → (terminal state)
pub fn validate_state_transition(
    from: ResourceStatus,
    to: ResourceStatus,
) -> ValidationResult {
    if !from.can_transition_to(&to) {
        return Err(ValidationError::InvalidTransition { from, to });
    }
    Ok(())
}

/// Validate resource can be activated
///
/// # Rules
/// - Must be in Provisioning state
/// - Should have organization assigned (optional enforcement)
/// - Should have location assigned (optional enforcement)
///
/// # Parameters
/// - `current_status`: Current resource status
/// - `has_organization`: Whether organization is assigned
/// - `has_location`: Whether location is assigned
pub fn validate_activation_preconditions(
    current_status: ResourceStatus,
    has_organization: bool,
    has_location: bool,
) -> ValidationResult {
    // Must be in Provisioning state
    if current_status != ResourceStatus::Provisioning {
        return Err(ValidationError::InvalidState {
            required: ResourceStatus::Provisioning,
            actual: current_status,
        });
    }

    // Organization is recommended (warning, not error)
    if !has_organization {
        // In a real system, this might log a warning
        // For now, we allow activation without organization
    }

    // Location is recommended before activation
    if !has_location {
        // In a real system, this might log a warning
        // For strict mode: return Err(ValidationError::LocationRequired)
    }

    Ok(())
}

/// Validate resource can enter maintenance
///
/// # Rules
/// - Must be in Active state
/// - Cannot go to maintenance from Provisioning or Decommissioned
pub fn validate_maintenance_preconditions(
    current_status: ResourceStatus,
) -> ValidationResult {
    if current_status != ResourceStatus::Active {
        return Err(ValidationError::InvalidState {
            required: ResourceStatus::Active,
            actual: current_status,
        });
    }
    Ok(())
}

/// Validate decommissioning preconditions
///
/// # Rules
/// - Can be decommissioned from any state except Decommissioned
/// - Once decommissioned, cannot be reactivated
pub fn validate_decommission_preconditions(
    current_status: ResourceStatus,
) -> ValidationResult {
    if current_status == ResourceStatus::Decommissioned {
        return Err(ValidationError::BusinessRule(
            "Resource is already decommissioned".to_string(),
        ));
    }
    Ok(())
}

/// Validate hardware details are complete for production use
///
/// # Rules
/// - Manufacturer should be present
/// - Model should be present
/// - Serial number recommended
pub fn validate_hardware_details(
    manufacturer: &Option<String>,
    model: &Option<String>,
    serial_number: &Option<String>,
) -> ValidationResult {
    if manufacturer.is_none() {
        return Err(ValidationError::IncompleteHardwareDetails {
            field: "manufacturer".to_string(),
        });
    }

    if model.is_none() {
        return Err(ValidationError::IncompleteHardwareDetails {
            field: "model".to_string(),
        });
    }

    // Serial number is recommended but not required
    if serial_number.is_none() {
        // In a real system, might log a warning
    }

    Ok(())
}

/// Validate policy assignment
///
/// # Rules
/// - Policy must not already be assigned
/// - Resource must be initialized
pub fn validate_policy_assignment(
    policy_id: &str,
    existing_policies: &[String],
) -> ValidationResult {
    if existing_policies.contains(&policy_id.to_string()) {
        return Err(ValidationError::PolicyViolation(format!(
            "Policy {} is already assigned",
            policy_id
        )));
    }
    Ok(())
}

/// Validate policy removal
///
/// # Rules
/// - Policy must exist in current assignments
pub fn validate_policy_removal(
    policy_id: &str,
    existing_policies: &[String],
) -> ValidationResult {
    if !existing_policies.contains(&policy_id.to_string()) {
        return Err(ValidationError::PolicyViolation(format!(
            "Policy {} is not assigned",
            policy_id
        )));
    }
    Ok(())
}

/// Validate metadata key format
///
/// # Rules
/// - Key must not be empty
/// - Key should follow naming conventions
pub fn validate_metadata_key(key: &str) -> ValidationResult {
    if key.is_empty() {
        return Err(ValidationError::BusinessRule(
            "Metadata key cannot be empty".to_string(),
        ));
    }

    // Additional rules could be added:
    // - No special characters
    // - Maximum length
    // - Naming conventions (snake_case, etc.)

    Ok(())
}

/// Composite validation for production readiness
///
/// Checks if a resource is ready for production use.
///
/// # Rules
/// - Must be in Active state
/// - Must have organization assigned
/// - Must have location assigned
/// - Hardware details should be complete
pub fn validate_production_readiness(
    status: ResourceStatus,
    has_organization: bool,
    has_location: bool,
    manufacturer: &Option<String>,
    model: &Option<String>,
) -> ValidationResult {
    // Must be active
    if status != ResourceStatus::Active {
        return Err(ValidationError::InvalidState {
            required: ResourceStatus::Active,
            actual: status,
        });
    }

    // Must have organization
    if !has_organization {
        return Err(ValidationError::OrganizationRequired);
    }

    // Must have location
    if !has_location {
        return Err(ValidationError::LocationRequired);
    }

    // Hardware details recommended
    validate_hardware_details(manufacturer, model, &None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_state_transition_valid() {
        assert!(validate_state_transition(
            ResourceStatus::Provisioning,
            ResourceStatus::Active
        )
        .is_ok());

        assert!(
            validate_state_transition(ResourceStatus::Active, ResourceStatus::Maintenance).is_ok()
        );

        assert!(validate_state_transition(
            ResourceStatus::Maintenance,
            ResourceStatus::Active
        )
        .is_ok());
    }

    #[test]
    fn test_validate_state_transition_invalid() {
        let result = validate_state_transition(
            ResourceStatus::Provisioning,
            ResourceStatus::Maintenance,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidTransition { .. }
        ));
    }

    #[test]
    fn test_validate_activation_preconditions() {
        // Valid: Provisioning with org and location
        assert!(validate_activation_preconditions(
            ResourceStatus::Provisioning,
            true,
            true
        )
        .is_ok());

        // Invalid: Not in Provisioning
        assert!(
            validate_activation_preconditions(ResourceStatus::Active, true, true).is_err()
        );
    }

    #[test]
    fn test_validate_maintenance_preconditions() {
        // Valid: Active resource
        assert!(validate_maintenance_preconditions(ResourceStatus::Active).is_ok());

        // Invalid: Provisioning resource
        assert!(validate_maintenance_preconditions(ResourceStatus::Provisioning).is_err());
    }

    #[test]
    fn test_validate_hardware_details() {
        // Valid: Has manufacturer and model
        assert!(validate_hardware_details(
            &Some("Dell".to_string()),
            &Some("PowerEdge R750".to_string()),
            &None
        )
        .is_ok());

        // Invalid: Missing manufacturer
        assert!(validate_hardware_details(
            &None,
            &Some("PowerEdge R750".to_string()),
            &None
        )
        .is_err());

        // Invalid: Missing model
        assert!(validate_hardware_details(&Some("Dell".to_string()), &None, &None).is_err());
    }

    #[test]
    fn test_validate_policy_assignment() {
        let existing = vec!["policy1".to_string(), "policy2".to_string()];

        // Valid: New policy
        assert!(validate_policy_assignment("policy3", &existing).is_ok());

        // Invalid: Duplicate policy
        assert!(validate_policy_assignment("policy1", &existing).is_err());
    }

    #[test]
    fn test_validate_production_readiness() {
        // Valid: All requirements met
        assert!(validate_production_readiness(
            ResourceStatus::Active,
            true,
            true,
            &Some("Dell".to_string()),
            &Some("PowerEdge R750".to_string())
        )
        .is_ok());

        // Invalid: Not active
        assert!(validate_production_readiness(
            ResourceStatus::Provisioning,
            true,
            true,
            &Some("Dell".to_string()),
            &Some("PowerEdge R750".to_string())
        )
        .is_err());

        // Invalid: No organization
        assert!(validate_production_readiness(
            ResourceStatus::Active,
            false,
            true,
            &Some("Dell".to_string()),
            &Some("PowerEdge R750".to_string())
        )
        .is_err());
    }

    #[test]
    fn test_validate_metadata_key() {
        // Valid key
        assert!(validate_metadata_key("environment").is_ok());

        // Invalid: empty key
        assert!(validate_metadata_key("").is_err());
    }
}
