// Copyright (c) 2025 - Cowboy AI, Inc.

//! Projection Adapter - Categorical Functor F: Events → TargetDatabase
//!
//! This module defines the core abstraction for projecting domain events
//! into read models stored in various databases.
//!
//! # Category Theory Foundation
//!
//! A projection is a **Functor** F: EventStream → DatabaseState where:
//!
//! - **Source Category**: Event streams with sequential composition
//! - **Target Category**: Database states with state transitions
//! - **Functor Mapping**: F(Event) = DatabaseUpdate
//! - **Composition Preservation**: F(e1 ∘ e2) = F(e1) ∘ F(e2)
//!
//! ## Functoriality Properties
//!
//! 1. **Identity**: F(id) = id
//!    - Empty event stream produces no database changes
//!
//! 2. **Composition**: F(g ∘ f) = F(g) ∘ F(f)
//!    - Applying events in sequence is same as applying projected updates in sequence
//!
//! # Architecture
//!
//! ```text
//! EventStream ────F──────> DatabaseState
//!    │                        │
//!    │ Events                 │ Updates
//!    ▼                        ▼
//! [e1, e2, e3]  ──>  [u1, u2, u3]
//! ```
//!
//! # Example Implementation
//!
//! ```rust,no_run
//! use cim_infrastructure::projection::ProjectionAdapter;
//! use async_trait::async_trait;
//! use serde_json::Value;
//!
//! struct MyProjection;
//!
//! #[async_trait]
//! impl ProjectionAdapter for MyProjection {
//!     type Event = Value;
//!     type Error = Box<dyn std::error::Error + Send + Sync>;
//!
//!     async fn project(&mut self, event: Self::Event) -> Result<(), Self::Error> {
//!         // Transform event into database update
//!         println!("Projecting event: {:?}", event);
//!         Ok(())
//!     }
//!
//!     async fn initialize(&mut self) -> Result<(), Self::Error> {
//!         // Setup database schema
//!         Ok(())
//!     }
//!
//!     async fn health_check(&self) -> Result<(), Self::Error> {
//!         // Verify database connectivity
//!         Ok(())
//!     }
//! }
//! ```

pub mod executor;
pub mod pure;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Projection Adapter trait - The categorical Functor
///
/// Defines the mapping F: Events → DatabaseState
///
/// Implementations must preserve:
/// - **Event order**: Events applied in sequence
/// - **Idempotency**: Re-applying same event produces same state
/// - **Consistency**: Database state reflects event history
#[async_trait]
pub trait ProjectionAdapter: Send + Sync {
    /// The event type this projection handles
    type Event: Send + Sync;

    /// Error type for projection operations
    type Error: std::error::Error + Send + Sync;

    /// Project an event into the target database
    ///
    /// This is the core functor mapping: F(Event) = DatabaseUpdate
    ///
    /// # Functoriality
    ///
    /// This method must preserve composition:
    /// - `project(e1)` followed by `project(e2)` should be equivalent to
    ///   projecting the composed event sequence
    ///
    /// # Idempotency
    ///
    /// Calling `project` with the same event multiple times should be safe.
    /// Use event IDs to detect and skip duplicate projections.
    async fn project(&mut self, event: Self::Event) -> Result<(), Self::Error>;

    /// Initialize the projection target (schema, indices, etc.)
    ///
    /// This prepares the target database for receiving projections.
    /// Should be idempotent - safe to call multiple times.
    async fn initialize(&mut self) -> Result<(), Self::Error>;

    /// Health check for the projection target
    ///
    /// Verifies connectivity and readiness of the target database.
    async fn health_check(&self) -> Result<(), Self::Error>;

    /// Reset the projection (clear all projected state)
    ///
    /// WARNING: This is destructive! Use only for rebuilding projections.
    /// Default implementation returns an error indicating reset is not supported.
    async fn reset(&mut self) -> Result<(), Self::Error>;


    /// Get the name of this projection adapter
    fn name(&self) -> &str;
}

/// Errors that can occur during projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectionError {
    /// Projection target is not available
    TargetUnavailable(String),

    /// Event cannot be projected (malformed, unknown type, etc.)
    InvalidEvent(String),

    /// Duplicate event detected (already projected)
    DuplicateEvent(Uuid),

    /// Projection failed due to database error
    DatabaseError(String),

    /// Reset operation is not supported by this projection
    ResetNotSupported,

    /// Initialization failed
    InitializationFailed(String),

    /// Generic projection error
    Other(String),
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectionError::TargetUnavailable(msg) => {
                write!(f, "Projection target unavailable: {}", msg)
            }
            ProjectionError::InvalidEvent(msg) => write!(f, "Invalid event: {}", msg),
            ProjectionError::DuplicateEvent(id) => {
                write!(f, "Duplicate event detected: {}", id)
            }
            ProjectionError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ProjectionError::ResetNotSupported => {
                write!(f, "Reset operation not supported by this projection")
            }
            ProjectionError::InitializationFailed(msg) => {
                write!(f, "Initialization failed: {}", msg)
            }
            ProjectionError::Other(msg) => write!(f, "Projection error: {}", msg),
        }
    }
}

impl std::error::Error for ProjectionError {}

/// Projection manager for coordinating multiple projections
///
/// Note: For simplicity, projection coordination is typically done at the
/// application level. If you need to fan-out events to multiple projections,
/// consider using NATS subscriptions with different consumer groups.

#[cfg(test)]
mod tests {
    use super::*;

    // Mock projection for testing
    struct MockProjection {
        events_received: Vec<String>,
    }

    impl MockProjection {
        fn new() -> Self {
            Self {
                events_received: Vec::new(),
            }
        }
    }

    #[async_trait]
    impl ProjectionAdapter for MockProjection {
        type Event = String;
        type Error = ProjectionError;

        async fn project(&mut self, event: Self::Event) -> Result<(), Self::Error> {
            self.events_received.push(event);
            Ok(())
        }

        async fn initialize(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn health_check(&self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn reset(&mut self) -> Result<(), Self::Error> {
            self.events_received.clear();
            Ok(())
        }

        fn name(&self) -> &str {
            "mock-projection"
        }
    }

    #[tokio::test]
    async fn test_projection_adapter() {
        let mut projection = MockProjection::new();

        projection.initialize().await.unwrap();
        projection.project("event1".to_string()).await.unwrap();
        projection.project("event2".to_string()).await.unwrap();

        assert_eq!(projection.events_received.len(), 2);
        assert_eq!(projection.events_received[0], "event1");
        assert_eq!(projection.events_received[1], "event2");

        // Test reset
        projection.reset().await.unwrap();
        assert_eq!(projection.events_received.len(), 0);
    }
}
