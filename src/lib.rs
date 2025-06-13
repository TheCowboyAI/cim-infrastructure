//! Infrastructure components for the Composable Information Machine
//!
//! This module provides infrastructure abstractions including NATS messaging,
//! event store interfaces, and other cross-cutting concerns.

pub mod nats;
pub mod errors;

// Re-export commonly used types
pub use nats::{NatsClient, NatsConfig, MessageHandler};
pub use errors::{InfrastructureError, InfrastructureResult};
