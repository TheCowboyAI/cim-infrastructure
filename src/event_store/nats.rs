// Copyright (c) 2025 - Cowboy AI, Inc.
//! NATS JetStream Event Store Implementation
//!
//! This module implements the EventStore trait using NATS JetStream as the
//! persistent storage backend, providing durable event streaming with replay.

use async_nats::jetstream::{self, stream::Stream};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde_json;
use uuid::Uuid;

use crate::errors::{InfrastructureError, InfrastructureResult};
use crate::event_store::EventStore;
use crate::events::InfrastructureEvent;
use crate::jetstream::{create_infrastructure_stream, JetStreamConfig, StoredEvent};
use crate::subjects::AggregateType;

/// NATS JetStream-backed event store
///
/// This implementation uses NATS JetStream for durable event storage with:
/// - Subject-based stream organization
/// - Sequence-based ordering guarantees
/// - Consumer groups for projections
/// - Persistent storage across restarts
///
/// # Example
///
/// ```rust,no_run
/// use cim_infrastructure::event_store::NatsEventStore;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = NatsEventStore::connect("nats://10.0.20.1:4222").await?;
///     // Use store...
///     Ok(())
/// }
/// ```
pub struct NatsEventStore {
    /// NATS JetStream context
    jetstream: jetstream::Context,

    /// JetStream stream for infrastructure events
    stream: Stream,

    /// Base subject prefix (e.g., "infrastructure")
    subject_prefix: String,
}

impl NatsEventStore {
    /// Connect to NATS and create event store
    ///
    /// This will connect to the NATS server and create or get the
    /// infrastructure events stream.
    ///
    /// # Arguments
    ///
    /// * `nats_url` - NATS server URL (e.g., "nats://localhost:4222")
    ///
    /// # Returns
    ///
    /// Connected NatsEventStore instance
    pub async fn connect(nats_url: &str) -> InfrastructureResult<Self> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

        let jetstream = jetstream::new(client);

        let config = JetStreamConfig::default();
        let stream = create_infrastructure_stream(jetstream.clone(), config).await?;

        Ok(Self {
            jetstream,
            stream,
            subject_prefix: "infrastructure".to_string(),
        })
    }

    /// Connect with custom configuration
    pub async fn connect_with_config(
        nats_url: &str,
        config: JetStreamConfig,
    ) -> InfrastructureResult<Self> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

        let jetstream = jetstream::new(client);
        let stream = create_infrastructure_stream(jetstream.clone(), config).await?;

        Ok(Self {
            jetstream,
            stream,
            subject_prefix: "infrastructure".to_string(),
        })
    }

    /// Build subject for an aggregate event
    ///
    /// Format: infrastructure.compute.<aggregate_id>.<event_type>
    fn build_subject(&self, aggregate_id: Uuid, event_type: &str) -> String {
        format!(
            "{}.{}.{}.{}",
            self.subject_prefix,
            AggregateType::Compute,
            aggregate_id,
            event_type.to_lowercase()
        )
    }

    /// Get stream subject filter for an aggregate
    ///
    /// Format: infrastructure.compute.<aggregate_id>.>
    fn aggregate_subject_filter(&self, aggregate_id: Uuid) -> String {
        format!(
            "{}.{}.{}.>",
            self.subject_prefix,
            AggregateType::Compute,
            aggregate_id
        )
    }
}

#[async_trait]
impl EventStore for NatsEventStore {
    async fn append(
        &self,
        aggregate_id: Uuid,
        events: Vec<InfrastructureEvent>,
        expected_version: Option<u64>,
    ) -> InfrastructureResult<u64> {
        // Get current version for concurrency check
        let current_version = self.get_version(aggregate_id).await?;

        // Verify expected version matches
        if let Some(expected) = expected_version {
            match current_version {
                Some(current) if current != expected => {
                    return Err(InfrastructureError::ConcurrencyError(format!(
                        "Expected version {}, but current version is {}",
                        expected, current
                    )));
                }
                None if expected != 0 => {
                    return Err(InfrastructureError::ConcurrencyError(format!(
                        "Expected version {}, but aggregate has no events",
                        expected
                    )));
                }
                _ => {}
            }
        }

        let mut next_sequence = current_version.map(|v| v + 1).unwrap_or(1);

        // Append each event
        for event in events {
            let event_type = event.event_type_name();
            let subject = self.build_subject(aggregate_id, event_type);

            // Wrap in StoredEvent envelope
            let stored_event = StoredEvent {
                event_id: event.aggregate_id(), // Use event's ID
                aggregate_id,
                sequence: next_sequence,
                timestamp: event.timestamp(),
                correlation_id: event.correlation_id(),
                causation_id: event.causation_id().unwrap_or(event.aggregate_id()),
                event_type: event_type.to_string(),
                data: event,
                metadata: None,
            };

            // Serialize to JSON
            let payload = serde_json::to_vec(&stored_event)
                .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;

            // Publish to JetStream
            self.jetstream
                .publish(subject, payload.into())
                .await
                .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?
                .await
                .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

            next_sequence += 1;
        }

        Ok(next_sequence - 1)
    }

