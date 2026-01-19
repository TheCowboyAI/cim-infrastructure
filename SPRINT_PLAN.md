<!-- Copyright (c) 2025 - Cowboy AI, Inc. -->

# CIM Infrastructure: Event Sourcing Refactoring Sprint

**Sprint Start**: 2026-01-19
**Sprint Duration**: 4 weeks
**Objective**: Transform cim-infrastructure from OOP mutation to functional event-sourcing architecture

---

## Sprint Objective

Transform cim-infrastructure from mutable OOP to immutable event-sourced functional DDD, addressing critical FRP violations identified in comprehensive expert reviews.

## Context

Three expert agent reviews identified:
- **SDLC Expert**: A- (92/100) - Good foundations, needs refinement
- **FRP Expert**: F (20/100) - SEVERE violations, 10+ mutable methods, no signal types
- **DDD Expert**: C+ (75/100) - Good value objects, missing event sourcing

## Critical Violations to Fix

1. 10+ mutable `&mut self` methods (set_organization, add_policy, etc.)
2. No event sourcing - state changes via direct mutation
3. 11+ occurrences of `Utc::now()` side effects
4. Missing InfrastructureEvent enum
5. No state machine for resource lifecycle
6. Missing FRP signal types (EventKind, StepKind)

---

## Sprint Phases

### Phase 1: Event Sourcing Foundation (Week 1) 🔴 CRITICAL

**Objective**: Establish event-driven architecture with immutable events

**Tasks**:
1. Define InfrastructureEvent enum with all domain events
2. Implement NatsEventStore with JetStream backend
3. Add correlation_id/causation_id to all events
4. Create event serialization/deserialization tests
5. Implement event versioning infrastructure

**Success Criteria**:
- [ ] All state changes represented as events
- [ ] Events stored in NATS JetStream
- [ ] Event correlation chain working
- [ ] Tests prove event immutability

**Files to Create**:
- `src/events/mod.rs` - Event module root
- `src/events/infrastructure.rs` - InfrastructureEvent enum
- `src/events/compute_resource.rs` - ComputeResource events
- `src/events/versioning.rs` - Event versioning infrastructure
- `src/event_store/mod.rs` - Event store abstraction
- `src/event_store/nats.rs` - NATS JetStream implementation
- `tests/events/event_serialization.rs` - Serialization tests
- `tests/events/event_versioning.rs` - Versioning tests

---

### Phase 2: Pure Functional Aggregates (Week 1-2) 🔴 CRITICAL

**Objective**: Transform aggregate from OOP mutations to pure functions

**Tasks**:
1. Create ComputeResourceState (immutable)
2. Replace all `&mut self` with `self` → `Self` pure functions
3. Remove all `Utc::now()` calls, pass time explicitly
4. Implement pure event application (fold)
5. Add command types (AssignOrganizationCommand, etc.)
6. Create compute_resource_aggregate module (pure functions)

**Success Criteria**:
- [ ] Zero `&mut self` methods remain
- [ ] All functions are referentially transparent
- [ ] State reconstruction via event folding works
- [ ] No side effects in domain logic

**Files to Modify**:
- `src/domain/compute_resource.rs` - Refactor to pure functions
- `src/domain/commands.rs` - Create command types
- `src/domain/state.rs` - Immutable state types

**Files to Create**:
- `src/aggregate/mod.rs` - Aggregate abstraction
- `src/aggregate/compute_resource.rs` - Pure aggregate functions
- `tests/aggregate/event_application.rs` - Event folding tests

---

### Phase 3: State Machine & Invariants (Week 2) 🟡 HIGH

**Objective**: Explicit lifecycle modeling with business rule enforcement

**Tasks**:
1. Define ResourceStatus enum (Provisioning, Active, Maintenance, Decommissioned)
2. Implement state transition validation
3. Create invariants module with pure validation functions
4. Integrate invariants into command handlers
5. Add Mealy state machine implementation
6. Document all business rules

**Success Criteria**:
- [ ] Invalid transitions rejected at compile time where possible
- [ ] All business rules documented and tested
- [ ] State machine visualization in tests
- [ ] Invariants enforced before event emission

