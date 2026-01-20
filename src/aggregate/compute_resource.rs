// Copyright (c) 2025 - Cowboy AI, Inc.
//! Pure Functional ComputeResource Aggregate
//!
//! Implements event sourcing pattern with pure functions:
//! - Immutable state
//! - Pure event application (fold)
//! - Command handlers as pure functions
//! - No side effects, no mutations
//!
//! # Architecture
//!
//! ```text
//! Command → handle_command() → Result<Event, Error>
//!                                    ↓
//! Events → apply_event() → New State
//! ```

use cim_domain::EntityId;
use cim_domain_location::LocationMarker;
use cim_domain_organization::Organization;
use cim_domain_person::PersonId;
use cim_domain_policy::PolicyId;
use cim_domain_spaces::ConceptId;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{Hostname, ResourceType};
use crate::events::compute_resource::*;
use crate::events::infrastructure::InfrastructureEvent;

/// Immutable ComputeResource State
///
/// This is the aggregate root state reconstructed from events.
/// All fields are public for read access, but the struct is immutable.
///
/// # Reconstruction
///
/// ```rust,ignore
/// let state = ComputeResourceState::from_events(&events);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputeResourceState {
    /// Aggregate ID
    pub id: Uuid,

    /// Hostname
    pub hostname: Hostname,

    /// Resource type
    pub resource_type: ResourceType,

    /// Organization ownership
    pub organization_id: Option<EntityId<Organization>>,

    /// Physical location
    pub location_id: Option<EntityId<LocationMarker>>,

    /// Owner/primary contact
    pub owner_id: Option<PersonId>,

    /// Applicable policies
    pub policy_ids: Vec<PolicyId>,

    /// Account concept
    pub account_concept_id: Option<ConceptId>,

    /// Hardware manufacturer
    pub manufacturer: Option<String>,

    /// Hardware model
    pub model: Option<String>,

    /// Serial number
    pub serial_number: Option<String>,

    /// Asset tag
    pub asset_tag: Option<String>,

    /// Custom metadata
    pub metadata: Vec<(String, String)>,

    /// Current status
    pub status: ResourceStatus,

    /// When this aggregate was created (first event timestamp)
    pub created_at: Option<DateTime<Utc>>,

    /// When this aggregate was last modified (latest event timestamp)
    pub updated_at: Option<DateTime<Utc>>,
}

impl ComputeResourceState {
    /// Create default empty state
    ///
    /// Used as initial state for event folding.
    pub fn default_for(id: Uuid) -> Self {
        Self {
            id,
            hostname: Hostname::new("uninitialized").unwrap(),
            resource_type: ResourceType::PhysicalServer,
            organization_id: None,
            location_id: None,
            owner_id: None,
            policy_ids: Vec::new(),
            account_concept_id: None,
            manufacturer: None,
            model: None,
            serial_number: None,
            asset_tag: None,
            metadata: Vec::new(),
            status: ResourceStatus::Provisioning,
            created_at: None,
            updated_at: None,
        }
    }

    /// Reconstruct state from event stream
    ///
    /// This is the core event sourcing fold operation:
    /// ```text
    /// State = fold(Events, InitialState, apply_event)
    /// ```
    pub fn from_events(events: &[ComputeResourceEvent]) -> Self {
        // Get aggregate ID from first event
        let aggregate_id = events
            .first()
            .map(|e| e.aggregate_id())
            .unwrap_or_else(Uuid::now_v7);

        let initial = Self::default_for(aggregate_id);

        events.iter().fold(initial, |state, event| {
            apply_event(state, event)
        })
    }

    /// Check if aggregate is initialized (has events)
    pub fn is_initialized(&self) -> bool {
        self.created_at.is_some()
    }

    /// Get current version (event count)
    pub fn version(&self, events: &[ComputeResourceEvent]) -> u64 {
        events.len() as u64
    }
}

