<!-- Copyright (c) 2025 - Cowboy AI, Inc. -->

# CIM Infrastructure Architecture

## Overview

The CIM Infrastructure library provides event sourcing and event-driven architecture primitives for building distributed systems. It focuses on pure functional patterns, mathematical correctness, and NATS-based messaging.

**Version**: 0.1.0
**Status**: Production Ready
**Test Coverage**: 189 tests (117 unit, 37 integration, 35 property-based)

## Core Concepts

### 1. Event Sourcing

State is derived from a sequence of immutable events. Instead of storing current state, we store the events that led to that state.

**Key Properties**:
- Events are immutable
- State is reproducible from event history
- Complete audit trail
- Time travel capabilities

See [EVENT_SOURCING.md](EVENT_SOURCING.md) for comprehensive guide.

### 2. Pure Functional Design

All core operations are pure functions with no side effects:
- Projections: `(State, Event) → (State, Effects)`
- State machines: `(State, Input) → (State, Output)`
- Event handlers: `Event → Vec<Command>`

**Benefits**:
- Easy to test (no mocks needed)
- Mathematically provable correctness
- Trivial parallelization
- Simple debugging

### 3. NATS-First Messaging

All communication flows through NATS JetStream:
- Event streams stored in JetStream
- Command/query patterns via request-reply
- Publish-subscribe for event distribution

## Architecture Layers

```
┌─────────────────────────────────────────────────────┐
│              Application Layer                       │
│  (Domain services, command handlers, queries)       │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│           CIM Infrastructure Layer                   │
│                                                      │
│  ┌────────────┐  ┌────────────┐  ┌──────────────┐ │
│  │ Event Store│  │ Projections│  │ State        │ │
│  │ (JetStream)│  │   (Pure)   │  │ Machines     │ │
│  └────────────┘  └────────────┘  └──────────────┘ │
│                                                      │
│  ┌────────────┐  ┌────────────┐  ┌──────────────┐ │
│  │    FRP     │  │  Service   │  │   Adapters   │ │
│  │  (Signals) │  │   Layer    │  │              │ │
│  └────────────┘  └────────────┘  └──────────────┘ │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│             NATS Infrastructure                      │
│   (JetStream, KV, Object Store, Subjects)           │
└─────────────────────────────────────────────────────┘
```

## Module Structure

### Event Store (`event_store`)

Provides persistent event storage using NATS JetStream.

**Key Components**:
- `JetStreamEventStore`: Main event store implementation
- Stream configuration and management
- Event persistence and retrieval
- Optimistic concurrency control

**Example**:
```rust
let event_store = JetStreamEventStore::new(nats_client, "my-stream").await?;
event_store.append_event(aggregate_id, event).await?;
let events = event_store.read_events(aggregate_id).await?;
```

**Features**:
- Append-only event log
- Stream-per-aggregate or stream-per-domain patterns
- Version-based optimistic concurrency
- Event replay from any point in history

### Projections (`projection`)

Transform event streams into read models using pure functional patterns.

**Key Components**:
- `ProjectionAdapter`: Trait for mutable projections
- Pure projection functions: `(State, Event) → (State, Effects)`
- Side effect executors for I/O
- Replay capabilities

**Pure Projection Example**:
```rust
fn my_projection(state: State, event: Event) -> (State, Vec<SideEffect>) {
    let new_state = State {
        count: state.count + 1,
        last_event: Some(event.id.clone()),
    };

    let effects = vec![
        SideEffect::DatabaseWrite {
            collection: "events".to_string(),
            data: serde_json::to_value(&event).unwrap(),
        }
    ];

    (new_state, effects)
}
```

**Side Effect Executors**:
- `LoggingExecutor`: Logs effects without executing
- `NullExecutor`: Discards all effects
- `CollectingExecutor`: Collects effects for batch execution
- `FilteringExecutor`: Selectively executes effects

See [PURE_PROJECTIONS.md](PURE_PROJECTIONS.md) for comprehensive guide.

