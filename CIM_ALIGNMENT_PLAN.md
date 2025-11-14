# CIM Infrastructure Alignment Plan

**Date**: 2025-11-13
**Status**: 📋 PLANNING

## Overview

This document outlines the improvements needed to align `cim-infrastructure` with current CIM methodologies and best practices.

## Current State Assessment

### ✅ What We Have

1. **Event Sourcing** - Complete implementation
   - Event-sourced aggregates
   - Immutable events
   - Command/Event separation (CQRS)

2. **UUID v7** - Correctly implemented
   - `Uuid::now_v7()` used for time-ordered IDs
   - No v4 or v5 usage

3. **Message Identity** - Complete
   - Correlation ID tracking
   - Causation ID for event chains
   - Proper factory methods

4. **DDD Patterns** - Implemented
   - Aggregates (InfrastructureAggregate)
   - Commands (with specs)
   - Events (immutable)
   - Value Objects (validated)

5. **Clean Architecture** - Good
   - Domain independence
   - Minimal dependencies
   - Framework-agnostic

### ❌ What's Missing

## 1. NATS Integration

**Priority**: 🔴 HIGH

**Current**: No NATS integration in infrastructure domain
**Needed**: Full NATS JetStream event store

### Implementation Plan

Create `cim-infrastructure-nats` module:

```
cim-infrastructure/
└── cim-infrastructure-nats/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── event_store.rs       # JetStream event persistence
    │   ├── publisher.rs          # Event publishing
    │   ├── subscriber.rs         # Event subscription
    │   ├── subjects.rs           # Subject hierarchy
    │   └── projections.rs        # Read model projections
    └── tests/
        └── integration_tests.rs  # Real NATS tests
```

**NATS Subject Pattern**:
```
infrastructure.{aggregate}.{operation}

Examples:
- infrastructure.compute.registered
- infrastructure.network.defined
- infrastructure.connection.established
- infrastructure.software.configured
- infrastructure.policy.set
```

**Requirements**:
- JetStream stream for event persistence
- KV store for read models
- Proper ack handling
- Replay capability
- Stream consumer groups

## 2. Nix Integration

**Priority**: 🔴 HIGH

**Current**: No flake.nix
**Needed**: Full Nix development environment

### Implementation Plan

**File**: `cim-infrastructure/flake.nix`

```nix
{
  description = "CIM Infrastructure - Event-sourced infrastructure management";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "cim-domain-infrastructure";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-nextest
            nats-server
          ];

          shellHook = ''
            echo "CIM Infrastructure Development Shell"
          '';
        };
      });
}
```

## 3. Graph Representations & IPLD

**Priority**: ✅ USE EXISTING MODULE

**Current**: Custom graph implementation (to be replaced)
**Needed**: Integration with `cim-graph`

### Implementation Plan

**DO NOT implement graph functionality in cim-infrastructure!**

Instead, integrate with existing `cim-graph` module:

```toml
# cim-infrastructure/Cargo.toml
[workspace.dependencies]
cim-graph = { path = "../cim-graph" }
```

```rust
// Use cim-graph for all graph operations
use cim_graph::{Graph, Node, Edge, MermaidRenderer};

// Implement conversion from Infrastructure to cim-graph
impl From<&InfrastructureAggregate> for Graph {
    fn from(infra: &InfrastructureAggregate) -> Self {
        // Convert infrastructure to cim-graph format
    }
}

// Use cim-graph's Mermaid renderer
let graph = Graph::from(&infrastructure);
let mermaid = graph.render_mermaid();
```

This follows CIM best practices:
- **COMPOSE** existing modules
- **DON'T** re-invent functionality
- **INTEGRATE** through standard interfaces

## 4. Progress Documentation

**Priority**: 🟢 LOW

**Current**: No PROGRESS_LOG.md
**Needed**: Continuous progress tracking

### Implementation Plan

**File**: `cim-infrastructure/PROGRESS_LOG.md`

Template:
```markdown
# CIM Infrastructure Progress Log

## [Date] - Session Title

### Completed
- Item 1
- Item 2

### In Progress
- Item 3

### Next Steps
- Item 4

### Learnings
- Best practice 1
- Best practice 2
```

## 5. Testing Infrastructure

**Priority**: 🔴 HIGH

**Current**: Basic unit tests
**Needed**: Integration tests with real NATS

### Implementation Plan

```bash
cim-infrastructure/
├── tests/
│   ├── integration_test.rs      # Real NATS integration
│   ├── nats_fixtures.rs         # NATS test harness
│   └── workflow_tests.rs        # End-to-end workflows
```

**Example Integration Test**:
```rust
#[tokio::test]
async fn test_infrastructure_event_workflow() {
    // Start NATS server
    let nats = start_nats_server().await;

    // Create aggregate
    let mut infra = InfrastructureAggregate::new(InfrastructureId::new());

    // Publish events to NATS
    let publisher = NatsPublisher::new(&nats).await;

    // Execute command
    let spec = ComputeResourceSpec { /* ... */ };
    let events = infra.handle_register_compute_resource(spec, &identity)?;

    // Publish events
    for event in events {
        publisher.publish_event(&event).await?;
    }

    // Verify in NATS
    let stored = nats.get_last_event("infrastructure.compute.registered").await?;
    assert!(stored.is_some());
}
```

## 6. Saga Implementation

**Priority**: 🟡 MEDIUM

**Current**: No saga support
**Needed**: Saga orchestration for complex workflows

### Implementation Plan

**File**: `cim-infrastructure/cim-infrastructure-sagas/`

