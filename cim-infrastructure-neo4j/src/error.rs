// Copyright 2025 Cowboy AI, LLC.

//! Error types for Neo4j projection

use thiserror::Error;

/// Result type for Neo4j projection operations
pub type Result<T> = std::result::Result<T, Neo4jError>;

/// Errors that can occur during Neo4j projection
#[derive(Debug, Error)]
pub enum Neo4jError {
    /// Neo4j database error
    #[error("Neo4j database error: {0}")]
    Database(#[from] neo4rs::Error),

    /// NATS messaging error
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    /// Event deserialization error
    #[error("Failed to deserialize event: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query error
    #[error("Query error: {0}")]
    Query(String),

    /// Projection error
    #[error("Projection error: {0}")]
    Projection(String),
}
