// Copyright (c) 2025 - Cowboy AI, Inc.
//! Functional Reactive Programming (FRP) Abstractions
//!
//! This module provides FRP types for modeling time-varying values in a pure
//! functional way. It distinguishes between continuous-time and discrete-time
//! signals, following classical FRP semantics.
//!
//! # Core Concepts
//!
//! ## Signal<T>
//!
//! Base trait for time-varying values. All FRP types implement this trait.
//!
//! ## Behavior<T> (Continuous-Time)
//!
//! A value that exists at all points in time. You can sample a Behavior at
//! any moment to get its current value.
//!
//! ```text
//! Time: ────────────────────────────→
//! Value:  ≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈
//! ```
//!
//! Examples:
//! - Current aggregate state (derived from event history)
//! - Mouse position
//! - Temperature reading
//!
//! ## Event<T> (Discrete-Time)
//!
//! A value that occurs at specific moments in time. Events have occurrences.
//!
//! ```text
//! Time: ────────────────────────────→
//! Value:      ●       ●   ●       ●
//! ```
//!
//! Examples:
//! - Domain events
//! - Mouse clicks
//! - Sensor alerts
//!
//! # FRP Laws
//!
//! Behaviors and Events must satisfy the following laws:
//!
//! ## Functor Laws
//!
//! ```text
//! map id = id
//! map (g . f) = map g . map f
//! ```
//!
//! ## Applicative Laws
//!
//! ```text
//! pure id <*> v = v
//! pure (.) <*> u <*> v <*> w = u <*> (v <*> w)
//! pure f <*> pure x = pure (f x)
//! u <*> pure y = pure ($ y) <*> u
//! ```
//!
//! # Usage with Event Sourcing
//!
//! In event sourcing, we model:
//!
//! - **Domain Events** as `Event<DomainEvent>`
//! - **Aggregate State** as `Behavior<AggregateState>` (derived via fold)
//! - **Projections** as functions from `Event<E>` to side effects
//!
//! ```rust,ignore
//! use cim_infrastructure::frp::*;
//!
//! // Event stream (discrete-time)
//! let events: Event<ComputeResourceEvent> = Event::from_stream(event_stream);
//!
//! // Aggregate state (continuous-time, derived from events)
//! let state: Behavior<ComputeResourceState> = events.fold(
//!     ComputeResourceState::default(),
//!     |state, event| state.apply(event)
//! );
//!
//! // Query current state at any time
//! let current = state.sample();
//! ```

pub mod signal;
pub mod behavior;
pub mod event;
pub mod combinators;

pub use signal::Signal;
pub use behavior::Behavior;
pub use event::DiscreteEvent;
pub use combinators::*;

/// Time representation (milliseconds since epoch)
pub type Time = i64;

/// A stream of occurrences at specific points in time
pub type Occurrence<T> = (Time, T);
