<!-- Copyright (c) 2025 - Cowboy AI, Inc. -->

# CIM Infrastructure: Improvement Roadmap

**Current Version**: v0.1.0
**Target Version**: v0.2.0
**Assessment Date**: 2026-01-19
**Overall Grade**: A- (92/100)

---

## Quick Reference

### Compliance Status

| Axiom | Status | Score | Priority |
|-------|--------|-------|----------|
| UUID v7 | ✅ Perfect | 100/100 | - |
| Event Sourcing | ✅ Excellent | 95/100 | P1 |
| Type Safety | ✅ Excellent | 95/100 | P2 |
| Immutability | ✅ Excellent | 95/100 | P1 |
| Domain Boundaries | ✅ Strong | 90/100 | - |
| NATS Integration | ✅ Strong | 90/100 | P2 |
| FRP Axioms | ⚠️ Good | 85/100 | P2 |

**Legend**: P1 = Critical, P2 = Important, P3 = Nice to Have

---

## Priority 1: Critical (Address Before v0.2.0)

### 1. Implement Pure Event-Sourced Aggregates

**Current Issue:**
```rust
// ❌ Current: Direct mutation
impl ComputeResource {
    pub fn set_organization(&mut self, org_id: EntityId<Organization>) {
        self.organization_id = Some(org_id);
        self.updated_at = Utc::now();  // Direct mutation
    }
}
```

**Desired Pattern:**
```rust
// ✅ Target: Event emission
impl ComputeResource {
    /// Assign resource to organization (command)
    pub fn assign_to_organization(
        &self,
        org_id: EntityId<Organization>,
    ) -> Result<OrganizationAssigned, ComputeResourceError> {
        // Validate business rules
        if self.organization_id.is_some() {
            return Err(ComputeResourceError::AlreadyAssigned);
        }

        // Emit event (immutable)
        Ok(OrganizationAssigned {
            event_id: Uuid::now_v7(),
            aggregate_id: self.id.into(),
            organization_id: org_id,
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            timestamp: Utc::now(),
        })
    }

    /// Apply event to produce new state
    pub fn apply(self, event: ComputeResourceEvent) -> Result<Self, ComputeResourceError> {
        match event {
            ComputeResourceEvent::OrganizationAssigned(e) => {
                Ok(Self {
                    organization_id: Some(e.organization_id),
                    ..self  // Immutable update
                })
            }
            // ... other events
        }
    }
}
```

**Files to Modify:**
- `src/domain/compute_resource.rs` - Add event types and apply method
- `src/events/compute_resource.rs` - Create new file for events
- `tests/domain/compute_resource_events.rs` - Test event application

**Estimated Effort**: 2-3 days

**Benefits**:
- True immutability
- Full event history
- Time travel debugging
- Perfect audit trail

---

### 2. Add Event Versioning Infrastructure

**Current Issue:**
```rust
// ❌ No versioning
pub struct OrganizationCreatedEvent {
    pub organization_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    // Missing version field
}
```

**Desired Pattern:**
```rust
// ✅ Versioned events
pub struct OrganizationCreatedEvent {
    pub event_version: u32,  // Schema version
    pub organization_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl Versioned for OrganizationCreatedEvent {
    const CURRENT_VERSION: u32 = 2;

    fn upcast(from_version: u32, data: serde_json::Value) -> Result<Self, EventError> {
        match from_version {
            1 => {
                // Migrate v1 → v2
                let v1: OrganizationCreatedEventV1 = serde_json::from_value(data)?;
                Ok(v1.into())  // Convert
            }
            2 => serde_json::from_value(data),
            _ => Err(EventError::UnknownVersion(from_version)),
        }
    }
}
```

**Files to Create:**
- `src/events/versioning.rs` - Versioning infrastructure
- `src/events/migrations/` - Event migration directory
- `tests/events/version_migrations.rs` - Migration tests

**Estimated Effort**: 1-2 days

**Benefits**:
- Schema evolution support
- Backward compatibility
- Safe system upgrades
- Migration testing

---

## Priority 2: Important (Address in v0.2.x)

### 3. Introduce FRP Signal Abstractions

**Current Issue:**
- Events are discrete, but infrastructure state is continuous
- No clear separation between Behaviors and Events
- Missing signal composition operators

**Desired Pattern:**
```rust
/// Continuous-time behavior (always has a value)
pub struct Behavior<T> {
    initial: T,
    changes: Stream<Event<T>>,
}

/// Discrete event in time
pub struct Event<T> {
    timestamp: DateTime<Utc>,
    value: T,
}

/// Signal combinators
impl<T> Behavior<T> {
    /// Map over behavior values
    pub fn map<U, F>(self, f: F) -> Behavior<U>
    where
        F: Fn(T) -> U + 'static,
    {
        Behavior {
            initial: f(self.initial),
            changes: self.changes.map(move |e| Event {
                timestamp: e.timestamp,
                value: f(e.value),
            }),
        }
    }

    /// Switch behaviors based on events
    pub fn switch<F>(self, switcher: Event<F>) -> Behavior<T>
    where
        F: Fn(T) -> T,
    {
        // Implementation
    }
}

// Example usage:
let resource_state: Behavior<ComputeResource> =
    Behavior::from_events(compute_events);

let reliability: Behavior<f64> =
    resource_state.map(|r| r.calculate_reliability());
```

