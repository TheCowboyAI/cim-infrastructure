//! Infrastructure components for the Composable Information Machine
//!
//! This library provides core infrastructure abstractions for CIM systems:
//!
//! - **NATS Integration**: Client abstraction and JetStream configuration
//! - **Subject Patterns**: Semantic NATS subject hierarchy
//! - **Projection System**: Category Theory-based event projection (Functors)
//! - **Adapters**: Concrete projections (Neo4j, etc.)
//!
//! # Architecture
//!
//! The projection system follows Category Theory principles:
//!
//! ```text
//! EventStream ───F──→ DatabaseState
//!   (Source)         (Target)
//! ```
//!
//! Where F is a Functor that preserves:
//! - Identity: F(id) = id
//! - Composition: F(g ∘ f) = F(g) ∘ F(f)
//!
//! # Modules
//!
//! - [`nats`] - NATS client abstraction
//! - [`jetstream`] - JetStream configuration and event store
//! - [`subjects`] - NATS subject patterns
//! - [`projection`] - Projection adapter trait (Functor interface)
//! - [`adapters`] - Concrete projection implementations
//! - [`errors`] - Error types
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use cim_infrastructure::{NatsClient, NatsConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = NatsConfig::default();
//!     let client = NatsClient::new(config).await?;
//!
//!     // Use client for messaging...
//!
//!     Ok(())
//! }
//! ```

// Core modules
pub mod domain;
pub mod errors;
pub mod jetstream;
pub mod nats;
pub mod projection;
pub mod subjects;

// Projection adapters (feature-gated)
pub mod adapters;

// Re-export commonly used types
pub use domain::{
    ComputeResource, ComputeResourceBuilder, ComputeResourceError, Hostname, HostnameError,
    IpAddressWithCidr, MacAddress, Mtu, NetworkError, ResourceCategory, ResourceType, VlanId,
};
pub use errors::{InfrastructureError, InfrastructureResult};
pub use jetstream::{
    AckPolicy, ConsumerConfig, DeliverPolicy, JetStreamConfig, RetentionPolicy, StorageType,
    StoredEvent,
};
pub use nats::{MessageHandler, NatsClient, NatsConfig};
pub use projection::{ProjectionAdapter, ProjectionError};
pub use subjects::{AggregateType, Operation, SubjectBuilder};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
