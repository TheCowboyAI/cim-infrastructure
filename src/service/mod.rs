// Copyright (c) 2025 - Cowboy AI, Inc.
//! Service Layer for Infrastructure Management
//!
//! This module provides the application service layer that orchestrates
//! domain logic, event sourcing, and infrastructure concerns.
//!
//! # Architecture
//!
//! ```text
//! Client Request
//!     ↓
//! Service Layer (this module)
//!     ↓
//! Command Handler → Aggregate → Event
//!     ↓
//! Event Store (NATS JetStream)
//!     ↓
//! NATS Publishing (event bus)
//!     ↓
//! Projections (Neo4j, etc.)
//! ```
//!
//! # Service Pattern
//!
//! Services coordinate between:
//! - **Command Handlers**: Pure domain logic
//! - **Event Store**: Persistence layer
//! - **Event Bus**: NATS publishing for projections
//! - **Query Side**: Read models and projections
//!
//! # Design Principles
//!
//! 1. **Transaction Boundaries**: Services define transaction scope
//! 2. **Command/Query Separation**: Separate write and read paths
//! 3. **Pure Domain Logic**: Services call pure functions
//! 4. **Async by Default**: All I/O is asynchronous
//!
//! # Example
//!
//! ```rust,ignore
//! use cim_infrastructure::service::ComputeResourceService;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let service = EventSourcedComputeResourceService::new(
//!         event_store,
//!         nats_client,
//!     );
//!
//!     // Execute command
//!     let result = service.register_resource(command).await?;
//!
//!     // Query current state
//!     let state = service.get_resource(id).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod compute_resource;

pub use compute_resource::{
    ComputeResourceService, EventSourcedComputeResourceService, ServiceError, ServiceResult,
};