**Files to Create**:
- `src/domain/resource_status.rs` - Status enum with transitions
- `src/domain/invariants.rs` - Pure validation functions
- `src/state_machine/mod.rs` - State machine abstraction
- `src/state_machine/resource_lifecycle.rs` - Resource lifecycle FSM
- `tests/state_machine/transitions.rs` - Transition tests
- `docs/STATE_MACHINE.md` - State machine documentation

---

### Phase 4: Service Layer & Integration (Week 2-3) 🟡 HIGH

**Objective**: Clean architecture with command/query separation

**Tasks**:
1. Implement ComputeResourceService trait
2. Create EventSourcedComputeResourceService
3. Integrate with NatsEventStore
4. Add NATS event publishing
5. Separate ConceptProjector service
6. Implement LiftableDomain trait for graph composition

**Success Criteria**:
- [ ] Commands handled via service layer
- [ ] Events published to NATS
- [ ] Projections decoupled from aggregate
- [ ] Integration tests with JetStream passing

**Files to Create**:
- `src/service/mod.rs` - Service layer root
- `src/service/compute_resource.rs` - ComputeResourceService
- `src/service/event_sourced.rs` - Event-sourced implementation
- `tests/service/integration.rs` - Integration tests

---

### Phase 5: FRP Signal Types (Week 3) 🟡 HIGH

**Objective**: Introduce proper FRP abstractions

**Tasks**:
1. Define Signal<T> trait
2. Implement Behavior<T> (continuous-time)
3. Implement Event<T> (discrete-time)
4. Add signal combinators (map, filter, fold, switch)
5. Create ResourceBehavior from event stream
6. Document FRP patterns

**Success Criteria**:
- [ ] Signal types properly model time
- [ ] Behavior vs Event distinction clear
- [ ] Combinators follow FRP laws
- [ ] Infrastructure state as continuous behavior

**Files to Create**:
- `src/frp/mod.rs` - FRP module root
- `src/frp/signal.rs` - Signal trait
- `src/frp/behavior.rs` - Behavior type
- `src/frp/event.rs` - Event type (rename existing)
- `src/frp/combinators.rs` - Signal operators
- `examples/frp_infrastructure.rs` - Usage examples
- `docs/FRP_GUIDE.md` - FRP patterns guide

---

### Phase 6: Pure Projection Adapters (Week 3-4) 🟢 MEDIUM

**Objective**: Remove mutable state from projections

**Tasks**:
1. Refactor ProjectionAdapter to pure function
2. Add ProjectionState type parameter
3. Return side effects explicitly
4. Update Neo4j adapter
5. Update NetBox adapter
6. Add projection replay capability

**Success Criteria**:
- [ ] No mutable state in adapters
- [ ] Pure function signatures
- [ ] Side effects explicit
- [ ] Projection replay works

**Files to Modify**:
- `src/projection.rs` - Update trait to pure functions
- `src/adapters/neo4j.rs` - Refactor to pure
- `src/adapters/netbox.rs` - Refactor to pure

---

### Phase 7: Testing & Documentation (Week 4) 🟢 MEDIUM

**Objective**: Comprehensive test coverage and documentation

**Tasks**:
1. Property-based tests for event application
2. State machine property tests
3. Integration tests with real NATS
4. Create migration guide from v0.1.0
5. Update ARCHITECTURE.md
6. Create EVENT_SOURCING.md guide

**Success Criteria**:
- [ ] 90%+ test coverage
- [ ] All properties proven with proptest
- [ ] Integration tests passing
- [ ] Complete documentation

**Files to Create**:
- `tests/property/event_application.rs` - Property tests
- `tests/property/state_machine.rs` - FSM property tests
- `docs/MIGRATION_v0.1_to_v0.2.md` - Migration guide
- `docs/EVENT_SOURCING.md` - Event sourcing patterns
- `docs/TESTING.md` - Testing strategy

---

## Progress Tracking

### Daily Updates
- Update SPRINT_PROGRESS.md daily
- Track blockers and technical decisions
- Note any scope changes

### Weekly Checkpoints
- Review completed tasks
- Adjust priorities if needed
- Update timeline estimates