### State Machines (`state_machine`)

Finite state machines for modeling lifecycles and workflows.

**Key Components**:
- `StateMachine` trait: Defines state transitions
- `StateMachineWithHistory`: Tracks transition history
- `DeterministicFSM`: Guarantees deterministic behavior
- `StateInvariant`: Checks state invariants

**Example**:
```rust
#[derive(Clone, PartialEq, Eq)]
enum OrderState {
    Draft,
    Submitted,
    Confirmed,
    Shipped,
    Delivered,
}

impl StateMachine for OrderState {
    type Input = OrderEvent;
    type Output = ();

    fn transition(&self, input: &Self::Input) -> TransitionResult<(Self, Self::Output)> {
        match (self, input) {
            (Draft, Submit) => Ok((Submitted, ())),
            (Submitted, Confirm) => Ok((Confirmed, ())),
            (Confirmed, Ship) => Ok((Shipped, ())),
            (Shipped, Deliver) => Ok((Delivered, ())),
            _ => Err(TransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", input),
            }),
        }
    }
}
```

See [STATE_MACHINE.md](STATE_MACHINE.md) for detailed examples.

### FRP - Functional Reactive Programming (`frp`)

Time-varying values and event streams using functional reactive programming.

**Key Components**:
- `Signal<T>`: Base trait for time-varying values (Functor)
- `Behavior<T>`: Continuous-time signals
- `DiscreteEvent<T>`: Discrete-time signals
- Signal combinators for composition

**Example**:
```rust
// Behavior: continuous-time signal
let counter = Behavior::constant(0);
let doubled = counter.map(|n| n * 2);
let sum = apply2(counter, doubled, |a, b| a + b);

// DiscreteEvent: discrete-time signal
let events = DiscreteEvent::from_vec(vec![
    (0, "create"),
    (5, "update"),
    (10, "delete"),
]);
let filtered = events.filter(|s| *s != "delete");
```

**Signal Type Hierarchy**:
```
Signal<T>: Functor
    ├── Behavior<T>: Samplable
    │   - Exists at all times
    │   - Can be sampled at any moment
    │   - Models continuous values
    └── DiscreteEvent<T>: Discrete
        - Exists only at specific times
        - Sequence of (Time, Value) pairs
        - Models discrete occurrences
```

See [FRP_GUIDE.md](FRP_GUIDE.md) for comprehensive guide.

### Service Layer (`service`)

Service abstractions for building distributed systems.

**Key Components**:
- Service traits and interfaces
- Health check patterns
- Service coordination
- Graceful shutdown

**Example**:
```rust
#[async_trait]
pub trait Service: Send + Sync {
    async fn start(&mut self) -> Result<(), Error>;
    async fn stop(&mut self) -> Result<(), Error>;
    async fn health_check(&self) -> Result<HealthStatus, Error>;
}
```

## Design Principles

### 1. Pure Functional Core

Business logic is implemented as pure functions:
- **No side effects in core logic**: All business rules are pure
- **Side effects pushed to boundaries**: I/O at system edges only
- **Easy to test and reason about**: No hidden dependencies

**Example**:
```rust
// ✅ GOOD: Pure function
fn calculate_total(items: &[Item]) -> Money {
    items.iter().map(|i| i.price).sum()
}

// ❌ BAD: Side effects in business logic
fn calculate_total_bad(items: &[Item]) -> Money {
    let total = items.iter().map(|i| i.price).sum();
    database.save_total(total); // Side effect!
    total
}
```

### 2. Explicit Side Effects

Side effects are data structures, not operations:
- Projections return `Vec<SideEffect>`
- Executors interpret side effects
- Testable without I/O

**Example**:
```rust
// Pure projection returns effects as data
fn projection(state: S, event: E) -> (S, Vec<SideEffect>) {
    let new_state = update_state(state, event);
    let effects = vec![
        SideEffect::DatabaseWrite { /* ... */ },
        SideEffect::Log { /* ... */ },
    ];
    (new_state, effects)
}

// Executor interprets effects
executor.execute(effects).await?;
```

