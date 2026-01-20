// Copyright (c) 2025 - Cowboy AI, Inc.
//! ComputeResource Service Layer
//!
//! Provides application service for managing compute resources through
//! event sourcing. This service coordinates:
//! - Command handling via pure functions
//! - Event persistence to JetStream
//! - Event publishing to NATS
//! - State reconstruction from events
//!
//! # Service Pattern
//!
//! ```text
//! Command → Service → Handler → Event → Event Store
//!                                  ↓
//!                              NATS Publish
//!                                  ↓
//!                             Projections
//! ```
//!
//! # Transaction Semantics
//!
//! Each service method is a transaction:
//! 1. Load events from store
//! 2. Reconstruct current state
//! 3. Handle command (pure function)
//! 4. Append event to store (optimistic concurrency)
//! 5. Publish event to NATS
//!
//! If any step fails, the entire transaction fails.

use async_trait::async_trait;
use uuid::Uuid;

use crate::aggregate::commands::*;
use crate::aggregate::handlers::*;
use crate::aggregate::ComputeResourceState;
use crate::event_store::{EventStore, NatsEventStore};
use crate::events::{ComputeResourceEvent, InfrastructureEvent};
use crate::nats::NatsClient;

/// Service layer result type
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Service layer errors
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// Command validation failed
    #[error("Command error: {0}")]
    CommandError(#[from] CommandError),

    /// Event store error
    #[error("Event store error: {0}")]
    EventStoreError(String),

    /// NATS publishing error
    #[error("NATS error: {0}")]
    NatsError(String),

    /// Aggregate not found
    #[error("Aggregate not found: {0}")]
    NotFound(Uuid),

    /// Concurrency conflict
    #[error("Concurrency conflict: expected version {expected}, got {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },

    /// Business rule violation
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
}

/// ComputeResource service trait
///
/// Defines the application service interface for compute resource management.
#[async_trait]
pub trait ComputeResourceService: Send + Sync {
    /// Register a new compute resource
    ///
    /// # Parameters
    /// - `command`: Registration command with hostname and type
    ///
    /// # Returns
    /// - Aggregate ID of the new resource
    async fn register_resource(&self, command: RegisterResourceCommand) -> ServiceResult<Uuid>;

    /// Assign organization to a resource
    async fn assign_organization(
        &self,
        aggregate_id: Uuid,
        command: AssignOrganizationCommand,
    ) -> ServiceResult<()>;

    /// Assign location to a resource
    async fn assign_location(
        &self,
        aggregate_id: Uuid,
        command: AssignLocationCommand,
    ) -> ServiceResult<()>;

    /// Assign owner to a resource
    async fn assign_owner(
        &self,
        aggregate_id: Uuid,
        command: AssignOwnerCommand,
    ) -> ServiceResult<()>;

    /// Add policy to a resource
    async fn add_policy(
        &self,
        aggregate_id: Uuid,
        command: AddPolicyCommand,
    ) -> ServiceResult<()>;

    /// Remove policy from a resource
    async fn remove_policy(
        &self,
        aggregate_id: Uuid,
        command: RemovePolicyCommand,
    ) -> ServiceResult<()>;

    /// Assign account concept for semantic classification
    async fn assign_account_concept(
        &self,
        aggregate_id: Uuid,
        command: AssignAccountConceptCommand,
    ) -> ServiceResult<()>;

    /// Clear account concept
    async fn clear_account_concept(
        &self,
        aggregate_id: Uuid,
        command: ClearAccountConceptCommand,
    ) -> ServiceResult<()>;

    /// Set hardware details
    async fn set_hardware_details(
        &self,
        aggregate_id: Uuid,
        command: SetHardwareDetailsCommand,
    ) -> ServiceResult<()>;

    /// Assign asset tag
    async fn assign_asset_tag(
        &self,
        aggregate_id: Uuid,
        command: AssignAssetTagCommand,
    ) -> ServiceResult<()>;

    /// Update metadata
    async fn update_metadata(
        &self,
        aggregate_id: Uuid,
        command: UpdateMetadataCommand,
    ) -> ServiceResult<()>;

    /// Change resource status
    async fn change_status(
        &self,
        aggregate_id: Uuid,
        command: ChangeStatusCommand,
    ) -> ServiceResult<()>;

