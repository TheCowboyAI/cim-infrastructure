//! Event publisher for infrastructure events
//!
//! Publishes infrastructure domain events to NATS subjects with:
//! - Automatic subject routing based on event type
//! - Event serialization
//! - Publish confirmations
//! - Correlation and causation tracking

use async_nats::Client;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info};
use uuid::Uuid;

use cim_domain_infrastructure::InfrastructureEvent;

use crate::event_store::EventStore;
use crate::subjects::{subjects, AggregateType, Operation, SubjectBuilder};

/// Error types for event publishing
#[derive(Debug, Error)]
pub enum PublishError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Invalid event: {0}")]
    InvalidEvent(String),
}

/// Result type for publish operations
pub type Result<T> = std::result::Result<T, PublishError>;

/// Event publisher with event store integration
pub struct EventPublisher {
    #[allow(dead_code)]
    client: Client,
    event_store: Arc<dyn EventStore>,
}

impl EventPublisher {
    /// Create a new event publisher
    pub fn new(client: Client, event_store: Arc<dyn EventStore>) -> Self {
        Self {
            client,
            event_store,
        }
    }

    /// Publish a single infrastructure event
    pub async fn publish(
        &self,
        event: &InfrastructureEvent,
        aggregate_id: Uuid,
        sequence: u64,
    ) -> Result<()> {
        let subject = self.get_subject_for_event(event);

        debug!(
            subject = %subject,
            aggregate_id = %aggregate_id,
            sequence = sequence,
            "Publishing infrastructure event"
        );

        // Store event in JetStream
        let stream_seq = self
            .event_store
            .append(&subject, event, aggregate_id, sequence)
            .await
            .map_err(|e| PublishError::EventStore(e.to_string()))?;

        info!(
            subject = %subject,
            aggregate_id = %aggregate_id,
            sequence = sequence,
            stream_sequence = stream_seq,
            "Event published successfully"
        );

        Ok(())
    }

    /// Publish multiple events (transactional batch)
    pub async fn publish_batch(
        &self,
        events: &[(InfrastructureEvent, Uuid, u64)],
    ) -> Result<Vec<u64>> {
        let mut sequences = Vec::with_capacity(events.len());

        for (event, aggregate_id, sequence) in events {
            let subject = self.get_subject_for_event(event);

            let stream_seq = self
                .event_store
                .append(&subject, event, *aggregate_id, *sequence)
                .await
                .map_err(|e| PublishError::EventStore(e.to_string()))?;

            sequences.push(stream_seq);

            debug!(
                subject = %subject,
                aggregate_id = %aggregate_id,
                sequence = sequence,
                stream_sequence = stream_seq,
                "Batch event published"
            );
        }

        info!(count = events.len(), "Batch published successfully");
        Ok(sequences)
    }

    /// Get NATS subject for an event
    fn get_subject_for_event(&self, event: &InfrastructureEvent) -> String {
        use cim_domain_infrastructure::InfrastructureEvent::*;

        match event {
            ComputeResourceRegistered { .. } => subjects::compute_registered(),
            ResourceRemoved { .. } => subjects::compute_decommissioned(),
            ResourceUpdated { .. } => subjects::compute_updated(),
            NetworkDefined { .. } => subjects::network_defined(),
            NetworkTopologyDefined { .. } => subjects::network_defined(),
            ResourcesConnected { .. } => subjects::connection_established(),
            SoftwareConfigured { .. } => subjects::software_configured(),
            PolicyApplied { .. } => subjects::policy_set(),
            InterfaceAdded { .. } => {
                // InterfaceAdded could be categorized under network
                SubjectBuilder::new()
                    .aggregate(AggregateType::Network)
                    .operation(Operation::Added)
                    .build()
            }
        }
    }
}

/// Message envelope for published events
#[derive(Debug, Clone, Serialize)]
pub struct EventEnvelope<T> {
    /// Event ID
    pub event_id: Uuid,

    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Event sequence number
    pub sequence: u64,

    /// Correlation ID
    pub correlation_id: Uuid,

    /// Causation ID
    pub causation_id: Uuid,

    /// Event timestamp (ISO 8601)
    pub timestamp: String,

    /// Event type name
    pub event_type: String,

    /// Event data
    pub data: T,
}

impl<T> EventEnvelope<T>
where
    T: Serialize,
{
    /// Create a new event envelope
    pub fn new(
        event_id: Uuid,
        aggregate_id: Uuid,
        sequence: u64,
        correlation_id: Uuid,
        causation_id: Uuid,
        event_type: String,
        data: T,
    ) -> Self {
        Self {
            event_id,
            aggregate_id,
            sequence,
            correlation_id,
            causation_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type,
            data,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

/// Builder for creating event publishers
pub struct EventPublisherBuilder {
    client: Option<Client>,
    event_store: Option<Arc<dyn EventStore>>,
}

impl EventPublisherBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            client: None,
            event_store: None,
        }
    }

    /// Set the NATS client
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Set the event store
    pub fn event_store(mut self, event_store: Arc<dyn EventStore>) -> Self {
        self.event_store = Some(event_store);
        self
    }

    /// Build the event publisher
    pub fn build(self) -> Result<EventPublisher> {
        let client = self
            .client
            .ok_or_else(|| PublishError::InvalidEvent("client not set".to_string()))?;

        let event_store = self
            .event_store
            .ok_or_else(|| PublishError::InvalidEvent("event_store not set".to_string()))?;

        Ok(EventPublisher::new(client, event_store))
    }
}

impl Default for EventPublisherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_envelope_creation() {
        let envelope = EventEnvelope::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            1,
            Uuid::now_v7(),
            Uuid::now_v7(),
            "ComputeResourceRegistered".to_string(),
            serde_json::json!({"test": "data"}),
        );

        assert_eq!(envelope.sequence, 1);
        assert_eq!(envelope.event_type, "ComputeResourceRegistered");
    }

    #[test]
    fn test_event_envelope_serialization() {
        let envelope = EventEnvelope::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            1,
            Uuid::now_v7(),
            Uuid::now_v7(),
            "ComputeResourceRegistered".to_string(),
            serde_json::json!({"test": "data"}),
        );

        let json = envelope.to_json().unwrap();
        assert!(!json.is_empty());
    }
}