**Files to Create:**
- `src/frp/behavior.rs` - Behavior type
- `src/frp/event.rs` - Event type (rename existing)
- `src/frp/signal.rs` - Signal trait
- `src/frp/combinators.rs` - Signal operators
- `examples/frp_infrastructure.rs` - Usage examples

**Estimated Effort**: 3-4 days

**Benefits**:
- True FRP semantics
- Continuous-time reasoning
- Composable signal processing
- Better alignment with CIM axioms

---

### 4. Pure Projection Adapters

**Current Issue:**
```rust
// ❌ Mutable state
impl ProjectionAdapter for NetBoxProjectionAdapter {
    async fn project(&mut self, event: Self::Event) -> Result<(), Self::Error> {
        self.device_cache.insert(event.device_id, device);  // Mutation
    }
}
```

**Desired Pattern:**
```rust
// ✅ Pure function
pub struct ProjectionState {
    device_cache: HashMap<Uuid, NetBoxDevice>,
    sequence: u64,
}

impl ProjectionAdapter for NetBoxProjectionAdapter {
    type State = ProjectionState;

    async fn project(
        &self,
        state: Self::State,
        event: Self::Event,
    ) -> Result<(Self::State, Vec<SideEffect>), Self::Error> {
        let new_cache = state.device_cache.clone();
        new_cache.insert(event.device_id, device);

        let effects = vec![
            SideEffect::HttpRequest(create_device_request),
        ];

        Ok((
            ProjectionState {
                device_cache: new_cache,
                sequence: state.sequence + 1,
            },
            effects,
        ))
    }
}
```

**Files to Modify:**
- `src/projection.rs` - Update trait definition
- `src/adapters/netbox.rs` - Refactor to pure functions
- `src/adapters/neo4j.rs` - Refactor to pure functions

**Estimated Effort**: 2-3 days

**Benefits**:
- Easier testing (pure functions)
- Better reasoning about state
- Parallel projection possible
- No hidden side effects

---

### 5. Enhanced Value Objects

**Current Issue:**
```rust
// ❌ Primitive obsession
pub struct ComputeResource {
    pub manufacturer: Option<String>,  // Unvalidated
    pub model: Option<String>,         // Unvalidated
    pub serial_number: Option<String>, // Unvalidated
}
```

**Desired Pattern:**
```rust
// ✅ Rich value objects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Manufacturer(String);

impl Manufacturer {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let name = name.into();
        if name.is_empty() || name.len() > 100 {
            return Err(DomainError::InvalidManufacturer);
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Model(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SerialNumber(String);

impl SerialNumber {
    pub fn new(serial: impl Into<String>) -> Result<Self, DomainError> {
        let serial = serial.into();
        // Validate format
        if !Self::is_valid(&serial) {
            return Err(DomainError::InvalidSerialNumber);
        }
        Ok(Self(serial))
    }

    fn is_valid(serial: &str) -> bool {
        // Alphanumeric, 6-50 chars
        serial.len() >= 6
            && serial.len() <= 50
            && serial.chars().all(|c| c.is_alphanumeric() || c == '-')
    }
}

// Typed metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetadataKey(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataValue(String);

pub type Metadata = HashMap<MetadataKey, MetadataValue>;
```

**Files to Create:**
- `src/domain/value_objects/manufacturer.rs`
- `src/domain/value_objects/model.rs`
- `src/domain/value_objects/serial_number.rs`
- `src/domain/value_objects/metadata.rs`

**Estimated Effort**: 2 days

**Benefits**:
- Stronger invariants
- Better validation
- Type-driven documentation
- Harder to misuse

---

## Priority 3: Nice to Have (Address in v0.3.x)

### 6. Circuit Breakers and Retry Logic

**Implementation:**
```rust
use tokio_retry::{strategy::ExponentialBackoff, Retry};

pub struct ResilientProjector<P: ProjectionAdapter> {
    adapter: P,
    circuit_breaker: CircuitBreaker,
    retry_strategy: ExponentialBackoff,
}

impl<P: ProjectionAdapter> ResilientProjector<P> {
    pub async fn project_with_retry(
        &mut self,
        event: P::Event,
    ) -> Result<(), ProjectionError> {
        if self.circuit_breaker.is_open() {
            return Err(ProjectionError::CircuitOpen);
        }

        let result = Retry::spawn(
            self.retry_strategy.clone(),
            || self.adapter.project(event.clone()),
        )
        .await;

        match result {
            Ok(_) => {
                self.circuit_breaker.record_success();
                Ok(())
            }
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e)
            }
        }
    }
}
```

**Files to Create:**
- `src/projection/resilience.rs`
- `src/projection/circuit_breaker.rs`
- `src/projection/retry_strategy.rs`

**Estimated Effort**: 2-3 days

---

### 7. Dimension Semantics Documentation

