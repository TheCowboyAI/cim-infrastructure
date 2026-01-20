// Copyright (c) 2025 - Cowboy AI, Inc.
//! FRP Infrastructure Example
//!
//! This example demonstrates how to use FRP (Functional Reactive Programming)
//! abstractions with simple event streams and behaviors.
//!
//! # Key Concepts
//!
//! 1. **DiscreteEvent<T>** - Values that occur at specific moments in time
//! 2. **Behavior<T>** - Values that exist at all points in time
//! 3. **Signal Operations** - map, filter, fold for composing signals
//!
//! # Example Flow
//!
//! ```text
//! Event Stream (Discrete)     Behavior (Continuous)
//! ─────────────────────      ────────────────────
//!      ●                            ≈≈≈≈≈
//!      |                            |
//!      ├─> Event 1         ──>      State 1
//!      |                            |
//!      ●                            ≈≈≈≈≈
//!      |                            |
//!      ├─> Event 2         ──>      State 2
//!      |                            |
//!      ●                            ≈≈≈≈≈
//!      |                            |
//!      └─> Event 3         ──>      State 3
//! ```

use cim_infrastructure::frp::{Behavior, DiscreteEvent};
use cim_infrastructure::frp::signal::{Discrete, Samplable, Signal};
use cim_infrastructure::frp::combinators::{apply2, apply3, merge};

fn main() {
    println!("=== FRP Infrastructure Example ===\n");

    // === Example 1: Discrete Events ===
    println!("=== Example 1: Discrete Events ===");

    let events = DiscreteEvent::from_vec(vec![
        (0, "Resource Registered"),
        (1000, "Organization Assigned"),
        (2000, "Status Changed to Active"),
    ]);

    println!("Event stream with {} occurrences", events.occurrences().len());

    // Map events to their lengths
    let event_lengths = events.clone().map(|s| s.len());
    println!("Event lengths: {:?}", event_lengths.occurrences());

    // Filter to only long events
    let long_events = events.filter(|s| s.len() > 20);
    println!("Long events: {:?}", long_events.occurrences());

    // Fold events into a count
    let event_count = DiscreteEvent::from_vec(vec![(0, 1), (1, 1), (2, 1)]);
    let total_count = event_count.fold(0, |acc, x| acc + x);
    println!("Total event count: {}\n", total_count);

    // === Example 2: Behaviors (Continuous-Time Signals) ===
    println!("=== Example 2: Behaviors ===");

    let temperature = Behavior::constant(72);
    println!("Current temperature: {}°F", temperature.sample());

    // Map behavior to Celsius
    let temperature_c = temperature.clone().map(|f| (f - 32) * 5 / 9);
    println!("Temperature in Celsius: {}°C", temperature_c.sample());

    // Combine multiple behaviors
    let humidity = Behavior::constant(45);
    let comfort_index = apply2(temperature, humidity, |temp, hum| {
        if temp >= 65 && temp <= 75 && hum >= 30 && hum <= 60 {
            "Comfortable"
        } else {
            "Uncomfortable"
        }
    });
    println!("Comfort index: {}\n", comfort_index.sample());

    // === Example 3: Combining Three Behaviors ===
    println!("=== Example 3: Combining Multiple Behaviors ===");

    let x = Behavior::constant(10);
    let y = Behavior::constant(20);
    let z = Behavior::constant(30);

    let sum = apply3(x, y, z, |a, b, c| a + b + c);
    println!("Sum of three values: {}\n", sum.sample());

    // === Example 4: Merging Event Streams ===
    println!("=== Example 4: Merging Event Streams ===");

    let stream1 = DiscreteEvent::from_vec(vec![
        (0, "A"),
        (2000, "C"),
    ]);

    let stream2 = DiscreteEvent::from_vec(vec![
        (1000, "B"),
        (3000, "D"),
    ]);

    let merged = merge(stream1, stream2);
    println!("Merged stream occurrences:");
    for (time, value) in merged.occurrences() {
        println!("  t={:4}ms: {}", time, value);
    }
    println!();

    // === Example 5: Scan (Running Accumulation) ===
    println!("=== Example 5: Scan Operation ===");

    let numbers = DiscreteEvent::from_vec(vec![
        (0, 1),
        (100, 2),
        (200, 3),
        (300, 4),
    ]);

    let running_sum = numbers.scan(0, |acc, x| acc + x);
    println!("Running sum:");
    for (time, value) in running_sum.occurrences() {
        println!("  t={:3}ms: sum={}", time, value);
    }
    println!();

    // === Example 6: Event Stream Derivation ===
    println!("=== Example 6: Deriving State from Events ===");

    // Simulate a series of state-changing events
    let state_events = DiscreteEvent::from_vec(vec![
        (0, 1),      // Initial state: 1
        (100, 2),    // Add 2, state becomes 3
        (200, 5),    // Add 5, state becomes 8
        (300, -3),   // Add -3, state becomes 5
    ]);

    // Scan creates a behavior-like stream of accumulated state
    let state_history = state_events.scan(0, |state, delta| state + delta);

    println!("State evolution:");
    for (time, state) in state_history.occurrences() {
        println!("  t={:3}ms: state={}", time, state);
    }

    // Final state as a behavior
    let final_state = state_history.fold(0, |_, state| state);
    let state_behavior = Behavior::constant(final_state);
    println!("Final state: {}", state_behavior.sample());

    println!("\n=== Example Complete ===");
}