    async fn read_events(
        &self,
        aggregate_id: Uuid,
    ) -> InfrastructureResult<Vec<StoredEvent<InfrastructureEvent>>> {
        self.read_events_from(aggregate_id, 1).await
    }

    async fn read_events_from(
        &self,
        aggregate_id: Uuid,
        from_version: u64,
    ) -> InfrastructureResult<Vec<StoredEvent<InfrastructureEvent>>> {
        // Create consumer for this aggregate
        let filter_subject = self.aggregate_subject_filter(aggregate_id);

        let consumer = self
            .stream
            .create_consumer(jetstream::consumer::pull::Config {
                filter_subject: filter_subject.clone(),
                ..Default::default()
            })
            .await
            .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

        let mut events = Vec::new();

        // Fetch messages in bounded batches to avoid infinite wait
        // Use a reasonable batch size - most aggregates will have < 10000 events
        const BATCH_SIZE: usize = 10000;

        loop {
            // Fetch a batch of messages with short timeout
            // If no messages available, fetch will timeout and we treat that as "no more messages"
            let messages_result = consumer
                .fetch()
                .max_messages(BATCH_SIZE)
                .expires(std::time::Duration::from_secs(2))
                .messages()
                .await;

            // Handle timeout as "no messages available" rather than error
            let mut messages = match messages_result {
                Ok(msgs) => msgs,
                Err(e) => {
                    // If timeout or "no messages", we're done
                    let err_msg = e.to_string().to_lowercase();
                    if err_msg.contains("timeout") || err_msg.contains("timed out") || err_msg.contains("no messages") {
                        break;
                    }
                    // Other errors are real problems
                    return Err(InfrastructureError::NatsConnection(e.to_string()));
                }
            };

            let mut batch_count = 0;

            while let Some(message) = messages.next().await {
                let msg = message.map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

                // Deserialize StoredEvent
                let stored_event: StoredEvent<InfrastructureEvent> = serde_json::from_slice(&msg.payload)
                    .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;

                // Filter by version
                if stored_event.sequence >= from_version {
                    events.push(stored_event);
                }

                // Acknowledge message
                msg.ack()
                    .await
                    .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

                batch_count += 1;
            }

            // If we got fewer messages than batch size, we've read all available events
            if batch_count < BATCH_SIZE {
                break;
            }
        }

        // Sort by sequence to ensure ordering
        events.sort_by_key(|e| e.sequence);

        Ok(events)
    }

