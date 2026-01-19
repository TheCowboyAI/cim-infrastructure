// Copyright (c) 2025 - Cowboy AI, Inc.

//! JetStream configuration and setup for CIM Infrastructure
//!
//! This module provides configuration and initialization for NATS JetStream,
//! following event sourcing patterns with persistent streams.
//!
//! # Architecture
//!
//! JetStream provides:
//! - **Persistent Event Streams**: Durable event storage with replay capability
//! - **Consumer Management**: Pull and push consumers for event processing
//! - **Stream Configuration**: Subject-based stream organization
//! - **Ordering Guarantees**: Sequence numbers and timestamps
//!
//! # Example
//!
//! ```rust,no_run
//! use cim_infrastructure::jetstream::{JetStreamConfig, create_infrastructure_stream};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = async_nats::connect("nats://localhost:4222").await?;
//!     let jetstream = async_nats::jetstream::new(client);
//!
//!     let config = JetStreamConfig::default();
//!     let stream = create_infrastructure_stream(jetstream, config).await?;
//!
//!     Ok(())
//! }
//! ```

use async_nats::jetstream::{self, stream::Stream};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::errors::{InfrastructureError, InfrastructureResult};

/// Configuration for JetStream infrastructure event streams
#[derive(Debug, Clone)]
pub struct JetStreamConfig {
    /// Stream name for infrastructure events
    pub stream_name: String,

    /// Subjects this stream will capture (defaults to "infrastructure.>")
    pub subjects: Vec<String>,

    /// Maximum age of messages (default: 30 days)
    pub max_age: Duration,

    /// Maximum bytes stored in stream (default: 10GB)
    pub max_bytes: i64,

    /// Storage type (File or Memory)
    pub storage: StorageType,

    /// Number of replicas (for clustered NATS)
    pub replicas: usize,

    /// Retention policy
    pub retention: RetentionPolicy,
}

impl Default for JetStreamConfig {
    fn default() -> Self {
        Self {
            stream_name: "INFRASTRUCTURE_EVENTS".to_string(),
            subjects: vec!["infrastructure.>".to_string()],
            max_age: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            max_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            storage: StorageType::File,
            replicas: 1,
            retention: RetentionPolicy::Limits,
        }
    }
}

/// Storage type for JetStream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// File-based storage (persistent across restarts)
    File,
    /// Memory-based storage (faster, but lost on restart)
    Memory,
}

/// Retention policy for stream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionPolicy {
    /// Limits-based retention (based on max_age and max_bytes)
    Limits,
    /// Interest-based retention (messages kept while there are consumers)
    Interest,
    /// Work queue retention (messages deleted after acknowledgment)
    WorkQueue,
}

/// Stored event envelope with metadata
///
/// This wraps domain events with correlation tracking and sequencing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent<E> {
    /// Unique event ID (UUID v7 for time-ordering)
    pub event_id: Uuid,

    /// Aggregate ID this event belongs to
    pub aggregate_id: Uuid,

    /// Sequence number within aggregate stream
    pub sequence: u64,

    /// Event timestamp (when it occurred)
    pub timestamp: DateTime<Utc>,

    /// Correlation ID (tracks related events across aggregates)
    pub correlation_id: Uuid,

    /// Causation ID (immediate cause of this event)
    pub causation_id: Uuid,

    /// Event type name (for deserialization)
    pub event_type: String,

    /// The actual domain event data
    pub data: E,

    /// Optional metadata (e.g., user context, source system)
    pub metadata: Option<serde_json::Value>,
}

impl<E> StoredEvent<E> {
    /// Create a new stored event envelope
    pub fn new(
        event_id: Uuid,
        aggregate_id: Uuid,
        sequence: u64,
        correlation_id: Uuid,
        causation_id: Uuid,
        event_type: impl Into<String>,
        data: E,
    ) -> Self {
        Self {
            event_id,
            aggregate_id,
            sequence,
            timestamp: Utc::now(),
            correlation_id,
            causation_id,
            event_type: event_type.into(),
            data,
            metadata: None,
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Create or update the infrastructure events stream
///
/// This function is idempotent - it will create the stream if it doesn't exist,
/// or update it if the configuration has changed.
pub async fn create_infrastructure_stream(
    jetstream: jetstream::Context,
    config: JetStreamConfig,
) -> InfrastructureResult<Stream> {
    let storage = match config.storage {
        StorageType::File => jetstream::stream::StorageType::File,
        StorageType::Memory => jetstream::stream::StorageType::Memory,
    };

    let retention = match config.retention {
        RetentionPolicy::Limits => jetstream::stream::RetentionPolicy::Limits,
        RetentionPolicy::Interest => jetstream::stream::RetentionPolicy::Interest,
        RetentionPolicy::WorkQueue => jetstream::stream::RetentionPolicy::WorkQueue,
    };

    let stream_config = jetstream::stream::Config {
        name: config.stream_name.clone(),
        subjects: config.subjects,
        max_age: config.max_age,
        max_bytes: config.max_bytes,
        storage,
        num_replicas: config.replicas,
        retention,
        ..Default::default()
    };

    let stream = jetstream
        .get_or_create_stream(stream_config)
        .await
        .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

    Ok(stream)
}

/// Consumer configuration for event processing
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// Consumer name (durable consumers survive restarts)
    pub name: String,

    /// Filter subject (e.g., "infrastructure.compute.>")
    pub filter_subject: Option<String>,

    /// Deliver policy (from beginning, from end, etc.)
    pub deliver_policy: DeliverPolicy,

    /// Acknowledgment policy
    pub ack_policy: AckPolicy,

    /// Maximum number of pending acks
    pub max_ack_pending: i64,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            name: "infrastructure-consumer".to_string(),
            filter_subject: None,
            deliver_policy: DeliverPolicy::All,
            ack_policy: AckPolicy::Explicit,
            max_ack_pending: 1000,
        }
    }
}

/// Deliver policy for consumers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliverPolicy {
    /// Deliver all messages from the stream start
    All,
    /// Deliver only new messages
    New,
    /// Deliver from a specific sequence
    ByStartSequence(u64),
    /// Deliver from a specific time
    ByStartTime(DateTime<Utc>),
}

/// Acknowledgment policy for consumers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckPolicy {
    /// Messages must be explicitly acknowledged
    Explicit,
    /// Messages are automatically acknowledged
    None,
    /// Entire batch must be acknowledged
    All,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = JetStreamConfig::default();
        assert_eq!(config.stream_name, "INFRASTRUCTURE_EVENTS");
        assert_eq!(config.subjects, vec!["infrastructure.>"]);
        assert_eq!(config.storage, StorageType::File);
        assert_eq!(config.retention, RetentionPolicy::Limits);
    }

    #[test]
    fn test_stored_event_creation() {
        let event_id = Uuid::now_v7();
        let aggregate_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();
        let causation_id = event_id;

        let event = StoredEvent::new(
            event_id,
            aggregate_id,
            1,
            correlation_id,
            causation_id,
            "ComputeRegistered",
            "test data",
        );

        assert_eq!(event.event_id, event_id);
        assert_eq!(event.aggregate_id, aggregate_id);
        assert_eq!(event.sequence, 1);
        assert_eq!(event.event_type, "ComputeRegistered");
        assert_eq!(event.data, "test data");
    }
}
