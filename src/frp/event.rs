// Copyright (c) 2025 - Cowboy AI, Inc.
//! DiscreteEvent - Discrete-Time Signals
//!
//! A `DiscreteEvent<T>` represents values that occur at specific moments in time.
//! Unlike Behaviors which exist at all times, Events only have values at their
//! occurrences.
//!
//! # Characteristics
//!
//! - **Discrete**: Has values only at specific time points
//! - **Occurrences**: Collection of (time, value) pairs
//! - **Pure**: No side effects, deterministic
//!
//! # Mathematical Model
//!
//! An Event is a finite list of occurrences:
//!
//! ```text
//! DiscreteEvent<T> ≅ [(Time, T)]
//! ```
//!
//! # Usage with Event Sourcing
//!
//! In event sourcing, domain events are naturally discrete-time signals:
//!
//! ```rust,ignore
//! use cim_infrastructure::frp::*;
//!
//! // Domain events as discrete-time signal
//! let events: DiscreteEvent<ComputeResourceEvent> = DiscreteEvent::from_vec(vec![
//!     (1000, ResourceRegistered { /* ... */ }),
//!     (2000, OrganizationAssigned { /* ... */ }),
//!     (3000, StatusChanged { /* ... */ }),
//! ]);
//!
//! // Filter to specific event types
//! let status_changes = events.filter(|e| matches!(e, StatusChanged { .. }));
//!
//! // Fold into continuous behavior (aggregate state)
//! let state: Behavior<ComputeResourceState> = events.fold(
//!     ComputeResourceState::default(),
//!     |state, event| state.apply(event)
//! );
//! ```
//!
//! # Examples
//!
//! ## Creating Events
//!
//! ```rust,ignore
//! let events = DiscreteEvent::from_vec(vec![
//!     (0, "first"),
//!     (1000, "second"),
//!     (2000, "third"),
//! ]);
//! ```
//!
//! ## Filtering Events
//!
//! ```rust,ignore
//! let numbers = DiscreteEvent::from_vec(vec![
//!     (0, 1),
//!     (1, 2),
//!     (2, 3),
//!     (3, 4),
//! ]);
//!
//! let evens = numbers.filter(|x| x % 2 == 0);
//! assert_eq!(evens.occurrences(), vec![(0, 2), (2, 4)]);
//! ```

use super::signal::{Discrete, Signal};
use super::Occurrence;
use std::fmt::Debug;

/// Discrete-time signal with values at specific moments
///
/// A `DiscreteEvent<T>` represents a collection of occurrences, where each
/// occurrence is a (time, value) pair.
///
/// # Type Parameters
///
/// - `T`: The type of value the event carries (must be Clone + Debug + Send + Sync)
#[derive(Clone, Debug)]
pub struct DiscreteEvent<T> {
    /// Occurrences sorted by time
    occurrences: Vec<Occurrence<T>>,
}