**Create type-level dimension constants:**
```rust
/// Conceptual space dimensions for infrastructure resources
pub mod dimensions {
    use std::marker::PhantomData;

    /// Dimension index type (compile-time safety)
    pub struct DimensionIndex<const N: usize>;

    /// Scale dimension (0.0 = tiny, 1.0 = massive)
    pub type Scale = DimensionIndex<0>;

    /// Complexity dimension (0.0 = simple, 1.0 = complex)
    pub type Complexity = DimensionIndex<1>;

    /// Reliability dimension (0.0 = unreliable, 1.0 = highly reliable)
    pub type Reliability = DimensionIndex<2>;

    /// Performance dimension (0.0 = slow, 1.0 = fast)
    pub type Performance = DimensionIndex<3>;

    /// Cost efficiency dimension (0.0 = expensive, 1.0 = efficient)
    pub type CostEfficiency = DimensionIndex<4>;

    /// N-dimensional position with typed dimensions
    pub struct Position<const N: usize>([f64; N]);

    impl Position<5> {
        pub fn scale(&self) -> f64 {
            self.0[0]
        }

        pub fn complexity(&self) -> f64 {
            self.0[1]
        }

        pub fn reliability(&self) -> f64 {
            self.0[2]
        }

        pub fn performance(&self) -> f64 {
            self.0[3]
        }

        pub fn cost_efficiency(&self) -> f64 {
            self.0[4]
        }
    }
}
```

**Files to Create:**
- `src/conceptual_space/dimensions.rs`
- `src/conceptual_space/position.rs`
- `docs/CONCEPTUAL_SPACE.md`

**Estimated Effort**: 1-2 days

---

### 8. Machine Learning Integration

**Concept:**
- Use ML to learn optimal dimension positions from real infrastructure data
- Train models to predict resource behavior from conceptual position
- Use geometric reasoning for anomaly detection

**Files to Create:**
- `src/ml/dimension_learner.rs`
- `src/ml/position_predictor.rs`
- `examples/ml_dimension_calibration.rs`

**Estimated Effort**: 1-2 weeks (research + implementation)

---

## Implementation Timeline

### Week 1-2: Priority 1 (Critical)
- [ ] Day 1-3: Implement event-sourced aggregates
- [ ] Day 4-5: Add event versioning infrastructure
- [ ] Day 6-7: Write migration tests and documentation

### Week 3-4: Priority 2 (Important)
- [ ] Day 1-4: FRP signal abstractions
- [ ] Day 5-7: Pure projection adapters
- [ ] Day 8-9: Enhanced value objects
- [ ] Day 10: Integration testing

### Week 5-6: Priority 3 (Nice to Have)
- [ ] Day 1-3: Circuit breakers and retry logic
- [ ] Day 4-5: Dimension semantics documentation
- [ ] Day 6-10: ML integration (research + POC)

---

## Success Criteria

### v0.2.0 Release Checklist

**Must Have:**
- [ ] All aggregates use pure event sourcing
- [ ] Event versioning infrastructure in place
- [ ] All tests passing with new patterns
- [ ] Documentation updated
- [ ] Migration guide for v0.1.0 → v0.2.0

**Should Have:**
- [ ] FRP signal abstractions
- [ ] Pure projection adapters
- [ ] Enhanced value objects
- [ ] Circuit breakers implemented

**Nice to Have:**
- [ ] Dimension semantics documentation
- [ ] ML integration POC
- [ ] Performance benchmarks

---

## Testing Strategy

### Unit Tests
- Event application logic
- Value object validation
- Signal combinators
- Projection state transitions

### Integration Tests
- End-to-end event sourcing
- JetStream persistence
- Projection adapter execution
- Event versioning migrations

### Property-Based Tests
- Event application properties (commutativity, associativity)
- Signal combinator laws
- Projection idempotency

---

## Documentation Updates

### New Documentation Required

1. **ARCHITECTURE.md** - System architecture with decision records
2. **EVENT_SOURCING.md** - Event sourcing patterns and examples
3. **FRP_GUIDE.md** - FRP signal usage guide
4. **PROJECTION_GUIDE.md** - Writing projection adapters
5. **MIGRATION_GUIDE.md** - v0.1.0 → v0.2.0 migration

### Updated Documentation

1. **README.md** - Update with new patterns
2. **CONTRIBUTING.md** - Add event sourcing guidelines
3. **TESTING.md** - Add property-based testing guide

---

## Conclusion

This roadmap provides a clear path from v0.1.0 (92/100) to v0.2.0 (target: 98/100). The improvements focus on:

1. **Correctness**: Pure event sourcing, immutability
2. **Maintainability**: Versioning, pure functions
3. **Composability**: FRP abstractions, signal operators
4. **Reliability**: Circuit breakers, retry logic
5. **Intelligence**: ML integration, semantic reasoning

By following this roadmap, cim-infrastructure will become a reference implementation of CIM architecture principles.

---

**Next Steps**:
1. Review and approve roadmap
2. Create GitHub issues for each Priority 1 item
3. Begin implementation of event-sourced aggregates
4. Schedule weekly progress reviews

**Estimated Total Effort**: 4-6 weeks for v0.2.0
