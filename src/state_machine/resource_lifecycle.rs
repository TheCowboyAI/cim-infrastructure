// Copyright (c) 2025 - Cowboy AI, Inc.
//! Resource Lifecycle State Machine
//!
//! Formal FSM implementation for ComputeResource lifecycle management.
//! Uses the generic StateMachine trait from parent module.
//!
//! # State Machine Type
//!
//! This is a **Mealy Machine**: outputs depend on both state and input.
//!
//! # States
//!
//! - Provisioning: Initial setup
//! - Active: Operational
//! - Maintenance: Under maintenance
//! - Decommissioned: Retired (terminal)
//!
//! # Inputs (Lifecycle Commands)
//!
//! - Activate: Provisioning → Active
//! - BeginMaintenance: Active → Maintenance
//! - EndMaintenance: Maintenance → Active
//! - Decommission: Any → Decommissioned
//! - FailedProvision: Provisioning → Decommissioned
//!
//! # Outputs
//!
//! - Warnings for state-specific constraints
//! - Metadata about transition

use super::{StateMachine, TransitionError, TransitionResult};
use crate::events::ResourceStatus;

/// Lifecycle command (FSM input)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleCommand {
    /// Activate a provisioning resource
    Activate,

    /// Begin maintenance on active resource
    BeginMaintenance,

    /// End maintenance, return to active
    EndMaintenance,

    /// Decommission resource (from any state)
    Decommission,

    /// Failed provision, move to decommissioned
    FailedProvision,

    /// Stay in current state (idempotent update)
    Update,
}

/// Transition output with metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionOutput {
    /// Warnings generated during transition
    pub warnings: Vec<String>,

    /// Whether this is a critical transition
    pub is_critical: bool,
}

impl TransitionOutput {
    /// Create output with no warnings
    pub fn ok() -> Self {
        Self {
            warnings: Vec::new(),
            is_critical: false,
        }
    }

    /// Create output with warnings
    pub fn with_warnings(warnings: Vec<String>) -> Self {
        Self {
            warnings,
            is_critical: false,
        }
    }

    /// Create output for critical transition
    pub fn critical(warnings: Vec<String>) -> Self {
        Self {
            warnings,
            is_critical: true,
        }
    }
}

impl StateMachine for ResourceStatus {
    type Input = LifecycleCommand;
    type Output = TransitionOutput;

    fn transition(&self, input: &Self::Input) -> TransitionResult<(Self, Self::Output)> {
        use LifecycleCommand::*;
        use ResourceStatus::*;

        match (self, input) {
            // Provisioning transitions
            (Provisioning, Activate) => {
                Ok((Active, TransitionOutput::ok()))
            }
            (Provisioning, FailedProvision) => Ok((
                Decommissioned,
                TransitionOutput::critical(vec!["Provisioning failed".to_string()]),
            )),
            (Provisioning, Decommission) => Ok((
                Decommissioned,
                TransitionOutput::with_warnings(vec![
                    "Decommissioning during provisioning".to_string()
                ]),
            )),
            (Provisioning, Update) => Ok((Provisioning, TransitionOutput::ok())),

            // Active transitions
            (Active, BeginMaintenance) => {
                Ok((Maintenance, TransitionOutput::ok()))
            }
            (Active, Decommission) => Ok((
                Decommissioned,
                TransitionOutput::critical(vec!["Decommissioning active resource".to_string()]),
            )),
            (Active, Update) => Ok((Active, TransitionOutput::ok())),

            // Maintenance transitions
            (Maintenance, EndMaintenance) => {
                Ok((Active, TransitionOutput::ok()))
            }
            (Maintenance, Decommission) => Ok((
                Decommissioned,
                TransitionOutput::with_warnings(vec![
                    "Decommissioning during maintenance".to_string()
                ]),
            )),
            (Maintenance, Update) => Ok((Maintenance, TransitionOutput::ok())),

            // Decommissioned transitions (terminal state)
            (Decommissioned, Update) => Ok((Decommissioned, TransitionOutput::ok())),
            (Decommissioned, _) => Err(TransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: "any state".to_string(),
            }),

            // Invalid transitions
            (Provisioning, BeginMaintenance) => Err(TransitionError::InvalidTransition {
                from: "Provisioning".to_string(),
                to: "Maintenance".to_string(),
            }),
            (Provisioning, EndMaintenance) => Err(TransitionError::InvalidTransition {
                from: "Provisioning".to_string(),
                to: "Active (via EndMaintenance)".to_string(),
            }),
            (Active, Activate) => Err(TransitionError::BusinessRuleViolation(
                "Already active".to_string(),
            )),
            (Active, FailedProvision) => Err(TransitionError::BusinessRuleViolation(
                "Cannot fail provision on active resource".to_string(),
            )),
            (Active, EndMaintenance) => Err(TransitionError::InvalidTransition {
                from: "Active".to_string(),
                to: "Active (via EndMaintenance)".to_string(),
            }),
            (Maintenance, Activate) => Err(TransitionError::BusinessRuleViolation(
                "Already was activated".to_string(),
            )),
            (Maintenance, FailedProvision) => Err(TransitionError::BusinessRuleViolation(
                "Cannot fail provision on resource in maintenance".to_string(),
            )),
            (Maintenance, BeginMaintenance) => Err(TransitionError::BusinessRuleViolation(
                "Already in maintenance".to_string(),
            )),
        }
    }

    fn valid_inputs(&self) -> Vec<Self::Input> {
        use LifecycleCommand::*;
        use ResourceStatus::*;

        match self {
            Provisioning => vec![Activate, FailedProvision, Decommission, Update],
            Active => vec![BeginMaintenance, Decommission, Update],
            Maintenance => vec![EndMaintenance, Decommission, Update],
            Decommissioned => vec![Update],
        }
    }
}

