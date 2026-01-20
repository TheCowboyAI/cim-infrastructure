<!-- Copyright (c) 2025 - Cowboy AI, Inc. -->

# Migration Guide

## Migrating to CIM Infrastructure v0.1.0

This guide helps you migrate existing code to use the new pure functional patterns introduced in CIM Infrastructure v0.1.0.

**Key Changes**:
- Pure projection functions replacing mutable projections
- Explicit side effect handling
- FRP signal abstractions
- Property-based testing infrastructure

---

## Table of Contents

- [Overview](#overview)
- [Breaking Changes](#breaking-changes)
- [Migration Paths](#migration-paths)
  - [Projections](#migrating-projections)
  - [State Machines](#migrating-state-machines)
  - [Event Store](#migrating-event-store)
- [Step-by-Step Migration](#step-by-step-migration)
- [Testing Your Migration](#testing-your-migration)
- [Common Pitfalls](#common-pitfalls)

---

## Overview

Version 0.1.0 introduces a major architectural shift toward pure functional programming patterns. The key insight: **separate pure business logic from side effects**.

### Before (v0.0.x)

Projections mixed logic with I/O:

```rust
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

### After (v0.1.0)

Pure functions + explicit side effects:

```rust
fn my_projection(state: State, event: Event) -> (State, Vec<SideEffect>) {
    // Pure state update
    let new_state = State {
        count: state.count + 1,
    };

    // Side effects as data
    let effects = vec![
        SideEffect::DatabaseWrite {
            collection: "events".to_string(),
            data: serde_json::to_value(&event).unwrap(),
        }
    ];

    (new_state, effects)
}
```

---

## Breaking Changes

### 1. Projection API

**Old API** (removed):
```rust
#[async_trait]
trait ProjectionAdapter {
    async fn project(&mut self, event: Event) -> Result<(), Error>;
}
```

**New API**:
```rust
type PureProjection<S, E> = fn(S, E) -> (S, Vec<SideEffect>);

// Use with fold_projection or replay_projection
let (state, effects) = fold_projection(my_projection, initial_state, events);
```

**Migration**: Convert mutable projections to pure functions.

### 2. Side Effect Handling

**Old**: Side effects performed directly in projection logic

**New**: Side effects returned as data structures:
- `SideEffect::DatabaseWrite`
- `SideEffect::DatabaseUpdate`
- `SideEffect::DatabaseDelete`
- `SideEffect::Log`
- `SideEffect::EmitEvent`

**Migration**: Extract I/O code to executors.

### 3. State Machine Traits

**Old**: Basic state machine trait

**New**: Enhanced with:
- `StateMachineWithHistory` - tracks transitions
- `DeterministicFSM` - guarantees determinism
- `StateInvariant` - checks invariants

**Migration**: Implement new trait methods where needed.

---

## Migration Paths

### Migrating Projections

#### Step 1: Extract State

Convert projection struct to a separate state type.

**Before**:
```rust
struct MyProjection {
    database: Database,
    count: usize,
    last_event_id: Option<String>,
}
```

**After**:
```rust
// Separate state (must be Clone + Debug + Default)
#[derive(Clone, Debug, Default)]
struct MyProjectionState {
    count: usize,
    last_event_id: Option<String>,
}

// Executor holds connections
struct MyExecutor {
    database: Database,
}
```

#### Step 2: Convert to Pure Function

Transform `project` method into a pure function.

**Before**:
```rust
async fn project(&mut self, event: Event) -> Result<(), Error> {
    self.count += 1;
    self.last_event_id = Some(event.id.clone());
    self.database.write(&event).await?;
    Ok(())
}
```

**After**:
```rust
fn my_projection(
    state: MyProjectionState,
    event: Event,
) -> (MyProjectionState, Vec<SideEffect>) {
    // Pure state update
    let new_state = MyProjectionState {
        count: state.count + 1,
        last_event_id: Some(event.id.clone()),
    };

    // Side effects as data
    let effects = vec![
        SideEffect::DatabaseWrite {
            collection: "events".to_string(),
            data: serde_json::to_value(&event).unwrap(),
        }
    ];

    (new_state, effects)
}
```

#### Step 3: Implement Executor

Create an executor to perform the side effects.

```rust
use cim_infrastructure::projection::executor::*;

#[async_trait]
impl SideEffectExecutor for MyExecutor {
    async fn execute(&mut self, effects: Vec<SideEffect>) -> Result<(), ExecutorError> {
        for effect in effects {
            match effect {
                SideEffect::DatabaseWrite { collection, data } => {
                    self.database.write(&collection, data).await
                        .map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
                }
                // Handle other effects...
                _ => {}
            }
        }
        Ok(())
    }
}
```

#### Step 4: Wire It Together

Use `fold_projection` with your executor.

```rust
// Process events
let (final_state, effects) = fold_projection(
    my_projection,
    MyProjectionState::default(),
    events,
);

// Execute side effects
let mut executor = MyExecutor { database };
executor.execute(effects).await?;
```

### Migrating State Machines

State machines require minimal changes if you already used the `StateMachine` trait.

#### Before (if not using trait):
```rust
enum OrderState {
    Draft,
    Submitted,
}

impl OrderState {
    fn submit(&self) -> Result<OrderState, Error> {
        match self {
            OrderState::Draft => Ok(OrderState::Submitted),
            _ => Err(Error::InvalidTransition),
        }
    }
}
```

#### After:
```rust
use cim_infrastructure::state_machine::*;

#[derive(Clone, PartialEq, Eq)]
enum OrderState {
    Draft,
    Submitted,
}

impl StateMachine for OrderState {
    type Input = OrderEvent;
    type Output = ();

    fn transition(&self, input: &Self::Input) -> TransitionResult<(Self, Self::Output)> {
        match (self, input) {
            (Draft, Submit) => Ok((Submitted, ())),
            _ => Err(TransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", input),
            }),
        }
    }
}
```

#### Add History Tracking (Optional):
```rust
let mut fsm = StateMachineWithHistory::new(OrderState::Draft);
fsm.transition_with_history(Submit, Utc::now())?;
assert_eq!(*fsm.current_state(), OrderState::Submitted);
```

### Migrating Event Store

Event store API is mostly unchanged, but has some improvements:

#### Before (v0.0.x):
```rust
let store = JetStreamEventStore::new(client, "stream").await?;
store.append_event(aggregate_id, event).await?;
```

#### After (v0.1.0):
```rust
// Same API, enhanced with better error handling
let store = JetStreamEventStore::new(client, "stream").await?;
store.append_event(aggregate_id, event).await?;
```

**No migration needed** unless you were using internal APIs.

---

## Step-by-Step Migration

### Full Example: Migrating an Order Projection

#### 1. Original Code (v0.0.x)

```rust
struct OrderProjection {
    database: Database,
    total_orders: usize,
    total_revenue: Money,
}

#[async_trait]
impl ProjectionAdapter for OrderProjection {
    async fn project(&mut self, event: OrderEvent) -> Result<(), Error> {
        match event {
            OrderEvent::OrderCreated { order_id, amount } => {
                self.total_orders += 1;
                self.total_revenue += amount;

                self.database.insert("orders", json!({
                    "order_id": order_id,
                    "amount": amount,
                })).await?;
            }
        }
        Ok(())
    }
}
```

#### 2. Extract State

```rust
#[derive(Clone, Debug, Default)]
struct OrderProjectionState {
    total_orders: usize,
    total_revenue: Money,
}
```

#### 3. Create Pure Projection

```rust
fn order_projection(
    state: OrderProjectionState,
    event: OrderEvent,
) -> (OrderProjectionState, Vec<SideEffect>) {
    match event {
        OrderEvent::OrderCreated { order_id, amount } => {
            let new_state = OrderProjectionState {
                total_orders: state.total_orders + 1,
                total_revenue: state.total_revenue + amount,
            };

            let effects = vec![
                SideEffect::DatabaseWrite {
                    collection: "orders".to_string(),
                    data: json!({
                        "order_id": order_id,
                        "amount": amount,
                        "total_orders": new_state.total_orders,
                        "total_revenue": new_state.total_revenue,
                    }),
                },
                SideEffect::Log {
                    level: LogLevel::Info,
                    message: format!("Order {} created for {}", order_id, amount),
                },
            ];

            (new_state, effects)
        }
    }
}
```

#### 4. Create Executor

```rust
struct OrderExecutor {
    database: Database,
}

#[async_trait]
impl SideEffectExecutor for OrderExecutor {
    async fn execute(&mut self, effects: Vec<SideEffect>) -> Result<(), ExecutorError> {
        for effect in effects {
            match effect {
                SideEffect::DatabaseWrite { collection, data } => {
                    self.database.insert(&collection, data).await
                        .map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
                }
                SideEffect::Log { level, message } => {
                    match level {
                        LogLevel::Info => tracing::info!("{}", message),
                        LogLevel::Error => tracing::error!("{}", message),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
```

#### 5. Use in Application

```rust
// Load events
let events = event_store.read_events(aggregate_id).await?;

// Apply projection
let (final_state, effects) = fold_projection(
    order_projection,
    OrderProjectionState::default(),
    events,
);

// Execute side effects
let mut executor = OrderExecutor { database };
executor.execute(effects).await?;

// Use the final state
println!("Total orders: {}", final_state.total_orders);
println!("Total revenue: {}", final_state.total_revenue);
```

---

## Testing Your Migration

### 1. Unit Tests (Pure Functions)

Test projection logic without I/O:

```rust
#[test]
fn test_order_projection() {
    let state = OrderProjectionState::default();
    let event = OrderEvent::OrderCreated {
        order_id: "123".to_string(),
        amount: Money::dollars(100),
    };

    let (new_state, effects) = order_projection(state, event);

    assert_eq!(new_state.total_orders, 1);
    assert_eq!(new_state.total_revenue, Money::dollars(100));
    assert_eq!(effects.len(), 2); // DatabaseWrite + Log
}
```

### 2. Property Tests

Verify mathematical properties:

```rust
proptest! {
    #[test]
    fn prop_projection_deterministic(events in event_sequence()) {
        let (state1, _) = fold_projection(order_projection, initial(), events.clone());
        let (state2, _) = fold_projection(order_projection, initial(), events);
        prop_assert_eq!(state1, state2);
    }
}
```

### 3. Integration Tests

Test with real executor:

```rust
#[tokio::test]
async fn test_order_projection_integration() {
    let database = connect_database().await?;
    let mut executor = OrderExecutor { database };

    let events = vec![/* ... */];
    let (state, effects) = fold_projection(order_projection, initial(), events);

    executor.execute(effects).await?;

    // Verify database state
    let orders = database.query("SELECT COUNT(*) FROM orders").await?;
    assert_eq!(orders, state.total_orders);
}
```

---

## Common Pitfalls

### 1. Forgetting to Clone State

**Problem**:
```rust
fn bad_projection(mut state: State, event: Event) -> (State, Vec<SideEffect>) {
    state.count += 1; // Mutates input!
    (state, vec![])
}
```

**Solution**:
```rust
fn good_projection(state: State, event: Event) -> (State, Vec<SideEffect>) {
    let new_state = State {
        count: state.count + 1, // Creates new state
    };
    (new_state, vec![])
}
```

### 2. Performing Side Effects in Projection

**Problem**:
```rust
fn bad_projection(state: State, event: Event) -> (State, Vec<SideEffect>) {
    println!("Processing event"); // Side effect!
    database.write(event); // Side effect!
    (state, vec![])
}
```

**Solution**:
```rust
fn good_projection(state: State, event: Event) -> (State, Vec<SideEffect>) {
    let effects = vec![
        SideEffect::Log {
            level: LogLevel::Info,
            message: "Processing event".to_string(),
        },
        SideEffect::DatabaseWrite { /* ... */ },
    ];
    (state, effects)
}
```

### 3. Not Implementing ProjectionState Traits

**Problem**:
```rust
struct MyState {
    data: Vec<u8>, // Large data, expensive to clone
}
```

**Solution**:
```rust
#[derive(Clone, Debug, Default)]
struct MyState {
    data: Arc<Vec<u8>>, // Cheap to clone
}
```

### 4. Mixing Concerns in State

**Problem**:
```rust
#[derive(Clone, Debug, Default)]
struct MyState {
    count: usize,
    database: Arc<Database>, // Don't put connections in state!
}
```

**Solution**:
```rust
#[derive(Clone, Debug, Default)]
struct MyState {
    count: usize, // Only domain data
}

// Connections go in executor
struct MyExecutor {
    database: Database,
}
```

---

## Benefits After Migration

After migrating to pure functional patterns, you'll gain:

1. **Testability**: No mocks needed, pure functions easy to test
2. **Replay**: Trivial to rebuild projections from events
3. **Debugging**: Clear separation between logic and effects
4. **Composability**: Projections can be combined and transformed
5. **Mathematical Correctness**: Property tests prove correctness

---

## Getting Help

If you encounter issues during migration:

1. Check the [examples](../examples/) directory for complete working examples
2. Read the comprehensive guides:
   - [EVENT_SOURCING.md](EVENT_SOURCING.md)
   - [PURE_PROJECTIONS.md](PURE_PROJECTIONS.md)
   - [ARCHITECTURE.md](ARCHITECTURE.md)
3. Review the test suite for patterns and best practices
4. Open an issue on GitHub with your specific migration question

---

## Migration Checklist

Use this checklist to track your migration progress:

- [ ] Extract projection state into separate types
- [ ] Convert projections to pure functions
- [ ] Implement side effect executors
- [ ] Update state machines to use traits
- [ ] Add property-based tests
- [ ] Update integration tests
- [ ] Remove old mutable projection code
- [ ] Update documentation
- [ ] Test thoroughly in staging environment
- [ ] Deploy to production

---

**Last Updated**: 2026-01-19
**Target Version**: v0.1.0
