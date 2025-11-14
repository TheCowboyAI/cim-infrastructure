# CIM Infrastructure

Infrastructure management modules for the CIM (Composable Information Machine) architecture.

## Overview

This workspace contains infrastructure-related modules for CIM. Infrastructure modules provide event-sourced domain models and tools for managing compute resources, networks, policies, and configurations.

## Workspace Structure

```
cim-infrastructure/
├── Cargo.toml                      # Workspace configuration
├── flake.nix                       # Nix development environment
├── README.md                       # This file
├── cim-domain-infrastructure/      # Infrastructure domain module
│   ├── src/
│   │   ├── aggregate.rs           # Event-sourced aggregate root
│   │   ├── commands.rs            # Command types
│   │   ├── events.rs              # Domain events
│   │   ├── value_objects.rs       # Value objects
│   │   └── lib.rs                 # Public API
│   ├── Cargo.toml
│   ├── README.md
│   └── EXTRACTION.md
└── cim-infrastructure-nats/        # NATS JetStream integration
    ├── src/
    │   ├── event_store.rs         # JetStream event persistence
    │   ├── publisher.rs           # Event publishing
    │   ├── subscriber.rs          # Event subscription
    │   ├── projections.rs         # Read model projections
    │   ├── subjects.rs            # Subject hierarchy
    │   └── lib.rs                 # Public API
    ├── tests/
    │   └── integration_test.rs    # Real NATS integration tests
    ├── Cargo.toml
    └── README.md
```

## Modules

### cim-domain-infrastructure

**Status**: ✅ Production Ready

Event-sourced infrastructure domain providing:
- **Compute Resources**: Physical servers, VMs, containers
- **Network Topology**: Networks, interfaces, connections
- **Software Management**: Artifacts, configurations, dependencies
- **Graph Integration**: cim-graph via Kan extension for visualization
- **Policy Integration**: Optional cim-domain-policy integration (enable with `--features policy`)

**Version**: 0.1.0

**Dependencies**:
- Core: serde, uuid, chrono, thiserror
- CIM: cim-domain, cim-graph
- Optional: cim-domain-policy (with "policy" feature)

See [cim-domain-infrastructure/README.md](./cim-domain-infrastructure/README.md) for details.

### cim-infrastructure-nats

**Status**: ✅ Production Ready

NATS JetStream integration for infrastructure events:
- **Event Store**: JetStream-backed event persistence
- **Publisher**: Type-safe event publishing to NATS subjects
- **Subscriber**: Event subscription with async handlers
- **Projections**: Read model projections from event streams
- **Subject Hierarchy**: `infrastructure.{aggregate}.{operation}`

**Version**: 0.1.0

**Dependencies**: async-nats, tokio, cim-domain-infrastructure

**NATS Server Required**: Tests require NATS server with JetStream enabled

See [cim-infrastructure-nats/README.md](./cim-infrastructure-nats/README.md) for details.

### cim-infrastructure-neo4j

**Status**: 🚧 In Development

Neo4j graph database projection for infrastructure visualization:
- **Graph Model**: Nodes (ComputeResource, Network, Interface, Software, Policy)
- **Relationships**: HAS_INTERFACE, CONNECTED_TO, ROUTES_TO, RUNS, ENFORCES, APPLIES
- **Event Subscriber**: Real-time projection from NATS events
- **Specialized Queries**: Routing paths, network topology, policy analysis
- **Functor Pattern**: Structure-preserving mapping F: Infrastructure → Neo4jGraph

**Version**: 0.1.0

**Dependencies**: neo4rs, async-nats, tokio, cim-domain-infrastructure, cim-infrastructure-nats

**Prerequisites**: Requires Neo4j database instance

See [cim-infrastructure-neo4j/README.md](./cim-infrastructure-neo4j/README.md) for details.

## Usage

### As a Workspace Member

When developing infrastructure modules:

```bash
cd cim-infrastructure
cargo build          # Build all modules
cargo test           # Test all modules
cargo check --workspace
```

### As a Dependency

Other CIM modules can depend on infrastructure modules:

```toml
# In your Cargo.toml
[dependencies]
cim-domain-infrastructure = { path = "../cim-infrastructure/cim-domain-infrastructure" }
```

Or when published to crates.io:

```toml
[dependencies]
cim-domain-infrastructure = "0.1"
```

## Current Modules

| Module | Version | Status | Description |
|--------|---------|--------|-------------|
| cim-domain-infrastructure | 0.1.0 | ✅ Ready | Event-sourced infrastructure domain |
| cim-infrastructure-nats | 0.1.0 | ✅ Ready | NATS JetStream integration |
| cim-infrastructure-neo4j | 0.1.0 | 🚧 Dev | Neo4j graph database projection |

## Saga Orchestration

Complex workflows are handled using `cim-domain::saga` infrastructure - no separate module needed!

See the [network_provisioning_saga.rs](./cim-domain-infrastructure/examples/network_provisioning_saga.rs) example for:
- Saga as aggregate-of-aggregates pattern
- Mealy state machine workflow coordination
- Vector clock causal ordering
- Multi-step infrastructure provisioning

## Future Modules

Planned infrastructure modules:

- **cim-infrastructure-metrics** - Infrastructure monitoring and metrics
- **cim-infrastructure-projections** - Specialized read model projections

## Integration with Other CIM Modules

Infrastructure integrates with existing CIM modules through composition:

- **cim-graph** ✅ - Graph visualization via Kan extension (INTEGRATED)
- **cim-domain-policy** ✅ - Policy enforcement and evaluation (INTEGRATED, optional feature)
- **cim-domain-git** - Git repository infrastructure tracking
- **cim-domain-nix** - Nix configuration and infrastructure as code

### Using Optional Features

```bash
# Build with policy integration
cargo build --features policy

# Run tests with policy integration
cargo test --features policy

# Build all features
cargo build --all-features
```

## Architecture Principles

All infrastructure modules follow these principles:

1. **Event Sourcing**: All state changes as immutable events
2. **Domain-Driven Design**: Clear bounded contexts and aggregates
3. **Framework Independence**: No coupling to specific frameworks
4. **Type Safety**: Strongly typed value objects with validation
5. **Testability**: Comprehensive unit and integration tests

## Development

### Prerequisites

- Rust 1.70+
- Cargo workspace support

### Building

```bash
# Build all modules
cargo build --workspace

# Build specific module
cargo build -p cim-domain-infrastructure

# Build for release
cargo build --workspace --release
```

### Testing

```bash
# Test all modules
cargo test --workspace

# Test specific module
cargo test -p cim-domain-infrastructure

# Test with coverage
cargo tarpaulin --workspace
```

### Adding a New Module

1. Create module directory:
   ```bash
   cargo new --lib cim-infrastructure-newmodule
   ```

2. Add to workspace in `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       "cim-domain-infrastructure",
       "cim-infrastructure-newmodule",  # Add here
   ]
   ```

3. Use workspace dependencies:
   ```toml
   # In new module's Cargo.toml
   [dependencies]
   serde = { workspace = true }
   uuid = { workspace = true }
   ```

## Used By

Infrastructure modules are used by:

- **cim-domain-nix** - Nix configuration management
- **cim-network** - Network topology tools
- **cim-domain-policy** - Policy management
- **cim-leaf-*** - Leaf node implementations

## Contributing

Contributions welcome! When adding infrastructure modules:

1. Follow DDD and event sourcing principles
2. Maintain minimal dependencies
3. Provide comprehensive tests
4. Document the public API
5. Include usage examples

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Copyright

Copyright 2025 Cowboy AI, LLC. All rights reserved.
