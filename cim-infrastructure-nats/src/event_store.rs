//! JetStream-backed event store for infrastructure events
//!
//! Provides persistent event storage using NATS JetStream with:
//! - Event append operations
//! - Stream replay capabilities
//! - Aggregate event streams
//! - Event ordering guarantees

use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use cim_domain_infrastructure::InfrastructureEvent;

/// Error types for event store operations
#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("JetStream error: {0}")]
    JetStream(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event not found: {0}")]
    EventNotFound(Uuid),

    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Invalid event data: {0}")]
    InvalidEventData(String),
}

/// Result type for event store operations
pub type Result<T> = std::result::Result<T, EventStoreError>;

/// Stored event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// Event ID (from domain event)
    pub event_id: Uuid,

    /// Infrastructure aggregate ID
    pub aggregate_id: Uuid,

    /// Event sequence number within aggregate
    pub sequence: u64,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for tracking related events
    pub correlation_id: Uuid,

    /// Causation ID for event chains
    pub causation_id: Uuid,

    /// Event type name
    pub event_type: String,

    /// Event data (serialized InfrastructureEvent)
    pub data: serde_json::Value,

    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Configuration for the event store
#[derive(Debug, Clone)]
pub struct EventStoreConfig {
    /// JetStream stream name
    pub stream_name: String,

    /// Maximum age of events (retention policy)
    pub max_age: Duration,

    /// Maximum number of events to retain
    pub max_messages: i64,

    /// Number of replicas (for clustered NATS)
    pub replicas: usize,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            stream_name: "INFRASTRUCTURE_EVENTS".to_string(),
            max_age: Duration::from_secs(365 * 24 * 60 * 60), // 1 year
            max_messages: 1_000_000,
            replicas: 1,
        }
    }
}

/// JetStream-backed event store
pub struct JetStreamEventStore {
    jetstream: jetstream::Context,
    config: EventStoreConfig,
    stream: Option<Stream>,
}

impl JetStreamEventStore {
    /// Create a new event store with the given configuration
    pub async fn new(
        client: async_nats::Client,
        config: EventStoreConfig,
    ) -> Result<Self> {
        let jetstream = jetstream::new(client);

        let mut store = Self {
            jetstream,
            config,
            stream: None,
        };

        store.ensure_stream().await?;
        Ok(store)
    }

