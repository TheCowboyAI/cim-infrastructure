//! Infrastructure components for the Composable Information Machine
//!
//! This module provides infrastructure abstractions including NATS messaging,
//! event store interfaces, and other cross-cutting concerns.

pub mod errors;
pub mod nats;

// Re-export commonly used types
pub use errors::{InfrastructureError, InfrastructureResult};
pub use nats::{MessageHandler, NatsClient, NatsConfig};
