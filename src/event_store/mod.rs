// Copyright (c) 2025 - Cowboy AI, Inc.
//! Event Store Abstraction
//!
//! This module defines the event storage interface and implementations for
//! persisting and retrieving domain events in event-sourced systems.
//!
//! # Architecture
//!
//! ```text
//! Command → Aggregate → Events → EventStore → Persistent Storage
//!                                    ↓
//!                              Projections
//! ```
//!
//! # Event Store Requirements
//!
//! 1. **Append-Only**: Events are never updated or deleted
//! 2. **Ordered**: Events maintain sequence within aggregate
//! 3. **Correlation**: Events track causation chains
//! 4. **Versioning**: Events support schema evolution
//! 5. **Replay**: Support reconstructing state from events
//!
//! # Example
//!
//! ```rust,no_run
//! use cim_infrastructure::event_store::{EventStore, NatsEventStore};
//! use cim_infrastructure::InfrastructureEvent;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = NatsEventStore::connect("nats://localhost:4222").await?;
//!
//!     // Append event
//!     let aggregate_id = uuid::Uuid::now_v7();
//!     let event = /* ... create event ... */;
//!     store.append(aggregate_id, vec![event]).await?;
//!
//!     // Read events
//!     let events = store.read_events(aggregate_id).await?;
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::InfrastructureResult;
use crate::events::InfrastructureEvent;
use crate::jetstream::StoredEvent;

pub mod nats;

pub use nats::NatsEventStore;

/// Event Store trait for persisting and retrieving domain events
///
/// This trait provides the core interface for event-sourced systems to
/// interact with persistent event storage. Implementations should ensure:
///
/// - **Atomicity**: Appending events succeeds or fails as a unit
/// - **Consistency**: Event ordering is maintained
/// - **Durability**: Events survive system failures
/// - **Replay**: Events can be read back in order
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to an aggregate's event stream
    ///
    /// Events are written atomically - either all succeed or all fail.
    /// The expected_version provides optimistic concurrency control.
    ///
    /// # Arguments
    ///
    /// * `aggregate_id` - The aggregate these events belong to
    /// * `events` - Events to append
    /// * `expected_version` - Expected current version (for concurrency control)
    ///
    /// # Returns
    ///
    /// The new version after appending events
    ///
    /// # Errors
    ///
    /// - `ConcurrencyError` if expected_version doesn't match actual version
    /// - `StorageError` if writing to storage fails
    async fn append(
        &self,
        aggregate_id: Uuid,
        events: Vec<InfrastructureEvent>,
        expected_version: Option<u64>,
    ) -> InfrastructureResult<u64>;

    /// Read all events for an aggregate
    ///
    /// Returns events in the order they were written.
    ///
    /// # Arguments
    ///
    /// * `aggregate_id` - The aggregate to read events for
    ///
    /// # Returns
    ///
    /// Vector of stored events in chronological order
    async fn read_events(
        &self,
        aggregate_id: Uuid,
    ) -> InfrastructureResult<Vec<StoredEvent<InfrastructureEvent>>>;

    /// Read events for an aggregate from a specific version
    ///
    /// Useful for incremental state updates or projections.
    ///
    /// # Arguments
    ///
    /// * `aggregate_id` - The aggregate to read events for
    /// * `from_version` - Start reading from this version (inclusive)
    ///
    /// # Returns
    ///
    /// Vector of stored events starting from from_version
    async fn read_events_from(
        &self,
        aggregate_id: Uuid,
        from_version: u64,
    ) -> InfrastructureResult<Vec<StoredEvent<InfrastructureEvent>>>;

    /// Read all events in a correlation chain
    ///
    /// Retrieves all events that share the same correlation_id,
    /// useful for tracing entire request flows across aggregates.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - The correlation ID to trace
    ///
    /// # Returns
    ///
    /// Vector of stored events with matching correlation_id
    async fn read_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> InfrastructureResult<Vec<StoredEvent<InfrastructureEvent>>>;

    /// Get the current version of an aggregate
    ///
    /// Returns the highest sequence number for the aggregate,
    /// or None if no events exist.
    ///
    /// # Arguments
    ///
    /// * `aggregate_id` - The aggregate to check
    ///
    /// # Returns
    ///
    /// Current version, or None if aggregate has no events
    async fn get_version(&self, aggregate_id: Uuid) -> InfrastructureResult<Option<u64>>;

    /// Read events within a time range
    ///
    /// Useful for temporal queries and time-based projections.
    ///
    /// # Arguments
    ///
    /// * `aggregate_id` - The aggregate to read events for
    /// * `from_time` - Start time (inclusive)
    /// * `to_time` - End time (inclusive)
    ///
    /// # Returns
    ///
    /// Vector of stored events within the time range
    async fn read_events_by_time_range(
        &self,
        aggregate_id: Uuid,
        from_time: DateTime<Utc>,
        to_time: DateTime<Utc>,
    ) -> InfrastructureResult<Vec<StoredEvent<InfrastructureEvent>>>;
}

/// Event metadata for correlation and causation tracking
#[derive(Debug, Clone)]
pub struct EventMetadata {
    /// Correlation ID - groups related events across aggregates
    pub correlation_id: Uuid,

    /// Causation ID - direct parent event that caused this event
    pub causation_id: Uuid,

    /// Optional user/system context
    pub context: Option<serde_json::Value>,
}

impl EventMetadata {
    /// Create new event metadata
    pub fn new(correlation_id: Uuid, causation_id: Uuid) -> Self {
        Self {
            correlation_id,
            causation_id,
            context: None,
        }
    }

    /// Add context metadata
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_metadata_creation() {
        let correlation_id = Uuid::now_v7();
        let causation_id = Uuid::now_v7();

        let metadata = EventMetadata::new(correlation_id, causation_id);

        assert_eq!(metadata.correlation_id, correlation_id);
        assert_eq!(metadata.causation_id, causation_id);
        assert!(metadata.context.is_none());
    }

    #[test]
    fn test_event_metadata_with_context() {
        let correlation_id = Uuid::now_v7();
        let causation_id = Uuid::now_v7();
        let context = serde_json::json!({"user": "alice", "source": "api"});

        let metadata = EventMetadata::new(correlation_id, causation_id)
            .with_context(context.clone());

        assert_eq!(metadata.correlation_id, correlation_id);
        assert_eq!(metadata.causation_id, causation_id);
        assert_eq!(metadata.context, Some(context));
    }
}