    /// Get current state of a resource
    ///
    /// # Parameters
    /// - `aggregate_id`: ID of the resource
    ///
    /// # Returns
    /// - Current state reconstructed from events
    async fn get_resource(&self, aggregate_id: Uuid) -> ServiceResult<ComputeResourceState>;

    /// Check if resource exists
    async fn exists(&self, aggregate_id: Uuid) -> ServiceResult<bool>;
}

/// Event-sourced implementation of ComputeResourceService
///
/// Uses NATS JetStream for event storage and publishing.
pub struct EventSourcedComputeResourceService {
    /// Event store for persistence
    event_store: NatsEventStore,

    /// NATS client for publishing
    nats_client: NatsClient,
}

impl EventSourcedComputeResourceService {
    /// Create a new event-sourced service
    pub fn new(event_store: NatsEventStore, nats_client: NatsClient) -> Self {
        Self {
            event_store,
            nats_client,
        }
    }

    /// Load current state from event store
    async fn load_state(&self, aggregate_id: Uuid) -> ServiceResult<ComputeResourceState> {
        let stored_events = self
            .event_store
            .read_events(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?;

        // Extract ComputeResourceEvent from StoredEvent<InfrastructureEvent>
        let events: Vec<ComputeResourceEvent> = stored_events
            .into_iter()
            .map(|stored| {
                // Currently only ComputeResource events exist
                let InfrastructureEvent::ComputeResource(event) = stored.data;
                event
            })
            .collect();

        Ok(ComputeResourceState::from_events(&events))
    }

    /// Append event and publish to NATS
    async fn append_and_publish(
        &self,
        aggregate_id: Uuid,
        event: ComputeResourceEvent,
        expected_version: Option<u64>,
    ) -> ServiceResult<()> {
        // Append to event store
        self.event_store
            .append(
                aggregate_id,
                vec![InfrastructureEvent::ComputeResource(event.clone())],
                expected_version,
            )
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?;

        // Publish to NATS for projections
        self.publish_event(&event)
            .await
            .map_err(|e| ServiceError::NatsError(e))?;

        Ok(())
    }

    /// Publish event to NATS
    async fn publish_event(&self, event: &ComputeResourceEvent) -> Result<(), String> {
        // Serialize event
        let payload = serde_json::to_vec(event).map_err(|e| format!("Serialization error: {}", e))?;

        // Determine subject based on event type
        let subject = self.event_subject(event);

        // Publish to NATS
        self.nats_client
            .publish(&subject, &payload)
            .await
            .map_err(|e| format!("NATS publish error: {}", e))?;

        Ok(())
    }

    /// Get NATS subject for event
    fn event_subject(&self, event: &ComputeResourceEvent) -> String {
        use crate::events::compute_resource::ComputeResourceEvent::*;

        let event_type = match event {
            ResourceRegistered(_) => "registered",
            OrganizationAssigned(_) => "organization_assigned",
            LocationAssigned(_) => "location_assigned",
            OwnerAssigned(_) => "owner_assigned",
            PolicyAdded(_) => "policy_added",
            PolicyRemoved(_) => "policy_removed",
            AccountConceptAssigned(_) => "account_concept_assigned",
            AccountConceptCleared(_) => "account_concept_cleared",
            HardwareDetailsSet(_) => "hardware_details_set",
            AssetTagAssigned(_) => "asset_tag_assigned",
            MetadataUpdated(_) => "metadata_updated",
            StatusChanged(_) => "status_changed",
        };

        format!("infrastructure.compute.{}.{}", event.aggregate_id(), event_type)
    }
}

#[async_trait]
impl ComputeResourceService for EventSourcedComputeResourceService {
    async fn register_resource(&self, command: RegisterResourceCommand) -> ServiceResult<Uuid> {
        // Generate new aggregate ID
        let aggregate_id = Uuid::now_v7();

        // Handle command (pure function)
        let initial_state = ComputeResourceState::default_for(aggregate_id);
        let event = handle_register_resource(&initial_state, command, aggregate_id)?;

        // Append and publish
        self.append_and_publish(aggregate_id, ComputeResourceEvent::ResourceRegistered(event), None)
            .await?;

        Ok(aggregate_id)
    }