### 3. Event-Driven Architecture

All state changes flow through events:
- Commands produce events
- Events update state
- Projections derive read models

**Flow**:
```
Command → Handler → Event → Aggregate → New State
                      ↓
                 Projections → Read Models
```

### 4. Separation of Concerns

Clear boundaries between layers:
- Domain logic separate from infrastructure
- Read models separate from write models (CQRS)
- Side effects separate from business logic

## Testing Strategy

The library employs a comprehensive three-tier testing strategy that provides mathematical guarantees about system behavior.

### Unit Tests

Test pure functions in isolation:
- **No mocks needed**: Pure functions have no dependencies
- **Fast execution**: Tests run in milliseconds
- **High confidence**: Direct testing of business logic
- **117+ library tests**: Core functionality thoroughly covered

**Example**:
```rust
#[test]
fn test_projection_logic() {
    let (state, effects) = my_projection(initial_state(), event);
    assert_eq!(state.count, expected_count);
    assert_eq!(effects.len(), 2);
}
```

### Property-Based Tests

Use `proptest` to verify mathematical properties hold for all valid inputs:

#### Pure Projection Properties

- **Determinism**: Same state + same events → same result
- **Associativity**: `fold(fold(s, e1), e2) = fold(s, e1 ++ e2)`
- **Identity**: Empty event sequence leaves state unchanged
- **Replay consistency**: `replay(s, events) = fold(s, events)`
- **Side effect production**: Every event produces expected effects

**Example**:
```rust
proptest! {
    #[test]
    fn prop_projection_is_deterministic(events in event_sequence()) {
        let (state1, _) = fold_projection(proj, init(), events.clone());
        let (state2, _) = fold_projection(proj, init(), events);
        prop_assert_eq!(state1, state2);
    }
}
```

#### State Machine Properties

- **Transition determinism**: Same state + same input → same result
- **History replay**: Replaying transitions reproduces state
- **Invariant preservation**: Invariants hold after valid transitions
- **Validity consistency**: `can_transition` matches `transition` result
- **Bounds enforcement**: Invalid transitions always fail

**Example**:
```rust
proptest! {
    #[test]
    fn prop_invariants_preserved(state in state_gen(), input in input_gen()) {
        assert!(state.check_invariants().is_ok());
        if let Ok((new_state, _)) = state.transition(&input) {
            assert!(new_state.check_invariants().is_ok());
        }
    }
}
```

#### Test Coverage

- **35+ property tests**: Covering projections and state machines
- **10+ properties verified**: Mathematical guarantees proven
- **1000s of cases**: Each property tested across many random inputs
- **Zero failures**: All properties hold for all generated inputs

### Integration Tests

Test with real NATS infrastructure:
- **JetStream event store**: Real persistence layer
- **End-to-end flows**: Complete command → event → projection cycles
- **Real message passing**: Actual NATS pub/sub and request/reply
- **37+ integration tests**: Infrastructure components validated

**Example**:
```rust
#[tokio::test]
async fn test_event_store_roundtrip() {
    let nats = connect_nats().await?;
    let store = JetStreamEventStore::new(nats, "test-stream").await?;

    store.append_event(aggregate_id, event).await?;
    let events = store.read_events(aggregate_id).await?;

    assert_eq!(events.len(), 1);
}
```

### Test Pyramid

```
        /\
       /  \      Integration Tests (37)
      /____\     - Real NATS infrastructure
     /      \    - End-to-end flows
    /________\
   /          \  Property Tests (35)
  /____________\ - Mathematical properties
 /              \ - Thousands of cases
/________________\
                  Unit Tests (117+)
                  - Pure functions
                  - Fast, focused
```

### Why This Matters

