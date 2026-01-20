// Copyright (c) 2025 - Cowboy AI, Inc.
//! Pure Functional Aggregates
//!
//! This module provides the functional aggregate pattern for event sourcing:
//! - Aggregates are pure functions: State → Command → Result<Event, Error>
//! - State reconstruction via event folding: [Event] → State
//! - No mutations, no side effects
//! - All state changes represented as events
//!
//! # Event Sourcing Pattern
//!
//! ```text
//! Command → Aggregate → Events → Event Store
//!    ↓          ↓          ↓
//! Intent   Validation  Facts
//! ```
//!
//! # Pure Functions
//!
//! All aggregate functions follow these principles:
//! 1. **Referential Transparency**: Same input → Same output
//! 2. **No Side Effects**: No I/O, no mutation, no time
//! 3. **Explicit Dependencies**: All inputs passed as parameters
//! 4. **Immutable State**: State is immutable, return new state
//!
//! # Fold Pattern
//!
//! State is reconstructed by folding events:
//!
//! ```rust,ignore
//! let initial = ComputeResourceState::default();
//! let state = events.iter().fold(initial, |state, event| {
//!     apply_event(state, event)
//! });
//! ```
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use cim_infrastructure::aggregate::compute_resource::*;
//!
//! // Load events from event store
//! let events = event_store.read_events(aggregate_id).await?;
//!
//! // Reconstruct current state
//! let state = ComputeResourceState::from_events(&events);
//!
//! // Handle command (pure function)
//! let command = AssignOrganizationCommand {
//!     organization_id,
//!     timestamp: Utc::now(),
//! };
//!
//! match handle_assign_organization(state, command) {
//!     Ok(event) => {
//!         // Apply event to get new state
//!         let new_state = apply_event(state, &event);
//!         // Persist event
//!         event_store.append(aggregate_id, vec![event], None).await?;
//!     }
//!     Err(err) => {
//!         // Handle business rule violation
//!     }
//! }
//! ```
//!
//! # Design Principles
//!
//! ## 1. Command-Event Separation
//! - Commands express intent (what should happen)
//! - Events express facts (what did happen)
//! - Commands can fail, events cannot
//!
//! ## 2. Pure Event Application
//! - `apply_event(State, Event) → State`
//! - No validation in event application (already happened)
//! - Deterministic reconstruction from events
//!
//! ## 3. Command Handlers
//! - `handle_command(State, Command) → Result<Event, Error>`
//! - All validation happens here
//! - Business rules enforced
//! - Pure functions (no side effects)
//!
//! ## 4. Time as Parameter
//! - Never call `Utc::now()` in domain logic
//! - Timestamp passed explicitly in commands
//! - Enables deterministic testing
//! - Time travel for debugging
//!
//! # References
//!
//! - Greg Young: Event Sourcing
//! - Functional Event Sourcing Decider Pattern
//! - F# Domain Modeling Made Functional

pub mod commands;
pub mod compute_resource;
pub mod handlers;

pub use commands::*;
pub use compute_resource::{
    ComputeResourceState,
    apply_event,
};
pub use handlers::*;