/// Apply event to state (pure function)
///
/// This is the core of event sourcing - reconstructing state by applying events.
/// Each event type has a specific transformation on the state.
///
/// # Invariants
/// - Function is pure (no side effects)
/// - Same event + same state = same result
/// - Never fails (events are facts that happened)
///
/// # Parameters
/// - `state`: Current state before event
/// - `event`: Event to apply
///
/// # Returns
/// New state after applying event
pub fn apply_event(state: ComputeResourceState, event: &ComputeResourceEvent) -> ComputeResourceState {
    use ComputeResourceEvent::*;

    match event {
        ResourceRegistered(e) => {
            ComputeResourceState {
                id: e.aggregate_id,
                hostname: e.hostname.clone(),
                resource_type: e.resource_type.clone(),
                created_at: Some(e.timestamp),
                updated_at: Some(e.timestamp),
                status: ResourceStatus::Provisioning,
                ..state
            }
        }

        OrganizationAssigned(e) => {
            ComputeResourceState {
                organization_id: Some(e.organization_id.clone()),
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        LocationAssigned(e) => {
            ComputeResourceState {
                location_id: Some(e.location_id.clone()),
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        OwnerAssigned(e) => {
            ComputeResourceState {
                owner_id: Some(e.owner_id.clone()),
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        PolicyAdded(e) => {
            let mut policy_ids = state.policy_ids.clone();
            if !policy_ids.contains(&e.policy_id) {
                policy_ids.push(e.policy_id.clone());
            }
            ComputeResourceState {
                policy_ids,
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        PolicyRemoved(e) => {
            let policy_ids: Vec<_> = state
                .policy_ids
                .iter()
                .filter(|&id| id != &e.policy_id)
                .copied()
                .collect();
            ComputeResourceState {
                policy_ids,
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        AccountConceptAssigned(e) => {
            ComputeResourceState {
                account_concept_id: Some(e.concept_id.clone()),
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        AccountConceptCleared(e) => {
            ComputeResourceState {
                account_concept_id: None,
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        HardwareDetailsSet(e) => {
            ComputeResourceState {
                manufacturer: e.manufacturer.clone(),
                model: e.model.clone(),
                serial_number: e.serial_number.clone(),
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        AssetTagAssigned(e) => {
            ComputeResourceState {
                asset_tag: Some(e.asset_tag.clone()),
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        MetadataUpdated(e) => {
            let mut metadata = state.metadata.clone();

            // Update or add the key-value pair
            if let Some(pos) = metadata.iter().position(|(k, _)| k == &e.key) {
                metadata[pos] = (e.key.clone(), e.value.clone());
            } else {
                metadata.push((e.key.clone(), e.value.clone()));
            }

            ComputeResourceState {
                metadata,
                updated_at: Some(e.timestamp),
                ..state
            }
        }

        StatusChanged(e) => {
            ComputeResourceState {
                status: e.to_status,
                updated_at: Some(e.timestamp),
                ..state
            }
        }
    }
}

/// Apply InfrastructureEvent to state (wrapper for polymorphic events)
pub fn apply_infrastructure_event(
    state: ComputeResourceState,
    event: &InfrastructureEvent,
) -> ComputeResourceState {
    match event {
        InfrastructureEvent::ComputeResource(compute_event) => {
            apply_event(state, compute_event)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Hostname, ResourceType};

    // Helper to create test timestamp
    fn test_timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-19T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn test_aggregate_id() -> Uuid {
        Uuid::parse_str("01934f4a-1000-7000-8000-000000001000").unwrap()
    }

    #[test]
    fn test_apply_resource_registered() {
        // Arrange
        let state = ComputeResourceState::default_for(test_aggregate_id());
        let event = ResourceRegistered {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: test_aggregate_id(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            hostname: Hostname::new("server01.example.com").unwrap(),
            resource_type: ResourceType::PhysicalServer,
        };

        // Act
        let new_state = apply_event(state, &ComputeResourceEvent::ResourceRegistered(event.clone()));

        // Assert
        assert_eq!(new_state.hostname.as_str(), "server01.example.com");
        assert_eq!(new_state.resource_type, ResourceType::PhysicalServer);
        assert_eq!(new_state.status, ResourceStatus::Provisioning);
        assert_eq!(new_state.created_at, Some(test_timestamp()));
    }

    #[test]
    fn test_apply_organization_assigned() {
        // Arrange
        let state = ComputeResourceState::default_for(test_aggregate_id());
        let org_id = EntityId::new();
        let event = OrganizationAssigned {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: test_aggregate_id(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            organization_id: org_id.clone(),
        };

        // Act
        let new_state = apply_event(state, &ComputeResourceEvent::OrganizationAssigned(event));

        // Assert
        assert_eq!(new_state.organization_id, Some(org_id));
        assert_eq!(new_state.updated_at, Some(test_timestamp()));
    }

    #[test]
    fn test_apply_policy_added() {
        // Arrange
        let state = ComputeResourceState::default_for(test_aggregate_id());
        let policy_id = PolicyId::new();
        let event = PolicyAdded {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: test_aggregate_id(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            policy_id: policy_id.clone(),
        };

        // Act
        let new_state = apply_event(state, &ComputeResourceEvent::PolicyAdded(event));

        // Assert
        assert_eq!(new_state.policy_ids.len(), 1);
        assert!(new_state.policy_ids.contains(&policy_id));
    }

    #[test]
    fn test_apply_policy_removed() {
        // Arrange
        let policy_id = PolicyId::new();
        let mut state = ComputeResourceState::default_for(test_aggregate_id());
        state.policy_ids = vec![policy_id.clone()];

        let event = PolicyRemoved {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: test_aggregate_id(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            policy_id: policy_id.clone(),
        };

        // Act
        let new_state = apply_event(state, &ComputeResourceEvent::PolicyRemoved(event));

        // Assert
        assert_eq!(new_state.policy_ids.len(), 0);
    }

    #[test]
    fn test_apply_status_changed() {
        // Arrange
        let state = ComputeResourceState::default_for(test_aggregate_id());
        let event = StatusChanged {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: test_aggregate_id(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            from_status: ResourceStatus::Provisioning,
            to_status: ResourceStatus::Active,
        };

        // Act
        let new_state = apply_event(state, &ComputeResourceEvent::StatusChanged(event));

        // Assert
        assert_eq!(new_state.status, ResourceStatus::Active);
    }

    #[test]
    fn test_from_events_reconstructs_state() {
        // Arrange - Create event stream
        let aggregate_id = test_aggregate_id();
        let events = vec![
            ComputeResourceEvent::ResourceRegistered(ResourceRegistered {
                event_version: 1,
                event_id: Uuid::now_v7(),
                aggregate_id,
                timestamp: test_timestamp(),
                correlation_id: Uuid::now_v7(),
                causation_id: None,
                hostname: Hostname::new("server01.example.com").unwrap(),
                resource_type: ResourceType::PhysicalServer,
            }),
            ComputeResourceEvent::OrganizationAssigned(OrganizationAssigned {
                event_version: 1,
                event_id: Uuid::now_v7(),
                aggregate_id,
                timestamp: test_timestamp(),
                correlation_id: Uuid::now_v7(),
                causation_id: None,
                organization_id: EntityId::new(),
            }),
        ];

        // Act
        let state = ComputeResourceState::from_events(&events);

        // Assert
        assert_eq!(state.hostname.as_str(), "server01.example.com");
        assert!(state.organization_id.is_some());
        assert!(state.is_initialized());
    }

    #[test]
    fn test_apply_metadata_updated() {
        // Arrange
        let state = ComputeResourceState::default_for(test_aggregate_id());
        let event = MetadataUpdated {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id: test_aggregate_id(),
            timestamp: test_timestamp(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            key: "environment".to_string(),
            value: "production".to_string(),
        };

        // Act
        let new_state = apply_event(state, &ComputeResourceEvent::MetadataUpdated(event));

        // Assert
        assert_eq!(new_state.metadata.len(), 1);
        assert_eq!(new_state.metadata[0], ("environment".to_string(), "production".to_string()));
    }
}
