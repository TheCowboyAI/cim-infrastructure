// Copyright (c) 2025 - Cowboy AI, Inc.
//! Finite State Machine Abstractions
//!
//! This module provides generic, reusable state machine types for modeling
//! domain lifecycles. All state machines are pure functional - transitions
//! are deterministic functions with no side effects.
//!
//! # State Machine Types
//!
//! ## Mealy Machine
//!
//! Output depends on both current state and input:
//! ```text
//! (State, Input) → (State, Output)
//! ```
//!
//! Use when: Event data matters for transition logic
//!
//! ## Moore Machine
//!
//! Output depends only on current state:
//! ```text
//! State → Output
//! (State, Input) → State
//! ```
//!
//! Use when: State alone determines outputs
//!
//! # Design Principles
//!
//! 1. **Type Safety**: States are strongly typed enums
//! 2. **Pure Functions**: All transitions are pure
//! 3. **Explicit**: All transitions explicitly defined
//! 4. **Composable**: State machines can be nested
//!
//! # Example
//!
//! ```rust,ignore
//! use cim_infrastructure::state_machine::*;
//!
//! // Define states
//! enum TrafficLight {
//!     Red,
//!     Yellow,
//!     Green,
//! }
//!
//! // Define inputs
//! enum Signal {
//!     Timer,
//!     Emergency,
//! }
//!
//! // Implement state machine
//! impl StateMachine for TrafficLight {
//!     type Input = Signal;
//!     type Output = Option<Warning>;
//!
//!     fn transition(&self, input: &Self::Input) -> TransitionResult<Self> {
//!         match (self, input) {
//!             (Red, Timer) => Ok(Green),
//!             (Green, Timer) => Ok(Yellow),
//!             (Yellow, Timer) => Ok(Red),
//!             (_, Emergency) => Ok(Red),
//!             _ => Err(TransitionError::InvalidTransition)
//!         }
//!     }
//! }
//! ```

pub mod resource_lifecycle;

/// Result of a state transition
pub type TransitionResult<S> = Result<S, TransitionError>;

/// Errors that can occur during state transitions
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransitionError {
    /// Transition from current state to target state is not allowed
    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    /// Precondition not met for transition
    #[error("Precondition failed: {0}")]
    PreconditionFailed(String),

    /// Postcondition violated after transition
    #[error("Postcondition violated: {0}")]
    PostconditionViolated(String),

    /// Business rule prevents transition
    #[error("Business rule violated: {0}")]
    BusinessRuleViolation(String),
}

/// Trait for finite state machines
///
/// Implement this trait to define a state machine with typed states,
/// inputs, and outputs.
pub trait StateMachine: Sized + Clone {
    /// Input type that triggers transitions
    type Input;

    /// Output type produced by transitions (use () if none)
    type Output;

    /// Attempt to transition to a new state given an input
    ///
    /// # Returns
    /// - Ok((new_state, output)) if transition is valid
    /// - Err(TransitionError) if transition is invalid
    fn transition(&self, input: &Self::Input) -> TransitionResult<(Self, Self::Output)>;

    /// Check if a transition is valid without performing it
    fn can_transition(&self, input: &Self::Input) -> bool {
        self.transition(input).is_ok()
    }

    /// Get all valid inputs from current state (if enumerable)
    fn valid_inputs(&self) -> Vec<Self::Input>
    where
        Self::Input: Clone,
    {
        // Default implementation returns empty vec
        // Override if Input is enumerable
        Vec::new()
    }
}

/// Trait for states with invariants
///
/// States can have invariants that must hold true.
/// These are checked before and after transitions.
pub trait StateInvariant {
    /// Check if state invariants hold
    fn check_invariants(&self) -> Result<(), String>;
}

/// Trait for deterministic state machines
///
/// A deterministic FSM has exactly one transition per (state, input) pair.
pub trait DeterministicFSM: StateMachine {
    /// Verify FSM is deterministic (for testing)
    fn is_deterministic() -> bool {
        true // Override if needed for compile-time verification
    }
}

/// Transition metadata
///
/// Records information about a state transition for auditing.
#[derive(Debug, Clone)]
pub struct Transition<S, I> {
    /// State before transition
    pub from: S,

    /// State after transition
    pub to: S,

    /// Input that triggered transition
    pub input: I,

    /// Timestamp of transition
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<S, I> Transition<S, I> {
    /// Create a new transition record
    pub fn new(from: S, to: S, input: I, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            from,
            to,
            input,
            timestamp,
        }
    }
}

/// State machine with history
///
/// Wraps a state machine and tracks transition history.
#[derive(Debug, Clone)]
pub struct StateMachineWithHistory<FSM: StateMachine> {
    /// Current state
    pub current: FSM,

    /// Transition history
    pub history: Vec<Transition<FSM, FSM::Input>>,
}

impl<FSM: StateMachine> StateMachineWithHistory<FSM> {
    /// Create a new state machine with history tracking
    pub fn new(initial: FSM) -> Self {
        Self {
            current: initial,
            history: Vec::new(),
        }
    }

    /// Transition with history recording
    pub fn transition_with_history(
        &mut self,
        input: FSM::Input,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> TransitionResult<FSM::Output>
    where
        FSM::Input: Clone,
    {
        let from = self.current.clone();
        let (to, output) = self.current.transition(&input)?;

        // Record transition
        self.history
            .push(Transition::new(from, to.clone(), input, timestamp));

        self.current = to;
        Ok(output)
    }

    /// Get transition history
    pub fn get_history(&self) -> &[Transition<FSM, FSM::Input>] {
        &self.history
    }

    /// Get current state
    pub fn current_state(&self) -> &FSM {
        &self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // Simple test FSM: On/Off switch
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Switch {
        Off,
        On,
    }

    #[derive(Clone)]
    enum SwitchInput {
        Press,
    }

    impl StateMachine for Switch {
        type Input = SwitchInput;
        type Output = ();

        fn transition(&self, input: &Self::Input) -> TransitionResult<(Self, Self::Output)> {
            match (self, input) {
                (Switch::Off, SwitchInput::Press) => Ok((Switch::On, ())),
                (Switch::On, SwitchInput::Press) => Ok((Switch::Off, ())),
            }
        }
    }

    #[test]
    fn test_simple_transition() {
        let switch = Switch::Off;
        let (new_state, _) = switch.transition(&SwitchInput::Press).unwrap();
        assert_eq!(new_state, Switch::On);
    }

    #[test]
    fn test_can_transition() {
        let switch = Switch::Off;
        assert!(switch.can_transition(&SwitchInput::Press));
    }

    #[test]
    fn test_state_machine_with_history() {
        let mut fsm = StateMachineWithHistory::new(Switch::Off);

        // First transition
        fsm.transition_with_history(SwitchInput::Press, Utc::now())
            .unwrap();
        assert_eq!(*fsm.current_state(), Switch::On);
        assert_eq!(fsm.get_history().len(), 1);

        // Second transition
        fsm.transition_with_history(SwitchInput::Press, Utc::now())
            .unwrap();
        assert_eq!(*fsm.current_state(), Switch::Off);
        assert_eq!(fsm.get_history().len(), 2);
    }
}