/// Helper to check if transition is allowed
pub fn is_valid_lifecycle_transition(
    from: ResourceStatus,
    to: ResourceStatus,
) -> bool {
    from.can_transition_to(&to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ResourceStatus;

    #[test]
    fn test_provisioning_to_active() {
        let state = ResourceStatus::Provisioning;
        let (new_state, output) = state
            .transition(&LifecycleCommand::Activate)
            .expect("Transition should succeed");

        assert_eq!(new_state, ResourceStatus::Active);
        assert!(!output.is_critical);
        assert!(output.warnings.is_empty());
    }

    #[test]
    fn test_provisioning_to_decommissioned_failed() {
        let state = ResourceStatus::Provisioning;
        let (new_state, output) = state
            .transition(&LifecycleCommand::FailedProvision)
            .expect("Transition should succeed");

        assert_eq!(new_state, ResourceStatus::Decommissioned);
        assert!(output.is_critical);
        assert!(!output.warnings.is_empty());
    }

    #[test]
    fn test_active_to_maintenance() {
        let state = ResourceStatus::Active;
        let (new_state, output) = state
            .transition(&LifecycleCommand::BeginMaintenance)
            .expect("Transition should succeed");

        assert_eq!(new_state, ResourceStatus::Maintenance);
        assert!(!output.is_critical);
    }

    #[test]
    fn test_maintenance_to_active() {
        let state = ResourceStatus::Maintenance;
        let (new_state, _) = state
            .transition(&LifecycleCommand::EndMaintenance)
            .expect("Transition should succeed");

        assert_eq!(new_state, ResourceStatus::Active);
    }

    #[test]
    fn test_decommissioned_is_terminal() {
        let state = ResourceStatus::Decommissioned;

        // Cannot activate decommissioned resource
        let result = state.transition(&LifecycleCommand::Activate);
        assert!(result.is_err());

        // Cannot begin maintenance
        let result = state.transition(&LifecycleCommand::BeginMaintenance);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_provisioning_to_maintenance() {
        let state = ResourceStatus::Provisioning;
        let result = state.transition(&LifecycleCommand::BeginMaintenance);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransitionError::InvalidTransition { .. }
        ));
    }

    #[test]
    fn test_idempotent_updates() {
        let states = vec![
            ResourceStatus::Provisioning,
            ResourceStatus::Active,
            ResourceStatus::Maintenance,
            ResourceStatus::Decommissioned,
        ];

        for state in states {
            let (new_state, _) = state
                .transition(&LifecycleCommand::Update)
                .expect("Update should always succeed");
            assert_eq!(new_state, state);
        }
    }

    #[test]
    fn test_valid_inputs() {
        // Provisioning has multiple valid inputs
        let inputs = ResourceStatus::Provisioning.valid_inputs();
        assert!(inputs.len() > 2);

        // Decommissioned has only Update
        let inputs = ResourceStatus::Decommissioned.valid_inputs();
        assert_eq!(inputs, vec![LifecycleCommand::Update]);
    }

    #[test]
    fn test_can_transition() {
        let state = ResourceStatus::Provisioning;

        // Can activate
        assert!(state.can_transition(&LifecycleCommand::Activate));

        // Cannot begin maintenance from provisioning
        assert!(!state.can_transition(&LifecycleCommand::BeginMaintenance));
    }

    #[test]
    fn test_decommission_from_any_state() {
        let states = vec![
            ResourceStatus::Provisioning,
            ResourceStatus::Active,
            ResourceStatus::Maintenance,
        ];

        for state in states {
            let result = state.transition(&LifecycleCommand::Decommission);
            assert!(result.is_ok());
            let (new_state, _) = result.unwrap();
            assert_eq!(new_state, ResourceStatus::Decommissioned);
        }
    }
}