    async fn assign_organization(
        &self,
        aggregate_id: Uuid,
        command: AssignOrganizationCommand,
    ) -> ServiceResult<()> {
        // Load current state
        let state = self.load_state(aggregate_id).await?;

        // Check if initialized
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        // Handle command
        let event = handle_assign_organization(&state, command)?;

        // Get current version
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        // Append and publish
        self.append_and_publish(
            aggregate_id,
            ComputeResourceEvent::OrganizationAssigned(event),
            Some(version),
        )
        .await?;

        Ok(())
    }

    async fn assign_location(
        &self,
        aggregate_id: Uuid,
        command: AssignLocationCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_assign_location(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(
            aggregate_id,
            ComputeResourceEvent::LocationAssigned(event),
            Some(version),
        )
        .await?;

        Ok(())
    }

    async fn assign_owner(
        &self,
        aggregate_id: Uuid,
        command: AssignOwnerCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_assign_owner(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(aggregate_id, ComputeResourceEvent::OwnerAssigned(event), Some(version))
            .await?;

        Ok(())
    }

    async fn add_policy(
        &self,
        aggregate_id: Uuid,
        command: AddPolicyCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_add_policy(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(aggregate_id, ComputeResourceEvent::PolicyAdded(event), Some(version))
            .await?;

        Ok(())
    }

    async fn remove_policy(
        &self,
        aggregate_id: Uuid,
        command: RemovePolicyCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_remove_policy(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(aggregate_id, ComputeResourceEvent::PolicyRemoved(event), Some(version))
            .await?;

        Ok(())
    }

    async fn assign_account_concept(
        &self,
        aggregate_id: Uuid,
        command: AssignAccountConceptCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_assign_account_concept(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(
            aggregate_id,
            ComputeResourceEvent::AccountConceptAssigned(event),
            Some(version),
        )
        .await?;

        Ok(())
    }

    async fn clear_account_concept(
        &self,
        aggregate_id: Uuid,
        command: ClearAccountConceptCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_clear_account_concept(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(
            aggregate_id,
            ComputeResourceEvent::AccountConceptCleared(event),
            Some(version),
        )
        .await?;

        Ok(())
    }

    async fn set_hardware_details(
        &self,
        aggregate_id: Uuid,
        command: SetHardwareDetailsCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_set_hardware_details(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(
            aggregate_id,
            ComputeResourceEvent::HardwareDetailsSet(event),
            Some(version),
        )
        .await?;

        Ok(())
    }

    async fn assign_asset_tag(
        &self,
        aggregate_id: Uuid,
        command: AssignAssetTagCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_assign_asset_tag(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(
            aggregate_id,
            ComputeResourceEvent::AssetTagAssigned(event),
            Some(version),
        )
        .await?;

        Ok(())
    }

    async fn update_metadata(
        &self,
        aggregate_id: Uuid,
        command: UpdateMetadataCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_update_metadata(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(
            aggregate_id,
            ComputeResourceEvent::MetadataUpdated(event),
            Some(version),
        )
        .await?;

        Ok(())
    }

    async fn change_status(
        &self,
        aggregate_id: Uuid,
        command: ChangeStatusCommand,
    ) -> ServiceResult<()> {
        let state = self.load_state(aggregate_id).await?;
        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        let event = handle_change_status(&state, command)?;
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        self.append_and_publish(aggregate_id, ComputeResourceEvent::StatusChanged(event), Some(version))
            .await?;

        Ok(())
    }

    async fn get_resource(&self, aggregate_id: Uuid) -> ServiceResult<ComputeResourceState> {
        let state = self.load_state(aggregate_id).await?;

        if !state.is_initialized() {
            return Err(ServiceError::NotFound(aggregate_id));
        }

        Ok(state)
    }

    async fn exists(&self, aggregate_id: Uuid) -> ServiceResult<bool> {
        let version = self
            .event_store
            .get_version(aggregate_id)
            .await
            .map_err(|e| ServiceError::EventStoreError(e.to_string()))?
            .unwrap_or(0);

        Ok(version > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests require a running NATS server
    // These are basic unit tests for the service structure

    #[test]
    fn test_service_error_display() {
        let err = ServiceError::NotFound(Uuid::now_v7());
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_command_error_conversion() {
        let cmd_err = CommandError::AlreadyInitialized;
        let svc_err: ServiceError = cmd_err.into();
        assert!(matches!(svc_err, ServiceError::CommandError(_)));
    }
}