### Sprint Retrospective
- Document lessons learned
- Identify patterns for future sprints
- Create best practices

---

## Agent Coordination

### TDD Expert (@tdd-expert)
- Create comprehensive test suite for event sourcing
- Property-based tests for event application
- State machine transition tests
- Integration tests with NATS

### FRP Expert (@frp-expert)
- Guide implementation of signal types (Phase 5)
- Review pure function implementations
- Ensure referential transparency
- Validate compositional patterns

### DDD Expert (@ddd-expert)
- Review aggregate boundaries
- Validate command/event design
- Ensure invariants are properly enforced
- Guide bounded context integration

### Git Expert (@git-expert)
- Module-per-aggregate repository strategy
- Event versioning in git
- Migration branch management

---

## Deliverables

By end of sprint:
1. ✅ Event-sourced ComputeResource aggregate
2. ✅ NatsEventStore integration
3. ✅ State machine implementation
4. ✅ Zero mutable methods
5. ✅ FRP signal types
6. ✅ Pure projection adapters
7. ✅ Complete test coverage
8. ✅ Migration documentation

---

## Files to Track

### Created Files (New)
```
src/events/
  mod.rs
  infrastructure.rs
  compute_resource.rs
  versioning.rs

src/event_store/
  mod.rs
  nats.rs

src/aggregate/
  mod.rs
  compute_resource.rs

src/domain/
  commands.rs
  state.rs
  resource_status.rs
  invariants.rs

src/state_machine/
  mod.rs
  resource_lifecycle.rs

src/service/
  mod.rs
  compute_resource.rs
  event_sourced.rs

src/frp/
  mod.rs
  signal.rs
  behavior.rs
  event.rs
  combinators.rs

tests/events/
  event_serialization.rs
  event_versioning.rs

tests/aggregate/
  event_application.rs

tests/state_machine/
  transitions.rs

tests/service/
  integration.rs

tests/property/
  event_application.rs
  state_machine.rs

docs/
  STATE_MACHINE.md
  FRP_GUIDE.md
  MIGRATION_v0.1_to_v0.2.md
  EVENT_SOURCING.md
  TESTING.md

examples/
  frp_infrastructure.rs
```

### Modified Files (Refactored)
```
src/domain/compute_resource.rs  - Remove mutations, add pure functions
src/projection.rs               - Pure function trait
src/adapters/neo4j.rs          - Pure projection
src/adapters/netbox.rs         - Pure projection
src/lib.rs                     - Export new modules
Cargo.toml                     - Add dependencies (proptest, etc.)
```

---

## Success Metrics

### Code Quality
- Zero `&mut self` methods in aggregates
- Zero `Utc::now()` calls in domain logic
- 90%+ test coverage
- All clippy warnings resolved

### Functional Correctness
- All property-based tests pass
- Integration tests with real NATS pass
- State machine transitions provably correct
- Event replay produces identical state

### Documentation
- All public APIs documented
- Migration guide complete
- Architecture decisions recorded
- Examples working and tested

---

## Risk Management

### High Risk
- **Breaking changes**: Existing code depends on mutable API
  - **Mitigation**: Create v0.2 branch, maintain v0.1 compatibility layer
- **NATS JetStream complexity**: Event store implementation complex
  - **Mitigation**: Start with in-memory store, add NATS incrementally

### Medium Risk
- **Scope creep**: FRP implementation could expand significantly
  - **Mitigation**: Focus on minimal FRP abstractions first
- **Performance**: Pure functions may have performance implications
  - **Mitigation**: Profile and optimize after correctness proven

### Low Risk
- **Testing time**: Property-based tests may take time to develop
  - **Mitigation**: Start with simple properties, expand iteratively

---

## Sprint Commitment

This sprint transforms cim-infrastructure into a functional event-sourcing exemplar that:

1. **Honors Event Sourcing**: All state changes via immutable events
2. **Embraces Purity**: Zero side effects in domain logic
3. **Respects FRP**: Proper signal abstractions
4. **Enforces Invariants**: Type-safe state machines
5. **Enables Reasoning**: Pure functions, property tests
6. **Documents Decisions**: Clear architecture records

**Let's build something beautiful!** 🚀
