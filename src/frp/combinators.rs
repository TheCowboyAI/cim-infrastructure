// Copyright (c) 2025 - Cowboy AI, Inc.
//! Signal Combinators
//!
//! This module provides combinator functions for composing and transforming signals.
//! All combinators are pure functions that create new signals without side effects.
//!
//! # Available Combinators
//!
//! ## For All Signals
//! - `constant` - Create a behavior with a fixed value
//! - `map` - Transform signal values (Functor)
//!
//! ## For Behaviors
//! - `apply2` - Combine two behaviors with a binary function
//! - `apply3` - Combine three behaviors with a ternary function
//!
//! ## For DiscreteEvents
//! - `filter` - Keep only occurrences matching a predicate
//! - `fold` - Reduce event stream to a single value
//! - `scan` - Accumulate values over time (like fold but emits intermediate results)
//! - `merge` - Combine two event streams
//!
//! # Examples
//!
//! ## Combining Behaviors
//!
//! ```rust,ignore
//! use cim_infrastructure::frp::combinators::*;
//!
//! let x = Behavior::constant(3);
//! let y = Behavior::constant(4);
//!
//! let sum = apply2(x, y, |a, b| a + b);
//! assert_eq!(sum.sample(), 7);
//! ```
//!
//! ## Event Stream Processing
//!
//! ```rust,ignore
//! let events = DiscreteEvent::from_vec(vec![
//!     (0, 1),
//!     (1, 2),
//!     (2, 3),
//! ]);
//!
//! let doubled = events.map(|x| x * 2);
//! let evens = doubled.filter(|x| x % 2 == 0);
//! ```

use super::behavior::Behavior;
use super::event::DiscreteEvent;
use super::signal::{Discrete, Samplable};
use std::fmt::Debug;

/// Combine two behaviors using a binary function
///
/// This is a convenience wrapper around `Behavior::apply2`.
///
/// # Arguments
///
/// * `a` - First behavior
/// * `b` - Second behavior
/// * `f` - Function to combine values
///
/// # Examples
///
/// ```rust,ignore
/// use cim_infrastructure::frp::combinators::*;
///
/// let x = Behavior::constant(3);
/// let y = Behavior::constant(4);
/// let sum = apply2(x, y, |a, b| a + b);
/// assert_eq!(sum.sample(), 7);
/// ```
pub fn apply2<T, U, V, F>(a: Behavior<T>, b: Behavior<U>, f: F) -> Behavior<V>
where
    T: Clone + Debug + Send + Sync + 'static,
    U: Clone + Debug + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
    F: Fn(T, U) -> V + Send + Sync + 'static,
{
    a.apply2(b, f)
}

/// Combine three behaviors using a ternary function
///
/// # Arguments
///
/// * `a` - First behavior
/// * `b` - Second behavior
/// * `c` - Third behavior
/// * `f` - Function to combine values
///
/// # Examples
///
/// ```rust,ignore
/// use cim_infrastructure::frp::combinators::*;
///
/// let x = Behavior::constant(1);
/// let y = Behavior::constant(2);
/// let z = Behavior::constant(3);
/// let sum = apply3(x, y, z, |a, b, c| a + b + c);
/// assert_eq!(sum.sample(), 6);
/// ```
pub fn apply3<T, U, V, W, F>(
    a: Behavior<T>,
    b: Behavior<U>,
    c: Behavior<V>,
    f: F,
) -> Behavior<W>
where
    T: Clone + Debug + Send + Sync + 'static,
    U: Clone + Debug + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
    W: Clone + Debug + Send + Sync + 'static,
    F: Fn(T, U, V) -> W + Send + Sync + 'static,
{
    let combined_ab = a.apply2(b, |x, y| (x, y));
    combined_ab.apply2(c, move |(x, y), z| f(x, y, z))
}

/// Merge two discrete event streams
///
/// Combines two event streams into a single stream containing all occurrences
/// from both streams, sorted by time.
///
/// # Arguments
///
/// * `a` - First event stream
/// * `b` - Second event stream
///
/// # Examples
///
/// ```rust,ignore
/// use cim_infrastructure::frp::combinators::*;
///
/// let events1 = DiscreteEvent::from_vec(vec![(0, "a"), (2, "c")]);
/// let events2 = DiscreteEvent::from_vec(vec![(1, "b"), (3, "d")]);
///
/// let merged = merge(events1, events2);
/// // Results: (0, "a"), (1, "b"), (2, "c"), (3, "d")
/// ```
pub fn merge<T>(a: DiscreteEvent<T>, b: DiscreteEvent<T>) -> DiscreteEvent<T>
where
    T: Clone + Debug + Send + Sync + 'static,
{
    let mut occurrences = a.occurrences();
    occurrences.extend(b.occurrences());
    occurrences.sort_by_key(|(time, _)| *time);

    DiscreteEvent::from_vec(occurrences)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply2() {
        let x = Behavior::constant(3);
        let y = Behavior::constant(4);

        let sum = apply2(x, y, |a, b| a + b);
        assert_eq!(sum.sample(), 7);
    }

    #[test]
    fn test_apply3() {
        let x = Behavior::constant(1);
        let y = Behavior::constant(2);
        let z = Behavior::constant(3);

        let sum = apply3(x, y, z, |a, b, c| a + b + c);
        assert_eq!(sum.sample(), 6);
    }

    #[test]
    fn test_merge() {
        let events1 = DiscreteEvent::from_vec(vec![(0, "a"), (2, "c")]);
        let events2 = DiscreteEvent::from_vec(vec![(1, "b"), (3, "d")]);

        let merged = merge(events1, events2);
        let occurrences = merged.occurrences();

        assert_eq!(occurrences.len(), 4);
        assert_eq!(occurrences[0], (0, "a"));
        assert_eq!(occurrences[1], (1, "b"));
        assert_eq!(occurrences[2], (2, "c"));
        assert_eq!(occurrences[3], (3, "d"));
    }

    #[test]
    fn test_merge_overlapping_times() {
        let events1 = DiscreteEvent::from_vec(vec![(0, 1), (2, 3)]);
        let events2 = DiscreteEvent::from_vec(vec![(2, 10), (3, 11)]);

        let merged = merge(events1, events2);
        let occurrences = merged.occurrences();

        assert_eq!(occurrences.len(), 4);
        // Both events at time 2 should be present
        assert!(occurrences.iter().any(|&(t, v)| t == 2 && v == 3));
        assert!(occurrences.iter().any(|&(t, v)| t == 2 && v == 10));
    }
}
