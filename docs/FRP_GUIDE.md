<!-- Copyright (c) 2025 - Cowboy AI, Inc. -->

# FRP (Functional Reactive Programming) Guide

## Overview

This guide explains how to use Functional Reactive Programming (FRP) abstractions in the CIM Infrastructure library. FRP provides a pure functional approach to modeling time-varying values and event streams.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Signal Types](#signal-types)
- [Common Patterns](#common-patterns)
- [Event Sourcing Integration](#event-sourcing-integration)
- [Best Practices](#best-practices)
- [Examples](#examples)

---

## Core Concepts

### What is FRP?

Functional Reactive Programming is a programming paradigm for working with time-varying values in a pure functional way. Instead of imperatively updating mutable state, FRP models values that change over time as first-class data structures.

### Key Benefits

1. **Referential Transparency**: No side effects, easier to reason about
2. **Composability**: Signals can be combined using pure functions
3. **Time as First-Class**: Explicit modeling of temporal relationships
4. **Type Safety**: Compiler-enforced correctness

### Core Abstractions

```text
Signal<T>           Base trait for all time-varying values
    ↓
    ├── Behavior<T>       Continuous-time (always has a value)
    └── DiscreteEvent<T>  Discrete-time (values at specific moments)
```

---

## Signal Types

### Signal<T> Trait

Base trait for all FRP types. Provides the fundamental `map` operation (Functor).

```rust
pub trait Signal<T>: Clone + Debug + Send + Sync {
    type Mapped<U>: Signal<U>;

    fn map<U, F>(self, f: F) -> Self::Mapped<U>
    where
        F: Fn(T) -> U + Clone + Send + Sync + 'static,
        U: Clone + Debug + Send + Sync + 'static;
}
```

#### Functor Laws

All Signal implementations must satisfy:

1. **Identity**: `signal.map(|x| x) == signal`
2. **Composition**: `signal.map(f).map(g) == signal.map(|x| g(f(x)))`

### Behavior<T> - Continuous-Time Signals

A `Behavior<T>` represents a value that exists at **all points in time**. You can sample it at any moment to get its current value.

```text
Time:  ──────────────────────────→
Value: ≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈
       (always has a value)
```

#### Mathematical Model

A Behavior is a function from Time to Value:

```text
Behavior<T> ≅ Time → T
```

#### Creating Behaviors

```rust
use cim_infrastructure::frp::Behavior;

// Constant behavior
let temperature = Behavior::constant(72);

// From a function
let counter = Behavior::from_fn(|| {
    // Custom sampling logic
    get_current_count()
});
```

#### Sampling Behaviors

```rust
use cim_infrastructure::frp::signal::Samplable;

let temperature = Behavior::constant(72);
let current_temp = temperature.sample(); // 72
```

#### Combining Behaviors

```rust
use cim_infrastructure::frp::combinators::*;

let x = Behavior::constant(3);
let y = Behavior::constant(4);

// Binary combination
let sum = apply2(x, y, |a, b| a + b);
assert_eq!(sum.sample(), 7);

// Ternary combination
let z = Behavior::constant(2);
let product = apply3(x, y, z, |a, b, c| a * b * c);
assert_eq!(product.sample(), 24);
```

### DiscreteEvent<T> - Discrete-Time Signals

A `DiscreteEvent<T>` represents values that occur at **specific moments** in time. It has occurrences, not continuous existence.

```text
Time:  ──────────────────────────→
Value:     ●     ●  ●       ●
       (values at specific times)
```

#### Mathematical Model

A DiscreteEvent is a finite list of occurrences:

```text
DiscreteEvent<T> ≅ [(Time, T)]
```

#### Creating DiscreteEvents

```rust
use cim_infrastructure::frp::DiscreteEvent;

let events = DiscreteEvent::from_vec(vec![
    (0, "first"),
    (1000, "second"),
    (2000, "third"),
]);

// Empty event stream
let empty = DiscreteEvent::<i32>::empty();
```

#### Accessing Occurrences

```rust
use cim_infrastructure::frp::signal::Discrete;

let events = DiscreteEvent::from_vec(vec![(0, 1), (1000, 2)]);
let occurrences = events.occurrences(); // Vec<(Time, T)>
```

---

## Common Patterns

### Pattern 1: Mapping Signals

Transform signal values using pure functions.

```rust
use cim_infrastructure::frp::{Behavior, DiscreteEvent};
use cim_infrastructure::frp::signal::Signal;

// Map behavior
let fahrenheit = Behavior::constant(72);
let celsius = fahrenheit.map(|f| (f - 32) * 5 / 9);

// Map discrete events
let numbers = DiscreteEvent::from_vec(vec![(0, 1), (1, 2), (2, 3)]);
let doubled = numbers.map(|x| x * 2);
```

### Pattern 2: Filtering Event Streams

Keep only occurrences matching a predicate.

```rust
let numbers = DiscreteEvent::from_vec(vec![
    (0, 1), (1, 2), (2, 3), (3, 4)
]);

let evens = numbers.filter(|x| x % 2 == 0);
// Result: [(1, 2), (3, 4)]
```

### Pattern 3: Folding Events

Reduce an event stream to a single value.

```rust
let numbers = DiscreteEvent::from_vec(vec![
    (0, 1), (1, 2), (2, 3)
]);

let sum = numbers.fold(0, |acc, x| acc + x);
assert_eq!(sum, 6);
```

### Pattern 4: Scanning (Running Accumulation)

Like fold, but emits intermediate results at each occurrence.

```rust
let numbers = DiscreteEvent::from_vec(vec![
    (0, 1), (1, 2), (2, 3)
]);

let running_sum = numbers.scan(0, |acc, x| acc + x);
// Results: (0, 1), (1, 3), (2, 6)
```

### Pattern 5: Merging Event Streams

Combine two event streams into one.

```rust
use cim_infrastructure::frp::combinators::merge;

let stream1 = DiscreteEvent::from_vec(vec![(0, "A"), (2000, "C")]);
let stream2 = DiscreteEvent::from_vec(vec![(1000, "B"), (3000, "D")]);

let merged = merge(stream1, stream2);
// Results: (0, "A"), (1000, "B"), (2000, "C"), (3000, "D")
```

### Pattern 6: Taking/Skipping Occurrences

```rust
let events = DiscreteEvent::from_vec(vec![
    (0, 1), (1, 2), (2, 3), (3, 4)
]);

let first_two = events.clone().take(2);  // (0, 1), (1, 2)
let last_two = events.skip(2);           // (2, 3), (3, 4)
```

---

## Event Sourcing Integration

FRP is a natural fit for event sourcing. Domain events are discrete-time signals, and aggregate state is a continuous-time signal derived from events.

### Events as DiscreteEvent<T>

Domain events naturally map to `DiscreteEvent`:

```rust
use cim_infrastructure::events::ComputeResourceEvent;
use cim_infrastructure::frp::DiscreteEvent;

let events = DiscreteEvent::from_vec(vec![
    (0, ComputeResourceEvent::ResourceRegistered(/* ... */)),
    (1000, ComputeResourceEvent::OrganizationAssigned(/* ... */)),
    (2000, ComputeResourceEvent::StatusChanged(/* ... */)),
]);
```

### State as Behavior<T>

Aggregate state is derived from events via folding:

```rust
use cim_infrastructure::aggregate::ComputeResourceState;
use cim_infrastructure::frp::Behavior;

// Fold events into state
let event_vec: Vec<ComputeResourceEvent> = events
    .occurrences()
    .into_iter()
    .map(|(_, event)| event)
    .collect();

let state = ComputeResourceState::from_events(&event_vec);
let state_behavior = Behavior::constant(state);

// Can sample state at any time
let current = state_behavior.sample();
```

### Projection as Event → SideEffect

Projections consume events and produce side effects:

```rust
// Projections subscribe to event streams and update read models
for (time, event) in events.occurrences() {
    match event {
        ComputeResourceEvent::ResourceRegistered(evt) => {
            // Update Neo4j projection
            update_graph(evt).await?;
        }
        // Handle other events...
    }
}
```

### Complete Event Sourcing Pattern

```text
┌─────────────────┐
│  Command        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Command Handler│  (Pure Function)
│  State + Cmd    │
│  → Result<E>    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Event          │  DiscreteEvent<E>
│  Append to      │
│  Event Store    │
└────────┬────────┘
         │
         ├──────────────────┐
         │                  │
         ▼                  ▼
┌─────────────────┐  ┌─────────────────┐
│  State Behavior │  │  Projections    │
│  fold events    │  │  (Side Effects) │
│  → State        │  │                 │
└─────────────────┘  └─────────────────┘
```

---

## Best Practices

### 1. Use Type Aliases for Domain Signals

```rust
pub type ResourceEventStream = DiscreteEvent<ComputeResourceEvent>;
pub type ResourceStateBehavior = Behavior<ComputeResourceState>;
```

### 2. Keep Signal Functions Pure

```rust
// ✅ GOOD: Pure function
let doubled = numbers.map(|x| x * 2);

// ❌ BAD: Side effect in map
let with_side_effect = numbers.map(|x| {
    println!("Processing: {}", x);  // Side effect!
    x * 2
});
```

### 3. Compose Small Functions

```rust
// ✅ GOOD: Composed transformations
let result = events
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .take(10);

// ❌ BAD: One large function
let result = events.filter(|x| {
    if x % 2 == 0 {
        // Large complex logic...
    }
});
```

### 4. Prefer Behaviors for Queries

If you need to query current state, use Behavior:

```rust
let state_behavior = Behavior::constant(current_state);

// Query specific aspects
let is_active = state_behavior.map(|s| s.is_active());
let hostname = state_behavior.map(|s| s.hostname().clone());
```

### 5. Use Scan for State Machines

When modeling state transitions, use `scan`:

```rust
let state_transitions = events.scan(initial_state, |state, event| {
    state.apply(event)
});
```

---

## Examples

### Example 1: Temperature Monitoring

```rust
use cim_infrastructure::frp::*;

// Current temperature (continuous)
let temperature_f = Behavior::constant(72);

// Convert to Celsius
let temperature_c = temperature_f.map(|f| (f - 32) * 5 / 9);

// Check if comfortable
let is_comfortable = temperature_f.map(|t| t >= 65 && t <= 75);

println!("Temperature: {}°F ({}°C)",
    temperature_f.sample(),
    temperature_c.sample()
);
println!("Comfortable: {}", is_comfortable.sample());
```

### Example 2: Event Stream Processing

```rust
use cim_infrastructure::frp::*;

let events = DiscreteEvent::from_vec(vec![
    (0, "ResourceRegistered"),
    (1000, "OrganizationAssigned"),
    (2000, "StatusChanged"),
]);

// Count events
let count = events.clone().fold(0, |acc, _| acc + 1);

// Filter long event names
let long_names = events.filter(|name| name.len() > 18);

// Map to lengths
let lengths = events.map(|name| name.len());
```

### Example 3: Running Totals

```rust
use cim_infrastructure::frp::*;

let purchases = DiscreteEvent::from_vec(vec![
    (0, 10.0),      // $10 purchase
    (100, 25.50),   // $25.50 purchase
    (200, 5.00),    // $5 purchase
]);

// Running total
let running_total = purchases.scan(0.0, |total, amount| total + amount);

for (time, total) in running_total.occurrences() {
    println!("t={}ms: Total spent: ${:.2}", time, total);
}
```

### Example 4: Combining Multiple Sources

```rust
use cim_infrastructure::frp::combinators::*;

let cpu_usage = Behavior::constant(45);
let memory_usage = Behavior::constant(60);
let disk_usage = Behavior::constant(30);

let health_score = apply3(cpu_usage, memory_usage, disk_usage, |cpu, mem, disk| {
    let avg = (cpu + mem + disk) / 3;
    if avg < 50 { "Healthy" }
    else if avg < 75 { "Warning" }
    else { "Critical" }
});

println!("System health: {}", health_score.sample());
```

---

## Theoretical Foundations

### Category Theory

FRP signals form a Functor:

```text
Signal<T> ───map(f)───> Signal<U>
    │                       │
    │                       │
    ▼                       ▼
    T ────────f────────>    U
```

Laws:
- **Identity**: `fmap id = id`
- **Composition**: `fmap (g ∘ f) = fmap g ∘ fmap f`

### Time Semantics

- **Behavior**: `B(t) ∈ T → A` (total function)
- **Event**: `E(t) ∈ Time → Maybe A` (partial function)

---

## Further Reading

- [Original FRP Paper](http://conal.net/papers/icfp97/) by Conal Elliott
- [Push-Pull FRP](http://conal.net/papers/push-pull-frp/) implementation
- [The Essence of FRP](http://conal.net/papers/essence-of-frp/) theoretical foundations

---

**Last Updated**: 2026-01-19
