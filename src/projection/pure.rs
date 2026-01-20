// Copyright (c) 2025 - Cowboy AI, Inc.
//! Pure Projection System
//!
//! This module provides a pure functional approach to projections where:
//! - Projections are pure functions: `(State, Event) → (State, Effects)`
//! - Side effects are returned as data, not performed
//! - Replay is trivial: just fold events through the projection function
//!
//! # Architecture
//!
//! ```text
//! Pure Projection Function          Side Effect Executor
//! ─────────────────────────         ──────────────────────
//!
//! (State, Event)                    Effects
//!      │                                 │
//!      ▼                                 ▼
//! ┌──────────────┐                 ┌──────────────┐
//! │   project()  │    Effects      │   execute()  │
//! │  pure func   │ ─────────────>  │  async I/O   │
//! └──────────────┘                 └──────────────┘
//!      │                                 │
//!      ▼                                 ▼
//! (New State, Effects)              Updated Database
//! ```
//!
//! # Benefits
//!
//! 1. **Testability**: Pure functions are easy to test
//! 2. **Replay**: Trivial to rebuild projections from events
//! 3. **Composition**: Multiple projections can be composed
//! 4. **Time Travel**: Can project to any point in event history
//!
//! # Example
//!
//! ```rust,ignore
//! use cim_infrastructure::projection::pure::*;
//!
//! #[derive(Clone)]
//! struct MyState {
//!     count: i32,
//! }
//!
//! fn my_projection(state: MyState, event: String) -> (MyState, Vec<SideEffect>) {
//!     let new_state = MyState { count: state.count + 1 };
//!     let effects = vec![
//!         SideEffect::DatabaseWrite {
//!             collection: "events".to_string(),
//!             data: serde_json::json!({ "event": event }),
//!         }
//!     ];
//!     (new_state, effects)
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

/// Side effects that can be produced by projections
///
/// These are returned as data structures, not performed immediately.
/// An executor can interpret these and perform the actual I/O.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffect {
    /// Write data to a database
    DatabaseWrite {
        /// Target collection/table name
        collection: String,
        /// Data to write (JSON)
        data: Value,
    },

    /// Update existing database record
    DatabaseUpdate {
        /// Target collection/table name
        collection: String,
        /// Record ID
        id: String,
        /// Updates to apply (JSON)
        updates: Value,
    },

    /// Delete from database
    DatabaseDelete {
        /// Target collection/table name
        collection: String,
        /// Record ID
        id: String,
    },

    /// Execute a database query
    DatabaseQuery {
        /// Query string
        query: String,
        /// Query parameters
        params: Vec<Value>,
    },

    /// Log a message
    Log {
        /// Log level
        level: LogLevel,
        /// Message
        message: String,
    },

    /// Emit a derived event
    EmitEvent {
        /// Event type
        event_type: String,
        /// Event data
        data: Value,
    },
}

/// Log levels for logging side effects
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
}

/// Pure projection function type
///
/// Takes current state and an event, returns new state and side effects.
///
/// # Type Parameters
///
/// - `S`: Projection state type (must be Clone)
/// - `E`: Event type
///
/// # Example
///
/// ```rust,ignore
/// fn my_projection(state: MyState, event: MyEvent) -> (MyState, Vec<SideEffect>) {
///     // Pure logic - no I/O!
///     let new_state = state.apply(&event);
///     let effects = vec![
///         SideEffect::DatabaseWrite { /* ... */ }
///     ];
///     (new_state, effects)
/// }
/// ```
pub type PureProjection<S, E> = fn(S, E) -> (S, Vec<SideEffect>);

/// Trait for pure projection state
///
/// Projection state must be:
/// - Cloneable (for functional updates)
/// - Debuggable (for logging and testing)
/// - Default constructible (for initialization)
pub trait ProjectionState: Clone + Debug + Default {}

// Blanket implementation for all types satisfying the bounds
impl<T: Clone + Debug + Default> ProjectionState for T {}

/// Fold a sequence of events through a pure projection
///
/// This is the fundamental operation for projections. Given an initial state
/// and a sequence of events, fold them through the projection function to
/// produce the final state and all accumulated side effects.
///
/// # Arguments
///
/// * `projection` - Pure projection function
/// * `initial_state` - Starting state
/// * `events` - Events to project
///
/// # Returns
///
/// Tuple of (final_state, all_effects)
///
/// # Example
///
/// ```rust,ignore
/// let (final_state, effects) = fold_projection(
///     my_projection,
///     MyState::default(),
///     vec![event1, event2, event3],
/// );
/// ```
pub fn fold_projection<S, E>(
    projection: PureProjection<S, E>,
    initial_state: S,
    events: Vec<E>,
) -> (S, Vec<SideEffect>)
where
    S: Clone,
{
    events.into_iter().fold(
        (initial_state, Vec::new()),
        |(state, mut all_effects), event| {
            let (new_state, mut effects) = projection(state, event);
            all_effects.append(&mut effects);
            (new_state, all_effects)
        },
    )
}

