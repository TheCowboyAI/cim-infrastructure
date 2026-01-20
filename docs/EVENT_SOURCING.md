<!-- Copyright (c) 2025 - Cowboy AI, Inc. -->

# Event Sourcing Guide

## Overview

This guide provides a comprehensive introduction to event sourcing as implemented in CIM Infrastructure. It covers core concepts, patterns, and best practices for building event-sourced systems.

## Table of Contents

- [What is Event Sourcing?](#what-is-event-sourcing)
- [Core Concepts](#core-concepts)
- [Architecture](#architecture)
- [Implementation Patterns](#implementation-patterns)
- [Command Handling](#command-handling)
- [Event Application](#event-application)
- [Projections](#projections)
- [Testing](#testing)
- [Best Practices](#best-practices)
- [Common Pitfalls](#common-pitfalls)

---

## What is Event Sourcing?

Event sourcing is a pattern where **state changes are stored as a sequence of events** rather than updating current state directly. Instead of CRUD operations, we append immutable events to an event log.

### Traditional State Storage

```text
Database: { id: 123, status: "active", count: 5 }
         ↓ UPDATE
Database: { id: 123, status: "inactive", count: 5 }
         (previous state lost)
```

### Event Sourcing

```text
Event Log:
  1. ResourceRegistered { id: 123 }
  2. StatusChanged { from: "provisioning", to: "active" }
  3. CounterIncremented { delta: 5 }
  4. StatusChanged { from: "active", to: "inactive" }

Current State = fold(initial_state, events)
```

### Key Benefits

1. **Complete Audit Trail**: Every state change is recorded
2. **Time Travel**: Reconstruct state at any point in history
3. **Replay**: Rebuild projections from events
4. **Event-Driven**: Natural fit for event-driven architectures
5. **Debugging**: Can replay events to reproduce bugs

---

## Core Concepts

### Events

**Events are facts that happened**. They are:
- **Immutable**: Once written, never changed
- **Past tense**: Named as facts (e.g., `ResourceRegistered`, not `RegisterResource`)
- **Ordered**: Sequential in the event stream
- **Timestamped**: Record when they occurred

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRegistered {
    pub event_version: u32,
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub hostname: Hostname,
    pub resource_type: ResourceType,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}
```

### Commands

**Commands are intentions to change state**. They:
- **Can fail**: Validation may reject them
- **Imperative**: Named as actions (e.g., `RegisterResource`)
- **Include timestamp**: Passed explicitly (no `Utc::now()` in domain logic)

```rust
pub struct RegisterResourceCommand {
    pub aggregate_id: Uuid,
    pub hostname: Hostname,
    pub resource_type: ResourceType,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
}
```

### Aggregates

**Aggregates are consistency boundaries**. They:
- **Enforce invariants**: Business rules that must hold
- **Process commands**: Decide if command should produce events
- **Apply events**: Derive current state from event history
- **Pure functions**: No side effects, referentially transparent

```rust
#[derive(Clone, Debug)]
pub struct ComputeResourceState {
    aggregate_id: Option<Uuid>,
    hostname: Option<Hostname>,
    status: Option<ResourceStatus>,
    // ... other fields
}

impl ComputeResourceState {
    /// Apply an event to produce new state (pure function)
    pub fn apply_event(&self, event: &ComputeResourceEvent) -> Self {
        match event {
            ComputeResourceEvent::ResourceRegistered(evt) => {
                // Return new state
                Self {
                    aggregate_id: Some(evt.aggregate_id),
                    hostname: Some(evt.hostname.clone()),
                    // ...
                }
            }
            // Handle other events...
        }
    }

    /// Reconstruct state from event history
    pub fn from_events(events: &[ComputeResourceEvent]) -> Self {
        events.iter().fold(Self::default(), |state, event| {
            state.apply_event(event)
        })
    }
}
```

---

## Architecture

### Complete Event Sourcing Flow

```text
┌──────────────┐
│   Command    │  (Intent to change state)
└──────┬───────┘
       │
       ▼
┌──────────────────────┐
│  Command Handler     │  (Pure Function)
│  State + Command     │
│  → Result<Event, E>  │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│  Event               │  (Fact that happened)
│  Append to           │
│  Event Store         │
└──────┬───────────────┘
       │
       ├─────────────────────────┐
       │                         │
       ▼                         ▼
┌──────────────┐         ┌──────────────┐
│  Aggregate   │         │  Projections │
│  State       │         │  (Read Model)│
│  fold events │         │              │
└──────────────┘         └──────────────┘
```

### Component Responsibilities

| Component | Responsibility | Mutable? |
|-----------|---------------|----------|
| Command | Express intent | No |
| Command Handler | Validate & decide | No |
| Event | Record fact | No |
| Aggregate | Enforce invariants | No |
| Event Store | Persist events | Yes (append-only) |
| Projection | Build read model | Yes (derived from events) |

---

## Implementation Patterns

### Pattern 1: Command Handler

Command handlers are **pure functions** that:
1. Take current state and a command
2. Validate business rules
3. Return either an event (success) or error (failure)

```rust
pub fn handle_register_resource(
    state: &ComputeResourceState,
    command: RegisterResourceCommand,
) -> Result<ResourceRegistered, CommandError> {
    // 1. Check preconditions
    if state.is_initialized() {
        return Err(CommandError::AlreadyInitialized);
    }

    // 2. Validate business rules
    validate_hostname(&command.hostname)?;

    // 3. Create event (fact)
    Ok(ResourceRegistered {
        event_version: CURRENT_VERSION,
        event_id: Uuid::now_v7(),
        aggregate_id: command.aggregate_id,
        timestamp: command.timestamp,  // From command, not Utc::now()!
        hostname: command.hostname,
    resource_type: command.resource_type,
        correlation_id: command.correlation_id,
        causation_id: None,
    })
}
```

### Pattern 2: Event Application

Event application is a **pure function** that:
1. Takes current state and an event
2. Derives new state from the event
3. Returns new state (no mutation)

```rust
impl ComputeResourceState {
    pub fn apply_event(&self, event: &ComputeResourceEvent) -> Self {
        match event {
            ComputeResourceEvent::ResourceRegistered(evt) => {
                Self {
                    aggregate_id: Some(evt.aggregate_id),
                    hostname: Some(evt.hostname.clone()),
                    resource_type: Some(evt.resource_type.clone()),
                    status: Some(ResourceStatus::Provisioning),
                    // ... other fields
                    ..*self  // Copy unchanged fields
                }
            }
            ComputeResourceEvent::StatusChanged(evt) => {
                Self {
                    status: Some(evt.to_status),
                    ..*self
                }
            }
            // Handle all event types...
        }
    }
}
```

### Pattern 3: Service Layer

Services orchestrate the command → event → projection flow:

```rust
impl EventSourcedComputeResourceService {
    pub async fn register_resource(
        &self,
        command: RegisterResourceCommand,
    ) -> ServiceResult<Uuid> {
        let aggregate_id = command.aggregate_id;

        // 1. Load current state
        let state = self.load_state(aggregate_id).await?;

        // 2. Handle command (pure function)
        let event = handle_register_resource(&state, command)?;

        // 3. Get version for concurrency control
        let version = self.event_store
            .get_version(aggregate_id)
            .await?
            .unwrap_or(0);

        // 4. Append event to store
        self.event_store.append(
            aggregate_id,
            vec![InfrastructureEvent::ComputeResource(
                ComputeResourceEvent::ResourceRegistered(event)
            )],
            Some(version),
        ).await?;

        // 5. Publish to NATS for projections
        self.publish_event(&event).await?;

        Ok(aggregate_id)
    }
}
```

---

## Command Handling

### Command Validation

Commands should be validated **before** creating events:

```rust
pub fn handle_activate_resource(
    state: &ComputeResourceState,
    command: ActivateResourceCommand,
) -> Result<StatusChanged, CommandError> {
    // 1. Validate state
    if !state.is_initialized() {
        return Err(CommandError::NotInitialized);
    }

    // 2. Validate business rules
    validate_activation_preconditions(
        state.status(),
        state.organization_id().is_some(),
        state.location_id().is_some(),
    )?;

    // 3. Validate state transition
    let current_status = state.status()
        .ok_or(CommandError::NotInitialized)?;

    if !current_status.can_transition_to(&ResourceStatus::Active) {
        return Err(CommandError::InvalidStateTransition {
            from: current_status,
            to: ResourceStatus::Active,
        });
    }

    // 4. Create event
    Ok(StatusChanged {
        event_version: CURRENT_VERSION,
        event_id: Uuid::now_v7(),
        aggregate_id: command.aggregate_id,
        timestamp: command.timestamp,
        from_status: current_status,
        to_status: ResourceStatus::Active,
        correlation_id: command.correlation_id,
        causation_id: Some(command.correlation_id),
    })
}
```

### Idempotency

Handle duplicate commands gracefully:

```rust
pub fn handle_add_policy(
    state: &ComputeResourceState,
    command: AddPolicyCommand,
) -> Result<PolicyAdded, CommandError> {
    // Check if policy already exists (idempotent)
    if state.policies().contains(&command.policy_id) {
        return Err(CommandError::PolicyAlreadyExists(command.policy_id));
    }

    // ... create event
}
```

---

## Event Application

### Fold Pattern

State is reconstructed by folding events:

```rust
pub fn from_events(events: &[ComputeResourceEvent]) -> Self {
    events.iter().fold(
        Self::default(),  // Initial state
        |state, event| state.apply_event(event)  // Apply each event
    )
}
```

### Event Application Rules

1. **Always succeed**: Event application cannot fail (events are facts)
2. **Idempotent**: Applying same event twice produces same result
3. **Pure**: No side effects, deterministic

```rust
// ✅ GOOD: Pure event application
pub fn apply_event(&self, event: &Event) -> Self {
    match event {
        Event::CounterIncremented(evt) => {
            Self { count: self.count + evt.delta }
        }
    }
}

// ❌ BAD: Side effects in event application
pub fn apply_event(&mut self, event: &Event) {
    match event {
        Event::CounterIncremented(evt) => {
            println!("Incrementing!");  // Side effect!
            self.database.write(evt);    // Side effect!
            self.count += evt.delta;
        }
    }
}
```

---

## Projections

### Read Models

Projections transform events into queryable read models:

```rust
fn neo4j_projection(
    state: ProjectionState,
    event: ComputeResourceEvent,
) -> (ProjectionState, Vec<SideEffect>) {
    let effects = match event {
        ComputeResourceEvent::ResourceRegistered(evt) => {
            vec![SideEffect::DatabaseWrite {
                collection: "ComputeResource".to_string(),
                data: json!({
                    "id": evt.aggregate_id,
                    "hostname": evt.hostname,
                    "type": evt.resource_type,
                }),
            }]
        }
        // Handle other events...
    };

    (state, effects)
}
```

### Projection Replay

Rebuild projections from scratch:

```rust
// Get all events
let events = event_store.read_all_events().await?;

// Replay through projection
let (final_state, effects) = replay_projection(
    my_projection,
    ProjectionState::default(),
    events,
);

// Execute effects
executor.execute(effects).await?;
```

### Multiple Projections

Same events can power multiple projections:

```text
Event Stream
     │
     ├──> Neo4j Projection (graph queries)
     ├──> Elasticsearch (text search)
     ├──> Redis (caching)
     └──> Metrics (monitoring)
```

---

## Testing

### Unit Tests for Command Handlers

```rust
#[test]
fn test_register_resource() {
    let state = ComputeResourceState::default();
    let command = RegisterResourceCommand {
        aggregate_id: Uuid::now_v7(),
        hostname: Hostname::new("server-01").unwrap(),
        resource_type: ResourceType::Server,
        timestamp: Utc::now(),
        correlation_id: Uuid::now_v7(),
    };

    let event = handle_register_resource(&state, command).unwrap();

    assert_eq!(event.hostname.as_str(), "server-01");
}
```

### Unit Tests for Event Application

```rust
#[test]
fn test_apply_registered_event() {
    let state = ComputeResourceState::default();
    let event = ResourceRegistered {
        aggregate_id: Uuid::now_v7(),
        hostname: Hostname::new("server-01").unwrap(),
        // ... other fields
    };

    let new_state = state.apply_event(
        &ComputeResourceEvent::ResourceRegistered(event)
    );

    assert!(new_state.is_initialized());
    assert_eq!(new_state.hostname().unwrap().as_str(), "server-01");
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_complete_workflow() {
    let service = create_test_service().await;

    // Register resource
    let id = service.register_resource(register_cmd).await.unwrap();

    // Assign organization
    service.assign_organization(id, assign_org_cmd).await.unwrap();

    // Activate
    service.activate(id, activate_cmd).await.unwrap();

    // Query state
    let state = service.get_resource(id).await.unwrap();
    assert_eq!(state.status(), Some(ResourceStatus::Active));
}
```

---

## Best Practices

### 1. Events are Immutable Facts

```rust
// ✅ GOOD: Events are facts (past tense)
pub struct ResourceRegistered { /* ... */ }
pub struct StatusChanged { /* ... */ }

// ❌ BAD: Commands (not events)
pub struct RegisterResource { /* ... */ }
pub struct ChangeStatus { /* ... */ }
```

### 2. No Business Logic in Events

```rust
// ✅ GOOD: Events are data
pub struct StatusChanged {
    pub from_status: ResourceStatus,
    pub to_status: ResourceStatus,
}

// ❌ BAD: Logic in events
impl StatusChanged {
    pub fn is_valid(&self) -> bool {
        self.from_status.can_transition_to(&self.to_status)
    }
}
```

### 3. Command Handlers are Pure

```rust
// ✅ GOOD: Pure function
pub fn handle_command(
    state: &State,
    command: Command,
) -> Result<Event, Error> {
    // Pure validation logic
    Ok(event)
}

// ❌ BAD: Side effects
pub fn handle_command(
    state: &State,
    command: Command,
) -> Result<Event, Error> {
    database.write(command)?;  // Side effect!
    Ok(event)
}
```

### 4. Event Versioning

Support schema evolution with versioning:

```rust
pub struct ResourceRegistered {
    pub event_version: u32,  // Schema version
    // ... fields
}

pub const CURRENT_VERSION: u32 = 1;

// When adding fields:
// 1. Increment CURRENT_VERSION
// 2. Make new fields Optional
// 3. Provide upcaster for old versions
```

### 5. Correlation and Causation IDs

Track event relationships:

```rust
pub struct Event {
    pub correlation_id: Uuid,      // Original command that started this flow
    pub causation_id: Option<Uuid>, // Immediate cause (previous event)
}
```

---

## Common Pitfalls

### Pitfall 1: Using `Utc::now()` in Domain Logic

```rust
// ❌ BAD: Timestamp generated in domain
pub fn handle_command(state: &State) -> Event {
    Event {
        timestamp: Utc::now(),  // Non-deterministic!
        // ...
    }
}

// ✅ GOOD: Timestamp from command
pub fn handle_command(state: &State, command: Command) -> Event {
    Event {
        timestamp: command.timestamp,  // Deterministic
        // ...
    }
}
```

### Pitfall 2: Mutable Aggregates

```rust
// ❌ BAD: Mutable aggregate
pub fn apply_event(&mut self, event: Event) {
    self.count += 1;  // Mutation!
}

// ✅ GOOD: Immutable aggregate
pub fn apply_event(&self, event: Event) -> Self {
    Self { count: self.count + 1 }  // New instance
}
```

### Pitfall 3: Validation in Event Application

```rust
// ❌ BAD: Validation in event application
pub fn apply_event(&self, event: Event) -> Result<Self, Error> {
    if event.is_valid() {  // Events are always valid!
        Ok(/* ... */)
    } else {
        Err(/* ... */)
    }
}

// ✅ GOOD: Events always apply successfully
pub fn apply_event(&self, event: Event) -> Self {
    // Just apply the event (it's a fact!)
    Self { /* ... */ }
}
```

### Pitfall 4: Deleting Events

```rust
// ❌ BAD: Deleting events
event_store.delete(event_id)?;

// ✅ GOOD: Compensating events
let compensation_event = ResourceDeleted {
    aggregate_id,
    timestamp: Utc::now(),
    reason: "User requested deletion",
};
event_store.append(compensation_event)?;
```

---

## Further Reading

- Martin Fowler: [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
- Greg Young: [CQRS Documents](https://cqrs.files.wordpress.com/2010/11/cqrs_documents.pdf)
- Vernon, Vaughn: *Implementing Domain-Driven Design*

---

**Last Updated**: 2026-01-19
