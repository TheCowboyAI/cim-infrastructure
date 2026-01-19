// Copyright (c) 2025 - Cowboy AI, Inc.
//! Infrastructure Domain Events
//!
//! This module defines all domain events for the infrastructure bounded context.
//! Events are immutable facts representing state changes that have occurred.
//!
//! # Event Sourcing Principles
//!
//! 1. **Events are immutable**: Once created, events never change
//! 2. **Events are past tense**: Named for what happened (Registered, not Register)
//! 3. **Events include metadata**: correlation_id, causation_id, timestamp
//! 4. **Events are versioned**: event_version field for schema evolution
//! 5. **Events are facts**: Represent what happened, not commands
//!
//! # Event Flow
//!
//! ```text
//! Command → Aggregate → Event → EventStore → Projections
//!   (what to do)  (validate)  (what happened)  (persist)  (update views)
//! ```
//!
//! # Correlation and Causation
//!
//! - **correlation_id**: Groups related events across aggregates (e.g., entire request flow)
//! - **causation_id**: Direct parent event that caused this event (event chain)
//!
//! Example:
//! ```text
//! UserRequest
//!   correlation_id: req-123
//!   ↓
//! ResourceRegistered
//!   correlation_id: req-123
//!   causation_id: None (first event)
//!   event_id: evt-1
//!   ↓
//! OrganizationAssigned
//!   correlation_id: req-123
//!   causation_id: evt-1
//!   event_id: evt-2
//!   ↓
//! LocationAssigned
//!   correlation_id: req-123
//!   causation_id: evt-2
//!   event_id: evt-3
//! ```
//!
//! # Event Versioning
//!
//! All events include an `event_version` field:
//! - Start at version 1
//! - Increment when schema changes
//! - Use upcasting to migrate old events to new schema
//! - Never delete old version handling code
//!
//! # Module Organization
//!
//! - [`infrastructure`] - Top-level polymorphic event envelope
//! - [`compute_resource`] - ComputeResource aggregate events
//! - [`versioning`] (future) - Event version migration infrastructure

pub mod compute_resource;
pub mod infrastructure;

// Re-export commonly used types
pub use compute_resource::{
    AccountConceptAssigned, AccountConceptCleared, AssetTagAssigned, ComputeResourceEvent,
    HardwareDetailsSet, LocationAssigned, MetadataUpdated, OrganizationAssigned, OwnerAssigned,
    PolicyAdded, PolicyRemoved, ResourceRegistered, ResourceStatus, StatusChanged,
};
pub use infrastructure::InfrastructureEvent;