```rust
// Example: Network Provisioning Saga
pub struct NetworkProvisioningSaga {
    state: SagaState,
    infrastructure_id: InfrastructureId,
}

impl Saga for NetworkProvisioningSaga {
    fn execute(&mut self, event: &InfrastructureEvent) -> Result<Vec<Command>>;
    fn compensate(&mut self, event: &InfrastructureEvent) -> Result<Vec<Command>>;
}

// States as Markov chain
pub enum SagaState {
    Initial,
    NetworkDefined,
    ResourcesRegistered,
    InterfacesConfigured,
    ConnectionsEstablished,
    Complete,
    Failed(String),
}
```

## 7. CI/CD Pipeline

**Priority**: 🟡 MEDIUM

**Current**: No CI/CD
**Needed**: GitHub Actions workflow

### Implementation Plan

**File**: `.github/workflows/ci.yml`

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: nixbuild/nix-quick-install-action@v25
      - run: nix develop --command cargo build --workspace
      - run: nix develop --command cargo test --workspace
      - run: nix develop --command cargo clippy -- -D warnings

  nats-integration:
    runs-on: ubuntu-latest
    services:
      nats:
        image: nats:latest
        ports:
          - 4222:4222
    steps:
      - uses: actions/checkout@v3
      - run: cargo test --test integration_test
```

## 8. Documentation Improvements

**Priority**: 🟢 LOW

**Current**: Basic README
**Needed**: Comprehensive documentation

### Implementation Plan

```
cim-infrastructure/
├── docs/
│   ├── architecture/
│   │   ├── domain-model.md       # Complete domain model
│   │   ├── event-sourcing.md     # Event sourcing patterns
│   │   └── nats-integration.md   # NATS patterns
│   ├── guides/
│   │   ├── getting-started.md    # Quick start guide
│   │   ├── testing.md            # Testing guide
│   │   └── deployment.md         # Deployment guide
│   └── examples/
│       ├── basic-usage.md        # Basic examples
│       └── advanced-workflows.md # Complex workflows
```

## 9. Examples

**Priority**: 🟡 MEDIUM

**Current**: No examples
**Needed**: Comprehensive usage examples

### Implementation Plan

```
cim-infrastructure/
└── examples/
    ├── basic_infrastructure.rs   # Simple resource registration
    ├── network_topology.rs       # Network setup
    ├── nats_integration.rs       # NATS event publishing
    ├── event_replay.rs           # Event sourcing replay
    └── saga_orchestration.rs     # Complex workflow
```

## 10. Policy Framework

**Priority**: ✅ USE EXISTING MODULE

**Current**: Basic policy events in infrastructure domain
**Needed**: Integration with `cim-domain-policy`

### Implementation Plan

**DO NOT implement policy engine in cim-infrastructure!**

Instead, integrate with existing `cim-domain-policy` module:

```toml
# cim-infrastructure/Cargo.toml
[workspace.dependencies]
cim-domain-policy = { path = "../cim-domain-policy" }
```

The infrastructure domain emits PolicyApplied events, but policy **enforcement and evaluation** should be handled by `cim-domain-policy`.

This follows CIM best practices:
- **ASSEMBLE** existing modules
- **DON'T** rebuild functionality
- **INTEGRATE** through events and commands

## Implementation Priority

### Phase 1: Core Infrastructure (Week 1)
1. ✅ Create workspace structure
2. ✅ Extract domain module
3. 🔴 Add NATS integration
4. 🔴 Add flake.nix
5. 🔴 Add integration tests

### Phase 2: Observability (Week 2)
6. 🟡 Add graph representations
7. 🟡 Add Mermaid diagrams
8. 🟡 Add progress logging
9. 🟡 Add CI/CD

### Phase 3: Advanced Features (Week 3)
10. 🟡 Add saga support
11. 🟡 Add policy engine
12. 🟡 Add examples
13. 🟢 Add comprehensive docs

## Success Criteria

### Must Have
- ✅ Event sourcing working
- ✅ UUID v7 everywhere
- ✅ Message identity with correlation/causation
- ✅ NATS JetStream integration
- ✅ Integration tests with real NATS
- ✅ Nix development environment

### Should Have
- ✅ Graph/IPLD representation
- ✅ Mermaid visualization
- 🟡 Saga orchestration
- 🟡 CI/CD pipeline

### Nice to Have
- ✅ Comprehensive examples (topology_visualization.rs)
- 🟢 Advanced documentation
- ✅ Policy integration (use cim-domain-policy, not rebuild)

## Next Actions

1. **Create cim-infrastructure-nats module**
   ```bash
   cd /git/thecowboyai/cim-infrastructure
   cargo new --lib cim-infrastructure-nats
   ```

2. **Add flake.nix**
   ```bash
   # Create flake.nix at workspace root
   ```

3. **Set up integration tests**
   ```bash
   mkdir tests
   # Create integration test files
   ```

4. **Update workspace Cargo.toml**
   ```toml
   [workspace]
   members = [
       "cim-domain-infrastructure",
       "cim-infrastructure-nats",  # Add new module
   ]
   ```

## References

- CIM Best Practices: `/git/thecowboyai/CLAUDE.md`
- NATS Patterns: `/git/thecowboyai/nats-infrastructure/`
- Domain Patterns: `/git/thecowboyai/cim-start/`
- Event Sourcing: `cim-domain-infrastructure/EXTRACTION.md`

---

**Status**: Ready for implementation
**Owner**: Infrastructure Team
**Timeline**: 3 weeks for full alignment
