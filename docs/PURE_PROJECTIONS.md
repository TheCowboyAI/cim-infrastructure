<!-- Copyright (c) 2025 - Cowboy AI, Inc. -->

# Pure Projections Guide

## Overview

This guide explains the pure functional approach to event projections in CIM Infrastructure. Pure projections separate projection logic from side effects, making projections testable, composable, and easy to replay.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Architecture](#architecture)
- [Pure Projection Functions](#pure-projection-functions)
- [Side Effect Executors](#side-effect-executors)
- [Projection Replay](#projection-replay)
- [Testing Projections](#testing-projections)
- [Migration from Mutable Projections](#migration-from-mutable-projections)
- [Best Practices](#best-practices)

---

## Core Concepts

### What are Pure Projections?

Pure projections are functions that:
1. Take current state and an event as input
2. Return new state and a list of side effects
3. Have no side effects themselves (no I/O, no mutation)
4. Are referentially transparent (same inputs → same outputs)

### Traditional vs Pure Approach

**Traditional (Mutable) Projection:**
```rust
async fn project(&mut self, event: Event) -> Result<(), Error> {
    // Mutates internal state
    self.state.counter += 1;

    // Performs I/O directly
    self.database.write(event).await?;

    Ok(())
}
```

**Pure Projection:**
```rust
fn project(state: State, event: Event) -> (State, Vec<SideEffect>) {
    // Returns new state (no mutation)
    let new_state = State { counter: state.counter + 1 };

    // Returns side effects as data (no I/O)
    let effects = vec![
        SideEffect::DatabaseWrite { /* ... */ }
    ];

    (new_state, effects)
}
```

### Key Benefits

1. **Testability**: Pure functions easy to test (no mocks needed)
2. **Replay**: Trivial to rebuild projections from event history
3. **Composability**: Multiple projections can be composed
4. **Time Travel**: Can project to any point in event history
5. **Debugging**: Clear separation between logic and effects

---

## Architecture

### Component Overview

```text
┌─────────────────────────────────────────────────────────┐
│                    Event Stream                          │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│            Pure Projection Function                      │
│                                                          │
│  fn project(state: S, event: E) -> (S, Vec<Effect>)    │
│                                                          │
│  • Takes state + event                                   │
│  • Returns new state + effects                           │
│  • NO side effects!                                      │
└─────────────────┬────────────────┬──────────────────────┘
                  │                │
                  │                │
        ┌─────────▼─────┐   ┌──────▼──────┐
        │   New State    │   │   Effects   │
        │   (in memory)  │   │  (as data)  │
        └────────────────┘   └──────┬──────┘
                                    │
                                    ▼
                      ┌──────────────────────────┐
                      │   Side Effect Executor   │
                      │                          │
                      │   Interprets effects     │
                      │   Performs actual I/O    │
                      └──────────┬───────────────┘
                                 │
                                 ▼
                      ┌──────────────────────────┐
                      │      Database/I/O        │
                      └──────────────────────────┘
```

### Type Flow

```rust
Event                          // Input
    ↓
(State, Event)                // Projection input
    ↓
fn project(S, E) → (S, Vec<Effect>)  // Pure transformation
    ↓
(New State, Effects)          // Projection output
    ↓
Executor::execute(Effects)    // Side effect interpretation
    ↓
Database Updated              // Final result
```

---

## Pure Projection Functions

### Basic Structure

```rust
use cim_infrastructure::projection::pure::*;

#[derive(Clone, Debug, Default)]
struct MyProjectionState {
    // Projection-specific state
    event_count: usize,
    last_event_id: Option<String>,
}

fn my_projection(
    state: MyProjectionState,
    event: MyEvent,
) -> (MyProjectionState, Vec<SideEffect>) {
    // 1. Update state (pure logic)
    let new_state = MyProjectionState {
        event_count: state.event_count + 1,
        last_event_id: Some(event.id.clone()),
    };

    // 2. Build side effects (as data)
    let effects = vec![
        SideEffect::DatabaseWrite {
            collection: "my_events".to_string(),
            data: serde_json::json!({
                "event_id": event.id,
                "event_type": event.event_type,
                "timestamp": event.timestamp,
            }),
        },
        SideEffect::Log {
            level: LogLevel::Info,
            message: format!("Processed event {}", event.id),
        },
    ];

    // 3. Return both
    (new_state, effects)
}
```

### State Requirements

Projection state must implement `ProjectionState` trait:

```rust
pub trait ProjectionState: Clone + Debug + Default {}
```

This requires:
- `Clone`: For functional updates
- `Debug`: For logging and debugging
- `Default`: For initialization

Any type satisfying these bounds automatically implements `ProjectionState`.

### Side Effect Types

```rust
pub enum SideEffect {
    /// Write data to database
    DatabaseWrite {
        collection: String,
        data: Value,
    },

    /// Update existing record
    DatabaseUpdate {
        collection: String,
        id: String,
        updates: Value,
    },

    /// Delete from database
    DatabaseDelete {
        collection: String,
        id: String,
    },

    /// Execute database query
    DatabaseQuery {
        query: String,
        params: Vec<Value>,
    },

    /// Log a message
    Log {
        level: LogLevel,
        message: String,
    },

    /// Emit a derived event
    EmitEvent {
        event_type: String,
        data: Value,
    },
}
```

---

## Side Effect Executors

### Executor Trait

```rust
#[async_trait]
pub trait SideEffectExecutor: Send + Sync {
    async fn execute(&mut self, effects: Vec<SideEffect>)
        -> Result<(), ExecutorError>;
}
```

### Built-in Executors

#### LoggingExecutor

Logs effects without executing them (useful for debugging):

```rust
use cim_infrastructure::projection::executor::*;

let mut executor = LoggingExecutor::new();
executor.execute(effects).await?;

// Check what was logged
for effect in executor.effects() {
    println!("Effect: {:?}", effect);
}
```

#### NullExecutor

Discards all effects (useful when you only care about state):

```rust
let mut executor = NullExecutor::new();
executor.execute(effects).await?;  // Does nothing
```

#### CollectingExecutor

Collects effects for later batch execution:

```rust
let mut executor = CollectingExecutor::new();

// Process multiple events
for event in events {
    let (state, effects) = my_projection(state, event);
    executor.execute(effects).await?;
}

// Execute all collected effects at once
let all_effects = executor.take_effects();
real_executor.execute(all_effects).await?;
```

#### FilteringExecutor

Filters effects before executing:

```rust
// Only execute database writes
let mut executor = FilteringExecutor::new(
    LoggingExecutor::new(),
    |effect| matches!(effect, SideEffect::DatabaseWrite { .. })
);

executor.execute(effects).await?;
```

### Custom Executors

Implement `SideEffectExecutor` for custom backends:

```rust
struct Neo4jExecutor {
    driver: Arc<neo4rs::Graph>,
}

#[async_trait]
impl SideEffectExecutor for Neo4jExecutor {
    async fn execute(&mut self, effects: Vec<SideEffect>)
        -> Result<(), ExecutorError> {
        for effect in effects {
            match effect {
                SideEffect::DatabaseWrite { collection, data } => {
                    // Execute Cypher query
                    let query = format!(
                        "CREATE (n:{} $data)",
                        collection
                    );
                    self.driver.execute(
                        neo4rs::query(&query)
                            .param("data", data)
                    ).await?;
                }
                // Handle other effects...
                _ => {}
            }
        }
        Ok(())
    }
}
```

---

## Projection Replay

### Why Replay?

Projections can get out of sync with the event stream due to:
- Bugs in projection logic
- Database failures mid-projection
- Schema changes requiring rebuild
- Adding new projections to existing events

Pure projections make replay trivial.

### Replaying Events

```rust
use cim_infrastructure::projection::pure::*;

// Get all events from event store
let events: Vec<MyEvent> = event_store.read_all_events().await?;

// Replay through projection
let (final_state, all_effects) = replay_projection(
    my_projection,
    MyProjectionState::default(),
    events,
);

// Execute all effects
executor.execute(all_effects).await?;
```

### Incremental Replay

Resume from last known state:

```rust
// Load checkpoint
let last_state = checkpoint_store.load().await?;
let last_event_number = last_state.last_event_number;

// Get events since checkpoint
let new_events = event_store
    .read_events_from(last_event_number + 1)
    .await?;

// Replay only new events
let (updated_state, effects) = replay_projection(
    my_projection,
    last_state,
    new_events,
);

// Execute and checkpoint
executor.execute(effects).await?;
checkpoint_store.save(updated_state).await?;
```

### Time Travel

Project to any point in event history:

```rust
// Get events up to a specific time
let events_until = event_store
    .read_events_until(target_timestamp)
    .await?;

// Project to that point in time
let (state_at_time, _) = fold_projection(
    my_projection,
    MyProjectionState::default(),
    events_until,
);

println!("State at {}: {:?}", target_timestamp, state_at_time);
```

---

## Testing Projections

### Pure Function Testing

No mocks or async runtime needed:

```rust
#[test]
fn test_projection_logic() {
    let state = MyProjectionState::default();
    let event = MyEvent { id: "123".to_string() };

    let (new_state, effects) = my_projection(state, event);

    assert_eq!(new_state.event_count, 1);
    assert_eq!(new_state.last_event_id, Some("123".to_string()));
    assert_eq!(effects.len(), 2);
}
```

### Testing Side Effects

Verify correct effects are produced:

```rust
#[test]
fn test_projection_produces_write() {
    let state = MyProjectionState::default();
    let event = MyEvent { id: "123".to_string() };

    let (_state, effects) = my_projection(state, event);

    // Check a DatabaseWrite effect was produced
    assert!(effects.iter().any(|effect| {
        matches!(effect, SideEffect::DatabaseWrite { .. })
    }));
}
```

### Testing with Logging Executor

Verify effects are correct without performing I/O:

```rust
#[tokio::test]
async fn test_projection_with_executor() {
    let mut executor = LoggingExecutor::new();
    let state = MyProjectionState::default();
    let event = MyEvent { id: "123".to_string() };

    let (_state, effects) = my_projection(state, event);
    executor.execute(effects).await.unwrap();

    assert_eq!(executor.effects().len(), 2);
}
```

### Property-Based Testing

Test projection properties with QuickCheck:

```rust
#[quickcheck]
fn prop_projection_always_increments_count(events: Vec<MyEvent>) -> bool {
    let (final_state, _) = fold_projection(
        my_projection,
        MyProjectionState::default(),
        events.clone(),
    );

    final_state.event_count == events.len()
}
```

---

## Migration from Mutable Projections

### Before (Mutable)

```rust
struct MyProjection {
    database: Database,
    state: MyState,
}

#[async_trait]
impl ProjectionAdapter for MyProjection {
    async fn project(&mut self, event: Event) -> Result<(), Error> {
        // Mutates self
        self.state.count += 1;

        // Performs I/O directly
        self.database.write(event).await?;

        Ok(())
    }
}
```

### After (Pure)

```rust
// 1. Extract state to separate type
#[derive(Clone, Debug, Default)]
struct MyProjectionState {
    count: usize,
}

// 2. Make projection a pure function
fn my_projection(
    state: MyProjectionState,
    event: Event,
) -> (MyProjectionState, Vec<SideEffect>) {
    // Pure state update
    let new_state = MyProjectionState {
        count: state.count + 1,
    };

    // Return side effects as data
    let effects = vec![
        SideEffect::DatabaseWrite {
            collection: "events".to_string(),
            data: event.to_json(),
        }
    ];

    (new_state, effects)
}

// 3. Create executor for I/O
struct MyExecutor {
    database: Database,
}

#[async_trait]
impl SideEffectExecutor for MyExecutor {
    async fn execute(&mut self, effects: Vec<SideEffect>)
        -> Result<(), ExecutorError> {
        for effect in effects {
            match effect {
                SideEffect::DatabaseWrite { data, .. } => {
                    self.database.write(data).await?;
                }
                // Handle other effects...
                _ => {}
            }
        }
        Ok(())
    }
}
```

---

## Best Practices

### 1. Keep Projection Functions Pure

```rust
// ✅ GOOD: Pure function
fn projection(state: State, event: Event) -> (State, Vec<SideEffect>) {
    let new_state = state.update(&event);
    let effects = vec![/* side effects */];
    (new_state, effects)
}

// ❌ BAD: Side effects in projection
fn projection(state: State, event: Event) -> (State, Vec<SideEffect>) {
    println!("Processing event");  // Side effect!
    database.write(event);          // Side effect!
    (state, vec![])
}
```

### 2. Make State Clonable and Lightweight

```rust
// ✅ GOOD: Lightweight state
#[derive(Clone, Debug, Default)]
struct State {
    counter: usize,
    last_id: Option<String>,
}

// ❌ BAD: Heavy state with connections
#[derive(Clone)]  // Expensive clone!
struct State {
    database: Arc<Database>,  // Don't put connections in state
    large_cache: HashMap<String, Vec<u8>>,  // Expensive to clone
}
```

### 3. Use Appropriate Executors

```rust
// Development: Log effects
let executor = LoggingExecutor::new();

// Testing: Collect effects for assertions
let executor = CollectingExecutor::new();

// Production: Real database executor
let executor = Neo4jExecutor::new(driver);
```

### 4. Batch Side Effects

```rust
// ✅ GOOD: Single DatabaseWrite with all data
let effects = vec![
    SideEffect::DatabaseWrite {
        collection: "events".to_string(),
        data: json!({
            "events": all_events,
        }),
    }
];

// ❌ BAD: Many individual writes
let effects: Vec<_> = all_events.iter().map(|event| {
    SideEffect::DatabaseWrite {
        collection: "events".to_string(),
        data: json!(event),
    }
}).collect();
```

### 5. Checkpoint Regularly

```rust
// Process events in batches and checkpoint
for chunk in events.chunks(100) {
    let (state, effects) = fold_projection(projection, state, chunk.to_vec());
    executor.execute(effects).await?;
    checkpoint_store.save(&state).await?;
}
```

---

## Advanced Patterns

### Composing Projections

```rust
fn composed_projection(
    state: (State1, State2),
    event: Event,
) -> ((State1, State2), Vec<SideEffect>) {
    let (state1, state2) = state;

    let (new_state1, effects1) = projection1(state1, event.clone());
    let (new_state2, effects2) = projection2(state2, event);

    let mut all_effects = effects1;
    all_effects.extend(effects2);

    ((new_state1, new_state2), all_effects)
}
```

### Conditional Projection

```rust
fn conditional_projection(
    state: State,
    event: Event,
) -> (State, Vec<SideEffect>) {
    match event.event_type {
        EventType::Create => create_projection(state, event),
        EventType::Update => update_projection(state, event),
        EventType::Delete => delete_projection(state, event),
    }
}
```

---

**Last Updated**: 2026-01-19
