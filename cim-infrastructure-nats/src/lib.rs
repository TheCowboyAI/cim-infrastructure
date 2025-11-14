//! NATS Integration for CIM Infrastructure
//!
//! This crate provides NATS JetStream integration for the CIM infrastructure domain,
//! enabling event sourcing with persistent event stores, event publishing/subscription,
//! and read model projections.
//!
//! # Architecture
//!
//! The integration follows event sourcing patterns with CQRS:
//!
//! - **Event Store**: JetStream-backed persistent event storage
//! - **Publisher**: Publishes domain events to NATS subjects
//! - **Subscriber**: Subscribes to event streams with handlers
//! - **Projections**: Builds read models from event streams
//! - **Subjects**: Type-safe NATS subject hierarchy
//!
//! # Subject Hierarchy
//!
//! All infrastructure events follow the pattern:
//!
//! ```text
//! infrastructure.{aggregate}.{operation}
//! ```
//!
//! Examples:
//! - `infrastructure.compute.registered`
//! - `infrastructure.network.defined`
//! - `infrastructure.connection.established`
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use async_nats;
//! use cim_infrastructure_nats::{
//!     event_store::{JetStreamEventStore, EventStoreConfig},
//!     publisher::EventPublisher,
//!     subjects,
//! };
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to NATS
//!     let client = async_nats::connect("nats://localhost:4222").await?;
//!
//!     // Create event store
//!     let config = EventStoreConfig::default();
//!     let event_store = JetStreamEventStore::new(client.clone(), config).await?;
//!     let event_store = Arc::new(event_store);
//!
//!     // Create publisher
//!     let publisher = EventPublisher::new(client, event_store);
//!
//!     // Publish events...
//!
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - **Event Sourcing**: Complete event store with JetStream persistence
//! - **Type Safety**: Strongly-typed subject patterns and event envelopes
//! - **CQRS**: Separate write (commands) and read (projections) models
//! - **Scalability**: Built on NATS for distributed systems
//! - **Observability**: Integrated tracing for debugging and monitoring
//!
//! # Module Organization
//!
//! - [`event_store`] - JetStream event persistence
//! - [`publisher`] - Event publishing
//! - [`subscriber`] - Event subscription and handlers
//! - [`projections`] - Read model projections
//! - [`subjects`] - NATS subject patterns

pub mod event_store;
pub mod projections;
pub mod publisher;
pub mod subjects;
pub mod subscriber;

// Re-export commonly used types
pub use event_store::{EventStore, EventStoreConfig, JetStreamEventStore, StoredEvent};
pub use projections::{ProjectionManager, TopologyView};
pub use publisher::{EventPublisher, EventPublisherBuilder};
pub use subjects::{AggregateType, Operation, SubjectBuilder};
pub use subscriber::{EventHandler, EventSubscriber, EventSubscriberBuilder};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module with commonly used imports
pub mod prelude {
    pub use crate::event_store::{EventStore, EventStoreConfig, JetStreamEventStore};
    pub use crate::projections::{ProjectionManager, TopologyView};
    pub use crate::publisher::{EventPublisher, EventPublisherBuilder};
    pub use crate::subjects::{subjects, AggregateType, Operation, SubjectBuilder};
    pub use crate::subscriber::{
        EventHandler, EventSubscriber, EventSubscriberBuilder, EventProjection,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