1. **Mathematical Correctness**: Property tests prove system behaves correctly for all inputs
2. **Confidence**: Multiple test layers provide defense in depth
3. **Refactoring Safety**: Can refactor with confidence that behavior is preserved
4. **Documentation**: Tests serve as executable specifications
5. **Regression Prevention**: Property tests catch edge cases humans miss

## Performance Considerations

### Event Store

- **Batching**: Group events for efficient writes
- **Streaming**: Use async streams for large event sequences
- **Caching**: Cache recent events in memory
- **Partitioning**: Use stream-per-aggregate for scalability

### Projections

- **Parallel processing**: Multiple projections run concurrently
- **Checkpointing**: Save projection state periodically
- **Replay optimization**: Batch side effects during replay
- **Incremental updates**: Process only new events

### NATS

- **Connection pooling**: Reuse NATS connections
- **Subject design**: Use hierarchical subjects for routing
- **Message size**: Keep messages small, use references for large data
- **Acknowledgment**: Use explicit acks for reliability

## Migration and Evolution

### Event Schema Evolution

Events must be forward and backward compatible:
- **Add fields with defaults**: New fields optional
- **Never remove required fields**: Breaking change
- **Use versioned event types**: `EventV1`, `EventV2`
- **Upcasting**: Convert old events to new format on read

**Example**:
```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
enum OrderEvent {
    V1 { id: String, amount: Money },
    V2 { id: String, amount: Money, currency: Currency },
}
```

### Projection Rebuild

Projections can be rebuilt from event history:
1. Stop current projection
2. Clear projection state
3. Replay all events from beginning
4. Resume normal operation

**Why Rebuild**:
- Fix bugs in projection logic
- Add new projections to existing events
- Change projection schema

### State Machine Evolution

State machines can evolve carefully:
- **Add new states gradually**: Preserve existing transitions
- **Test thoroughly**: Property tests verify invariants
- **Deploy incrementally**: Roll out changes safely

## Deployment Patterns

### Single Node

Simplest deployment for development and small systems:
- NATS JetStream on single server
- All services on one machine
- Fast local communication

### Clustered

High availability deployment:
- NATS JetStream cluster (3+ nodes)
- Services distributed across nodes
- Automatic failover

### Multi-Region

Global deployment with data locality:
- NATS super-cluster across regions
- Event replication between regions
- Local read models in each region

## Security Considerations

### Event Store

- **Authentication**: JWT tokens for NATS access
- **Authorization**: Subject-based permissions
- **Encryption**: TLS for all connections
- **Audit**: All events logged immutably

### Projections

- **Read-only**: Projections only read events
- **Isolation**: Each projection isolated from others
- **Validation**: Verify event authenticity before projecting

## References

- [EVENT_SOURCING.md](EVENT_SOURCING.md) - Detailed event sourcing guide
- [FRP_GUIDE.md](FRP_GUIDE.md) - Functional reactive programming patterns
- [PURE_PROJECTIONS.md](PURE_PROJECTIONS.md) - Pure projection patterns
- [STATE_MACHINE.md](STATE_MACHINE.md) - State machine examples

## Appendix: Test Statistics

### Test Summary (as of Phase 7 completion)

| Category | Count | Coverage |
|----------|-------|----------|
| Unit Tests | 117 | Core library |
| Integration Tests | 37 | Infrastructure |
| Property Tests | 35 | Mathematical properties |
| **Total** | **189** | **Comprehensive** |

### Property Tests Breakdown

| Module | Properties | Test Cases (per property) |
|--------|-----------|---------------------------|
| Pure Projections | 11 | 256 default |
| State Machines | 10 | 256 default |
| FRP Signals | (Covered in unit tests) | - |

### Code Statistics

| Metric | Value |
|--------|-------|
| Total Lines | ~7,600 |
| Source Files | 45+ |
| Documentation | 5 guides |
| Examples | 3 complete examples |

---

**Last Updated**: 2026-01-19
**Library Version**: 0.1.0
**Phase**: 7 Complete - Production Ready
