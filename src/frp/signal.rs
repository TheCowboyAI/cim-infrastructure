// Copyright (c) 2025 - Cowboy AI, Inc.
//! Signal Trait - Base abstraction for time-varying values
//!
//! This module defines the `Signal` trait, which is the foundation for all
//! FRP types. A Signal represents a value that varies over time.
//!
//! # Type Hierarchy
//!
//! ```text
//! Signal<T>
//!   ├── Behavior<T>  (continuous-time)
//!   └── DiscreteEvent<T>  (discrete-time)
//! ```
//!
//! # Functor Laws
//!
//! All Signal implementations must satisfy the Functor laws:
//!
//! 1. **Identity**: `signal.map(|x| x) == signal`
//! 2. **Composition**: `signal.map(f).map(g) == signal.map(|x| g(f(x)))`
//!
//! # Design Rationale
//!
//! We use a trait-based approach rather than an enum to allow for:
//! - Type-level distinction between Behavior and Event
//! - Efficient implementations tailored to each signal type
//! - Extensibility for custom signal types
//!
//! # Example
//!
//! ```rust,ignore
//! use cim_infrastructure::frp::Signal;
//!
//! fn double_signal<S: Signal<i32>>(signal: S) -> impl Signal<i32> {
//!     signal.map(|x| x * 2)
//! }
//! ```

use std::fmt::Debug;

/// Base trait for time-varying values
///
/// A `Signal<T>` represents a value of type `T` that changes over time.
/// Signals are functors, meaning they support the `map` operation.
///
/// # Type Parameters
///
/// - `T`: The type of value the signal carries (must be Send + Sync for concurrency)
///
/// # Implementations
///
/// - `Behavior<T>`: Continuous-time signals (always has a value)
/// - `DiscreteEvent<T>`: Discrete-time signals (has values at specific moments)
pub trait Signal<T: Send + Sync>: Clone + Debug + Send + Sync {
    /// The type of signal produced by map
    type Mapped<U: Clone + Debug + Send + Sync + 'static>: Signal<U>;

    /// Apply a function to the signal's values
    ///
    /// This is the fundamental Functor operation. It transforms a `Signal<T>`
    /// into a `Signal<U>` by applying function `f` to all values.
    ///
    /// # Laws
    ///
    /// Must satisfy the Functor laws:
    /// 1. `signal.map(id) == signal`
    /// 2. `signal.map(f).map(g) == signal.map(|x| g(f(x)))`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let numbers: Behavior<i32> = Behavior::constant(5);
    /// let doubled: Behavior<i32> = numbers.map(|x| x * 2);
    /// ```
    fn map<U, F>(self, f: F) -> Self::Mapped<U>
    where
        F: Fn(T) -> U + Clone + Send + Sync + 'static,
        U: Clone + Debug + Send + Sync + 'static;
}

/// Marker trait for signals that can be sampled at any time
///
/// Only continuous-time signals (Behaviors) implement this trait.
pub trait Samplable<T: Send + Sync>: Signal<T> {
    /// Get the current value of the signal at this moment in time
    ///
    /// # Returns
    ///
    /// The current value of type `T`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let state: Behavior<ComputeResourceState> = ...;
    /// let current: ComputeResourceState = state.sample();
    /// ```
    fn sample(&self) -> T;
}

/// Marker trait for signals with discrete occurrences
///
/// Only discrete-time signals (Events) implement this trait.
pub trait Discrete<T: Send + Sync>: Signal<T> {
    /// Get all occurrences of the signal
    ///
    /// Returns a vector of (time, value) pairs representing when
    /// the signal had occurrences.
    ///
    /// # Returns
    ///
    /// Vector of occurrences, sorted by time
    fn occurrences(&self) -> Vec<crate::frp::Occurrence<T>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that the Signal trait compiles and can be used generically
    fn generic_map<T, S>(signal: S) -> impl Signal<String>
    where
        T: Debug + Clone + ToString + Send + Sync + 'static,
        S: Signal<T>,
    {
        signal.map(|x| x.to_string())
    }

    #[test]
    fn test_signal_trait_compiles() {
        // This test just verifies the trait compiles correctly
        // Concrete implementations are tested in behavior.rs and event.rs
    }
}