    async fn read_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> InfrastructureResult<Vec<StoredEvent<InfrastructureEvent>>> {
        // Create consumer for all infrastructure events
        let consumer = self
            .stream
            .create_consumer(jetstream::consumer::pull::Config {
                filter_subject: format!("{}.>", self.subject_prefix),
                ..Default::default()
            })
            .await
            .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

        let mut events = Vec::new();

        // Fetch messages in bounded batches to avoid infinite wait
        const BATCH_SIZE: usize = 10000;

        loop {
            // Fetch a batch of messages with short timeout
            // If no messages available, fetch will timeout and we treat that as "no more messages"
            let messages_result = consumer
                .fetch()
                .max_messages(BATCH_SIZE)
                .expires(std::time::Duration::from_secs(2))
                .messages()
                .await;

            // Handle timeout as "no messages available" rather than error
            let mut messages = match messages_result {
                Ok(msgs) => msgs,
                Err(e) => {
                    // If timeout or "no messages", we're done
                    let err_msg = e.to_string().to_lowercase();
                    if err_msg.contains("timeout") || err_msg.contains("timed out") || err_msg.contains("no messages") {
                        break;
                    }
                    // Other errors are real problems
                    return Err(InfrastructureError::NatsConnection(e.to_string()));
                }
            };

            let mut batch_count = 0;

            while let Some(message) = messages.next().await {
                let msg = message.map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

                // Deserialize StoredEvent
                let stored_event: StoredEvent<InfrastructureEvent> = serde_json::from_slice(&msg.payload)
                    .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;

                // Filter by correlation_id
                if stored_event.correlation_id == correlation_id {
                    events.push(stored_event);
                }

                // Acknowledge message
                msg.ack()
                    .await
                    .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

                batch_count += 1;
            }

            // If we got fewer messages than batch size, we've read all available events
            if batch_count < BATCH_SIZE {
                break;
            }
        }

        // Sort by timestamp for chronological order
        events.sort_by_key(|e| e.timestamp);

        Ok(events)
    }

    async fn get_version(&self, aggregate_id: Uuid) -> InfrastructureResult<Option<u64>> {
        let events = self.read_events(aggregate_id).await?;

        Ok(events.iter().map(|e| e.sequence).max())
    }

    async fn read_events_by_time_range(
        &self,
        aggregate_id: Uuid,
        from_time: DateTime<Utc>,
        to_time: DateTime<Utc>,
    ) -> InfrastructureResult<Vec<StoredEvent<InfrastructureEvent>>> {
        let events = self.read_events(aggregate_id).await?;

        let filtered: Vec<_> = events
            .into_iter()
            .filter(|e| e.timestamp >= from_time && e.timestamp <= to_time)
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Hostname, ResourceType};
    use crate::events::compute_resource::{ComputeResourceEvent, ResourceRegistered};

    // Integration tests with real NATS
    // These require a running NATS server and are marked with #[ignore]

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_nats_event_store_integration() -> InfrastructureResult<()> {
        let store = NatsEventStore::connect("nats://10.0.20.1:4222").await?;

        let aggregate_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();

        // Create test event
        let event = InfrastructureEvent::ComputeResource(
            ComputeResourceEvent::ResourceRegistered(ResourceRegistered {
                event_version: 1,
                event_id: Uuid::now_v7(),
                aggregate_id,
                timestamp: Utc::now(),
                correlation_id,
                causation_id: None,
                hostname: Hostname::new("test-server01").unwrap(),
                resource_type: ResourceType::PhysicalServer,
            }),
        );

        // Append event
        let version = store.append(aggregate_id, vec![event], None).await?;
        assert_eq!(version, 1);

        // Read events back
        let events = store.read_events(aggregate_id).await?;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 1);

        // Verify correlation tracking
        let correlated = store.read_by_correlation(correlation_id).await?;
        assert_eq!(correlated.len(), 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_concurrency_control() -> InfrastructureResult<()> {
        let store = NatsEventStore::connect("nats://10.0.20.1:4222").await?;

        let aggregate_id = Uuid::now_v7();

        // Append first event
        let event1 = InfrastructureEvent::ComputeResource(
            ComputeResourceEvent::ResourceRegistered(ResourceRegistered {
                event_version: 1,
                event_id: Uuid::now_v7(),
                aggregate_id,
                timestamp: Utc::now(),
                correlation_id: Uuid::now_v7(),
                causation_id: None,
                hostname: Hostname::new("test-server01").unwrap(),
                resource_type: ResourceType::PhysicalServer,
            }),
        );

        store.append(aggregate_id, vec![event1], None).await?;

        // Try to append with wrong expected version
        let event2 = InfrastructureEvent::ComputeResource(
            ComputeResourceEvent::ResourceRegistered(ResourceRegistered {
                event_version: 1,
                event_id: Uuid::now_v7(),
                aggregate_id,
                timestamp: Utc::now(),
                correlation_id: Uuid::now_v7(),
                causation_id: None,
                hostname: Hostname::new("test-server02").unwrap(),
                resource_type: ResourceType::VirtualMachine,
            }),
        );

        let result = store.append(aggregate_id, vec![event2], Some(0)).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InfrastructureError::ConcurrencyError(_)
        ));

        Ok(())
    }
}