/// Replay events through a projection to rebuild state
///
/// This is identical to `fold_projection` but with a more specific name
/// that emphasizes the replay use case.
///
/// # Arguments
///
/// * `projection` - Pure projection function
/// * `initial_state` - Starting state
/// * `events` - Events to replay
///
/// # Returns
///
/// Tuple of (rebuilt_state, all_effects)
pub fn replay_projection<S, E>(
    projection: PureProjection<S, E>,
    initial_state: S,
    events: Vec<E>,
) -> (S, Vec<SideEffect>)
where
    S: Clone,
{
    fold_projection(projection, initial_state, events)
}

/// Projection result type
///
/// Convenience alias for projection results.
pub type ProjectionResult<S> = Result<(S, Vec<SideEffect>), ProjectionError>;

/// Errors that can occur during projection
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProjectionError {
    /// Invalid event format
    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    /// State transition error
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    /// Projection logic error
    #[error("Projection error: {0}")]
    ProjectionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test state
    #[derive(Clone, Debug, Default, PartialEq)]
    struct CounterState {
        count: i32,
        events_processed: usize,
    }

    // Test event
    #[derive(Clone, Debug)]
    enum CounterEvent {
        Increment,
        Decrement,
        Reset,
    }

    // Pure projection function
    fn counter_projection(state: CounterState, event: CounterEvent) -> (CounterState, Vec<SideEffect>) {
        let mut effects = Vec::new();

        let new_state = match event {
            CounterEvent::Increment => {
                effects.push(SideEffect::Log {
                    level: LogLevel::Debug,
                    message: format!("Incrementing from {}", state.count),
                });
                CounterState {
                    count: state.count + 1,
                    events_processed: state.events_processed + 1,
                }
            }
            CounterEvent::Decrement => {
                effects.push(SideEffect::Log {
                    level: LogLevel::Debug,
                    message: format!("Decrementing from {}", state.count),
                });
                CounterState {
                    count: state.count - 1,
                    events_processed: state.events_processed + 1,
                }
            }
            CounterEvent::Reset => {
                effects.push(SideEffect::Log {
                    level: LogLevel::Info,
                    message: "Resetting counter".to_string(),
                });
                CounterState {
                    count: 0,
                    events_processed: state.events_processed + 1,
                }
            }
        };

        // Always emit a database write
        effects.push(SideEffect::DatabaseWrite {
            collection: "counters".to_string(),
            data: serde_json::json!({
                "count": new_state.count,
                "events_processed": new_state.events_processed,
            }),
        });

        (new_state, effects)
    }

    #[test]
    fn test_pure_projection() {
        let state = CounterState::default();
        let (new_state, effects) = counter_projection(state, CounterEvent::Increment);

        assert_eq!(new_state.count, 1);
        assert_eq!(new_state.events_processed, 1);
        assert_eq!(effects.len(), 2); // Log + DatabaseWrite
    }

    #[test]
    fn test_fold_projection() {
        let events = vec![
            CounterEvent::Increment,
            CounterEvent::Increment,
            CounterEvent::Decrement,
        ];

        let (final_state, effects) = fold_projection(
            counter_projection,
            CounterState::default(),
            events,
        );

        assert_eq!(final_state.count, 1); // +1 +1 -1 = 1
        assert_eq!(final_state.events_processed, 3);
        assert_eq!(effects.len(), 6); // 2 effects per event * 3 events
    }

    #[test]
    fn test_replay_projection() {
        let events = vec![
            CounterEvent::Increment,
            CounterEvent::Increment,
            CounterEvent::Reset,
            CounterEvent::Increment,
        ];

        let (final_state, _effects) = replay_projection(
            counter_projection,
            CounterState::default(),
            events,
        );

        assert_eq!(final_state.count, 1); // +1 +1 reset(0) +1 = 1
        assert_eq!(final_state.events_processed, 4);
    }

    #[test]
    fn test_projection_is_pure() {
        let initial_state = CounterState::default();
        let event = CounterEvent::Increment;

        // Call twice with same inputs
        let (state1, _) = counter_projection(initial_state.clone(), event.clone());
        let (state2, _) = counter_projection(initial_state.clone(), event.clone());

        // Should produce identical results (referential transparency)
        assert_eq!(state1, state2);
    }

    #[test]
    fn test_projection_composition() {
        // Events can be split and composed
        let events1 = vec![CounterEvent::Increment, CounterEvent::Increment];
        let events2 = vec![CounterEvent::Decrement];

        // Project first batch
        let (state1, _) = fold_projection(
            counter_projection,
            CounterState::default(),
            events1,
        );

        // Project second batch from intermediate state
        let (final_state, _) = fold_projection(counter_projection, state1, events2);

        // Should be same as projecting all at once
        let all_events = vec![
            CounterEvent::Increment,
            CounterEvent::Increment,
            CounterEvent::Decrement,
        ];
        let (expected_state, _) = fold_projection(
            counter_projection,
            CounterState::default(),
            all_events,
        );

        assert_eq!(final_state, expected_state);
    }
}