impl<T: Clone + Debug + Send + Sync + 'static> DiscreteEvent<T> {
    /// Create an event stream from a vector of occurrences
    ///
    /// The occurrences will be sorted by time.
    ///
    /// # Arguments
    ///
    /// * `occurrences` - Vector of (time, value) pairs
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let events = DiscreteEvent::from_vec(vec![
    ///     (1000, "first"),
    ///     (2000, "second"),
    /// ]);
    /// ```
    pub fn from_vec(mut occurrences: Vec<Occurrence<T>>) -> Self {
        occurrences.sort_by_key(|(time, _)| *time);
        Self { occurrences }
    }

    /// Create an empty event stream
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let empty: DiscreteEvent<i32> = DiscreteEvent::empty();
    /// assert_eq!(empty.occurrences().len(), 0);
    /// ```
    pub fn empty() -> Self {
        Self {
            occurrences: Vec::new(),
        }
    }

    /// Filter occurrences based on a predicate
    ///
    /// Returns a new event stream containing only occurrences where the
    /// predicate returns true.
    ///
    /// # Arguments
    ///
    /// * `predicate` - Function to test each value
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let numbers = DiscreteEvent::from_vec(vec![
    ///     (0, 1),
    ///     (1, 2),
    ///     (2, 3),
    /// ]);
    ///
    /// let evens = numbers.filter(|x| x % 2 == 0);
    /// ```
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool,
    {
        let filtered = self
            .occurrences
            .into_iter()
            .filter(|(_, value)| predicate(value))
            .collect();

        Self {
            occurrences: filtered,
        }
    }

    /// Fold the event stream into a single value
    ///
    /// This is the classical fold operation, processing occurrences in
    /// chronological order.
    ///
    /// # Arguments
    ///
    /// * `init` - Initial accumulator value
    /// * `f` - Function to combine accumulator with each occurrence
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let numbers = DiscreteEvent::from_vec(vec![
    ///     (0, 1),
    ///     (1, 2),
    ///     (2, 3),
    /// ]);
    ///
    /// let sum = numbers.fold(0, |acc, x| acc + x);
    /// assert_eq!(sum, 6);
    /// ```
    pub fn fold<A, F>(self, init: A, f: F) -> A
    where
        F: Fn(A, T) -> A,
    {
        self.occurrences
            .into_iter()
            .fold(init, |acc, (_, value)| f(acc, value))
    }

    /// Scan the event stream to create a new stream of accumulated values
    ///
    /// Like fold, but emits intermediate results at each occurrence.
    ///
    /// # Arguments
    ///
    /// * `init` - Initial accumulator value
    /// * `f` - Function to combine accumulator with each occurrence
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let numbers = DiscreteEvent::from_vec(vec![
    ///     (0, 1),
    ///     (1, 2),
    ///     (2, 3),
    /// ]);
    ///
    /// let running_sum = numbers.scan(0, |acc, x| acc + x);
    /// // Results: (0, 1), (1, 3), (2, 6)
    /// ```
    pub fn scan<A, F>(self, init: A, f: F) -> DiscreteEvent<A>
    where
        A: Clone + Debug + Send + Sync + 'static,
        F: Fn(A, T) -> A,
    {
        let mut acc = init;
        let results = self
            .occurrences
            .into_iter()
            .map(|(time, value)| {
                acc = f(acc.clone(), value);
                (time, acc.clone())
            })
            .collect();

        DiscreteEvent {
            occurrences: results,
        }
    }

    /// Take only the first N occurrences
    ///
    /// # Arguments
    ///
    /// * `n` - Number of occurrences to take
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let events = DiscreteEvent::from_vec(vec![
    ///     (0, 1),
    ///     (1, 2),
    ///     (2, 3),
    /// ]);
    ///
    /// let first_two = events.take(2);
    /// assert_eq!(first_two.occurrences().len(), 2);
    /// ```
    pub fn take(self, n: usize) -> Self {
        let occurrences = self.occurrences.into_iter().take(n).collect();
        Self { occurrences }
    }

    /// Skip the first N occurrences
    ///
    /// # Arguments
    ///
    /// * `n` - Number of occurrences to skip
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let events = DiscreteEvent::from_vec(vec![
    ///     (0, 1),
    ///     (1, 2),
    ///     (2, 3),
    /// ]);
    ///
    /// let last_one = events.skip(2);
    /// assert_eq!(last_one.occurrences().len(), 1);
    /// ```
    pub fn skip(self, n: usize) -> Self {
        let occurrences = self.occurrences.into_iter().skip(n).collect();
        Self { occurrences }
    }
}

impl<T: Clone + Debug + Send + Sync + 'static> Signal<T> for DiscreteEvent<T> {
    type Mapped<U: Clone + Debug + Send + Sync + 'static> = DiscreteEvent<U>;

    fn map<U, F>(self, f: F) -> Self::Mapped<U>
    where
        F: Fn(T) -> U + Clone + Send + Sync + 'static,
        U: Clone + Debug + Send + Sync + 'static,
    {
        let mapped = self
            .occurrences
            .into_iter()
            .map(|(time, value)| (time, f(value)))
            .collect();

        DiscreteEvent {
            occurrences: mapped,
        }
    }
}

impl<T: Clone + Debug + Send + Sync + 'static> Discrete<T> for DiscreteEvent<T> {
    fn occurrences(&self) -> Vec<Occurrence<T>> {
        self.occurrences.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrete_event_from_vec() {
        let events = DiscreteEvent::from_vec(vec![(1000, "first"), (2000, "second")]);

        assert_eq!(events.occurrences().len(), 2);
        assert_eq!(events.occurrences()[0], (1000, "first"));
        assert_eq!(events.occurrences()[1], (2000, "second"));
    }

    #[test]
    fn test_discrete_event_sorted() {
        // Out of order input should be sorted
        let events = DiscreteEvent::from_vec(vec![(2000, "second"), (1000, "first")]);

        assert_eq!(events.occurrences()[0], (1000, "first"));
        assert_eq!(events.occurrences()[1], (2000, "second"));
    }

    #[test]
    fn test_discrete_event_empty() {
        let events: DiscreteEvent<i32> = DiscreteEvent::empty();
        assert_eq!(events.occurrences().len(), 0);
    }

    #[test]
    fn test_discrete_event_map() {
        let events = DiscreteEvent::from_vec(vec![(0, 1), (1, 2), (2, 3)]);

        let doubled = events.map(|x| x * 2);
        let occurrences = doubled.occurrences();

        assert_eq!(occurrences[0], (0, 2));
        assert_eq!(occurrences[1], (1, 4));
        assert_eq!(occurrences[2], (2, 6));
    }

    #[test]
    fn test_discrete_event_filter() {
        let events = DiscreteEvent::from_vec(vec![(0, 1), (1, 2), (2, 3), (3, 4)]);

        let evens = events.filter(|x| x % 2 == 0);
        let occurrences = evens.occurrences();

        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0], (1, 2));
        assert_eq!(occurrences[1], (3, 4));
    }

    #[test]
    fn test_discrete_event_fold() {
        let events = DiscreteEvent::from_vec(vec![(0, 1), (1, 2), (2, 3)]);

        let sum = events.fold(0, |acc, x| acc + x);
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_discrete_event_scan() {
        let events = DiscreteEvent::from_vec(vec![(0, 1), (1, 2), (2, 3)]);

        let running_sum = events.scan(0, |acc, x| acc + x);
        let occurrences = running_sum.occurrences();

        assert_eq!(occurrences[0], (0, 1));
        assert_eq!(occurrences[1], (1, 3));
        assert_eq!(occurrences[2], (2, 6));
    }

    #[test]
    fn test_discrete_event_take() {
        let events = DiscreteEvent::from_vec(vec![(0, 1), (1, 2), (2, 3)]);

        let first_two = events.take(2);
        assert_eq!(first_two.occurrences().len(), 2);
        assert_eq!(first_two.occurrences()[0], (0, 1));
        assert_eq!(first_two.occurrences()[1], (1, 2));
    }

    #[test]
    fn test_discrete_event_skip() {
        let events = DiscreteEvent::from_vec(vec![(0, 1), (1, 2), (2, 3)]);

        let last_one = events.skip(2);
        assert_eq!(last_one.occurrences().len(), 1);
        assert_eq!(last_one.occurrences()[0], (2, 3));
    }

    #[test]
    fn test_discrete_event_map_composition() {
        // Test Functor law: map f . map g = map (f . g)
        let events = DiscreteEvent::from_vec(vec![(0, 2)]);

        let result1 = events.clone().map(|x| x + 1).map(|x| x * 2);
        let result2 = events.map(|x| (x + 1) * 2);

        assert_eq!(result1.occurrences()[0].1, result2.occurrences()[0].1);
    }
}
