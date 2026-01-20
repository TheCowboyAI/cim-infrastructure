// Copyright (c) 2025 - Cowboy AI, Inc.
//! Behavior - Continuous-Time Signals
//!
//! A `Behavior<T>` represents a value that exists at all points in time.
//! You can sample a Behavior at any moment to get its current value.
//!
//! # Characteristics
//!
//! - **Always has a value**: Can be sampled at any time `t`
//! - **Continuous**: Value exists for all `t ∈ Time`
//! - **Pure**: Sampling is deterministic and referentially transparent
//!
//! # Mathematical Model
//!
//! A Behavior is a function from Time to Value:
//!
//! ```text
//! Behavior<T> ≅ Time → T
//! ```
//!
//! # Usage with Event Sourcing
//!
//! In event sourcing, aggregate state is naturally a Behavior:
//!
//! ```rust,ignore
//! // Events are discrete occurrences
//! let events: DiscreteEvent<ComputeResourceEvent> = ...;
//!
//! // State is continuous (derived from events via fold)
//! let state: Behavior<ComputeResourceState> = events.fold(
//!     ComputeResourceState::default(),
//!     |state, event| state.apply(event)
//! );
//!
//! // Can query state at any time
//! let current = state.sample();
//! ```
//!
//! # Examples
//!
//! ## Constant Behavior
//!
//! ```rust,ignore
//! let constant: Behavior<i32> = Behavior::constant(42);
//! assert_eq!(constant.sample(), 42);
//! ```
//!
//! ## Derived Behavior
//!
//! ```rust,ignore
//! let numbers: Behavior<i32> = Behavior::constant(5);
//! let doubled: Behavior<i32> = numbers.map(|x| x * 2);
//! assert_eq!(doubled.sample(), 10);
//! ```

use super::signal::{Samplable, Signal};
use std::fmt::Debug;
use std::sync::Arc;

/// Continuous-time signal that always has a value
///
/// A `Behavior<T>` can be sampled at any point in time to get the current value.
/// Behaviors are implemented as pure functions from Time to Value.
///
/// # Type Parameters
///
/// - `T`: The type of value the behavior carries (must be Clone + Debug)
///
/// # Implementation
///
/// Behaviors are implemented using `Arc<dyn Fn() -> T>` to allow efficient
/// cloning and sharing of the behavior function.
#[derive(Clone)]
pub struct Behavior<T> {
    /// Function that produces the current value when sampled
    sampler: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Debug> Debug for Behavior<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Behavior<{}>", std::any::type_name::<T>())
    }
}

impl<T: Clone + Debug + Send + Sync + 'static> Behavior<T> {
    /// Create a constant behavior with a fixed value
    ///
    /// The resulting behavior always returns the same value when sampled.
    ///
    /// # Arguments
    ///
    /// * `value` - The constant value
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let constant = Behavior::constant(42);
    /// assert_eq!(constant.sample(), 42);
    /// ```
    pub fn constant(value: T) -> Self {
        Self {
            sampler: Arc::new(move || value.clone()),
        }
    }

    /// Create a behavior from a sampling function
    ///
    /// This allows you to create custom behaviors with arbitrary logic.
    ///
    /// # Arguments
    ///
    /// * `f` - Function that produces a value when called
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::sync::atomic::{AtomicI32, Ordering};
    /// use std::sync::Arc;
    ///
    /// let counter = Arc::new(AtomicI32::new(0));
    /// let counter_clone = counter.clone();
    ///
    /// let behavior = Behavior::from_fn(move || {
    ///     counter_clone.fetch_add(1, Ordering::SeqCst)
    /// });
    /// ```
    pub fn from_fn<F>(f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            sampler: Arc::new(f),
        }
    }

    /// Apply a binary function to two behaviors
    ///
    /// Combines two behaviors using function `f`. When sampled, both behaviors
    /// are sampled and their values are combined.
    ///
    /// # Arguments
    ///
    /// * `other` - The second behavior
    /// * `f` - Function to combine the values
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let a = Behavior::constant(5);
    /// let b = Behavior::constant(3);
    /// let sum = a.apply2(b, |x, y| x + y);
    /// assert_eq!(sum.sample(), 8);
    /// ```
    pub fn apply2<U, V, F>(self, other: Behavior<U>, f: F) -> Behavior<V>
    where
        U: Clone + Debug + 'static,
        V: Clone + Debug + 'static,
        F: Fn(T, U) -> V + Send + Sync + 'static,
    {
        let sampler1 = self.sampler;
        let sampler2 = other.sampler;

        Behavior {
            sampler: Arc::new(move || {
                let v1 = sampler1();
                let v2 = sampler2();
                f(v1, v2)
            }),
        }
    }
}

impl<T: Clone + Debug + Send + Sync + 'static> Signal<T> for Behavior<T> {
    type Mapped<U: Clone + Debug + Send + Sync + 'static> = Behavior<U>;

    fn map<U, F>(self, f: F) -> Self::Mapped<U>
    where
        F: Fn(T) -> U + Clone + Send + Sync + 'static,
        U: Clone + Debug + Send + Sync + 'static,
    {
        let sampler = self.sampler;
        Behavior {
            sampler: Arc::new(move || f(sampler())),
        }
    }
}

impl<T: Clone + Debug + Send + Sync + 'static> Samplable<T> for Behavior<T> {
    fn sample(&self) -> T {
        (self.sampler)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_behavior() {
        let behavior = Behavior::constant(42);
        assert_eq!(behavior.sample(), 42);
        assert_eq!(behavior.sample(), 42); // Consistent
    }

    #[test]
    fn test_behavior_map() {
        let behavior = Behavior::constant(5);
        let doubled = behavior.map(|x| x * 2);
        assert_eq!(doubled.sample(), 10);
    }

    #[test]
    fn test_behavior_map_composition() {
        // Test Functor law: map f . map g = map (f . g)
        let behavior = Behavior::constant(2);

        let result1 = behavior.clone().map(|x| x + 1).map(|x| x * 2);
        let result2 = behavior.map(|x| (x + 1) * 2);

        assert_eq!(result1.sample(), result2.sample());
    }

    #[test]
    fn test_behavior_map_identity() {
        // Test Functor law: map id = id
        let behavior = Behavior::constant(42);
        let mapped = behavior.clone().map(|x| x);

        assert_eq!(behavior.sample(), mapped.sample());
    }

    #[test]
    fn test_behavior_apply2() {
        let a = Behavior::constant(5);
        let b = Behavior::constant(3);
        let sum = a.apply2(b, |x, y| x + y);

        assert_eq!(sum.sample(), 8);
    }

    #[test]
    fn test_behavior_from_fn() {
        use std::sync::atomic::{AtomicI32, Ordering};
        use std::sync::Arc;

        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = counter.clone();

        let behavior = Behavior::from_fn(move || counter_clone.fetch_add(1, Ordering::SeqCst));

        // Each sample increments the counter
        assert_eq!(behavior.sample(), 0);
        assert_eq!(behavior.sample(), 1);
        assert_eq!(behavior.sample(), 2);
    }

    #[test]
    fn test_behavior_clone() {
        let behavior = Behavior::constant(42);
        let cloned = behavior.clone();

        assert_eq!(behavior.sample(), cloned.sample());
    }
}