    /// Ensure the JetStream stream exists
    async fn ensure_stream(&mut self) -> Result<()> {
        let stream = self
            .jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: self.config.stream_name.clone(),
                subjects: vec!["infrastructure.>".to_string()],
                max_age: self.config.max_age,
                max_messages: self.config.max_messages,
                num_replicas: self.config.replicas,
                ..Default::default()
            })
            .await
            .map_err(|e| EventStoreError::JetStream(e.to_string()))?;

        self.stream = Some(stream);
        Ok(())
    }

    /// Append an event to the store
    pub async fn append_event(
        &self,
        subject: &str,
        event: &InfrastructureEvent,
        aggregate_id: Uuid,
        sequence: u64,
    ) -> Result<u64> {
        let stored_event = self.to_stored_event(event, aggregate_id, sequence)?;
        let payload = serde_json::to_vec(&stored_event)?;

        let publish_ack = self
            .jetstream
            .publish(subject.to_string(), payload.into())
            .await
            .map_err(|e| EventStoreError::JetStream(e.to_string()))?;

        let ack = publish_ack
            .await
            .map_err(|e| EventStoreError::JetStream(e.to_string()))?;

        Ok(ack.sequence)
    }

    /// Read events for a specific aggregate
    pub async fn read_aggregate_events(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Vec<StoredEvent>> {
        let stream = self
            .stream
            .as_ref()
            .ok_or_else(|| EventStoreError::StreamNotFound(self.config.stream_name.clone()))?;

        // Create a consumer for this aggregate
        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                filter_subject: "infrastructure.>".to_string(),
                durable_name: Some(format!("aggregate-{}", aggregate_id)),
                ..Default::default()
            })
            .await
            .map_err(|e| EventStoreError::JetStream(e.to_string()))?;

        self.read_from_consumer(consumer, Some(aggregate_id)).await
    }

    /// Read all events from the stream
    pub async fn read_all_events(&self) -> Result<Vec<StoredEvent>> {
        let stream = self
            .stream
            .as_ref()
            .ok_or_else(|| EventStoreError::StreamNotFound(self.config.stream_name.clone()))?;

        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                filter_subject: "infrastructure.>".to_string(),
                ..Default::default()
            })
            .await
            .map_err(|e| EventStoreError::JetStream(e.to_string()))?;

        self.read_from_consumer(consumer, None).await
    }

    /// Read events from a consumer
    async fn read_from_consumer(
        &self,
        consumer: PullConsumer,
        aggregate_id_filter: Option<Uuid>,
    ) -> Result<Vec<StoredEvent>> {
        let mut events = Vec::new();
        let mut messages = consumer
            .fetch()
            .max_messages(1000)
            .messages()
            .await
            .map_err(|e| EventStoreError::JetStream(e.to_string()))?;

        while let Some(message) = messages.next().await {
            let message = message.map_err(|e| EventStoreError::JetStream(e.to_string()))?;
            let stored_event: StoredEvent = serde_json::from_slice(&message.payload)?;

            // Filter by aggregate ID if specified
            if let Some(filter_id) = aggregate_id_filter {
                if stored_event.aggregate_id == filter_id {
                    events.push(stored_event);
                }
            } else {
                events.push(stored_event);
            }

            message
                .ack()
                .await
                .map_err(|e| EventStoreError::JetStream(e.to_string()))?;
        }

        Ok(events)
    }

    /// Convert domain event to stored event
    fn to_stored_event(
        &self,
        event: &InfrastructureEvent,
        aggregate_id: Uuid,
        sequence: u64,
    ) -> Result<StoredEvent> {
        let event_type = self.get_event_type_name(event);
        let data = serde_json::to_value(event)?;

        Ok(StoredEvent {
            event_id: Uuid::now_v7(),
            aggregate_id,
            sequence,
            timestamp: Utc::now(),
            correlation_id: Uuid::now_v7(), // Should come from command context
            causation_id: Uuid::now_v7(),   // Should come from command context
            event_type,
            data,
            metadata: None,
        })
    }

    /// Get event type name from event
    fn get_event_type_name(&self, event: &InfrastructureEvent) -> String {
        use cim_domain_infrastructure::InfrastructureEvent::*;

        match event {
            ComputeResourceRegistered { .. } => "ComputeResourceRegistered",
            ResourceRemoved { .. } => "ResourceRemoved",
            ResourceUpdated { .. } => "ResourceUpdated",
            NetworkDefined { .. } => "NetworkDefined",
            NetworkTopologyDefined { .. } => "NetworkTopologyDefined",
            ResourcesConnected { .. } => "ResourcesConnected",
            SoftwareConfigured { .. } => "SoftwareConfigured",
            PolicyApplied { .. } => "PolicyApplied",
            InterfaceAdded { .. } => "InterfaceAdded",
        }
        .to_string()
    }
}

/// Trait for event store operations
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append an event to the store
    async fn append(
        &self,
        subject: &str,
        event: &InfrastructureEvent,
        aggregate_id: Uuid,
        sequence: u64,
    ) -> Result<u64>;

    /// Read events for an aggregate
    async fn read_aggregate(&self, aggregate_id: Uuid) -> Result<Vec<StoredEvent>>;

    /// Read all events
    async fn read_all(&self) -> Result<Vec<StoredEvent>>;
}

#[async_trait]
impl EventStore for JetStreamEventStore {
    async fn append(
        &self,
        subject: &str,
        event: &InfrastructureEvent,
        aggregate_id: Uuid,
        sequence: u64,
    ) -> Result<u64> {
        self.append_event(subject, event, aggregate_id, sequence)
            .await
    }

    async fn read_aggregate(&self, aggregate_id: Uuid) -> Result<Vec<StoredEvent>> {
        self.read_aggregate_events(aggregate_id).await
    }

    async fn read_all(&self) -> Result<Vec<StoredEvent>> {
        self.read_all_events().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_store_config_default() {
        let config = EventStoreConfig::default();
        assert_eq!(config.stream_name, "INFRASTRUCTURE_EVENTS");
        assert_eq!(config.replicas, 1);
    }

    #[test]
    fn test_stored_event_serialization() {
        let event = StoredEvent {
            event_id: Uuid::now_v7(),
            aggregate_id: Uuid::now_v7(),
            sequence: 1,
            timestamp: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            event_type: "ComputeResourceRegistered".to_string(),
            data: serde_json::json!({"test": "data"}),
            metadata: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StoredEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_id, deserialized.event_id);
        assert_eq!(event.event_type, deserialized.event_type);
    }
}
