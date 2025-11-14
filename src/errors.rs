//! Error types for infrastructure operations

use thiserror::Error;

/// Errors that can occur in infrastructure operations
#[derive(Debug, Error)]
pub enum InfrastructureError {
    /// NATS connection error
    #[error("NATS connection error: {0}")]
    NatsConnection(String),

    /// NATS publish error
    #[error("NATS publish error: {0}")]
    NatsPublish(String),

    /// NATS subscribe error
    #[error("NATS subscribe error: {0}")]
    NatsSubscribe(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Generic infrastructure error
    #[error("Infrastructure error: {0}")]
    Generic(String),
}

/// Result type for infrastructure operations
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;

impl From<async_nats::Error> for InfrastructureError {
    fn from(err: async_nats::Error) -> Self {
        InfrastructureError::NatsConnection(err.to_string())
    }
}

impl From<serde_json::Error> for InfrastructureError {
    fn from(err: serde_json::Error) -> Self {
        InfrastructureError::Serialization(err.to_string())
    }
}
