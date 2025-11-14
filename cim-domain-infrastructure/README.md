# CIM Domain Infrastructure

Event-sourced infrastructure domain for the CIM (Composable Information Machine) architecture.

## Overview

`cim-domain-infrastructure` provides a complete Domain-Driven Design (DDD) implementation for managing infrastructure using event sourcing. It models compute resources, network topology, software configurations, and policies as first-class domain concepts.

## Features

- **Event-Sourced**: All state changes represented as immutable events
- **Type-Safe**: Strongly typed value objects with validation
- **Domain-Driven**: Clean separation of concerns using DDD patterns
- **Framework-Independent**: No coupling to specific frameworks or storage
- **Production-Ready**: Comprehensive error handling and validation

## Architecture

The domain follows these principles:

1. **Event Sourcing**: All state changes are represented as immutable events
2. **CQRS**: Commands modify state, events represent what happened
3. **Aggregate Root**: `InfrastructureAggregate` maintains consistency
4. **Value Objects**: Immutable, validated data types
5. **Domain Independence**: No knowledge of external formats (Nix, YAML, etc.)

## Core Concepts

### Compute Resources
- Physical servers
- Virtual machines
- Containers
- Network devices

### Network Topology
- Networks (LAN, WAN, VLAN)
- Network interfaces (Ethernet, WiFi, Loopback)
- Physical connections
- IPv4/IPv6 addressing with CIDR

### Software Management
- Software artifacts
- Configurations
- Dependencies
- Versioning

### Policy Rules
- Security policies
- Access control
- Compliance rules
- Performance policies

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cim-domain-infrastructure = "0.1"
```

### Basic Example

```rust
use cim_domain_infrastructure::*;

// Create infrastructure aggregate
let infrastructure_id = InfrastructureId::new();
let mut infrastructure = InfrastructureAggregate::new(infrastructure_id);
let identity = MessageIdentity::new_root();

// Register a compute resource
let spec = ComputeResourceSpec {
    id: ResourceId::new("web-server-01")?,
    resource_type: ComputeType::Physical,
    hostname: Hostname::new("web-server-01.example.com")?,
    system: SystemArchitecture::x86_64_linux(),
    capabilities: ResourceCapabilities::new(),
};

infrastructure.handle_register_compute_resource(spec, &identity)?;

// Define a network
let network_spec = NetworkSpec {
    id: NetworkId::new("production-lan")?,
    name: "Production LAN".to_string(),
    cidr_v4: Some(Ipv4Network::new(
        std::net::Ipv4Addr::new(10, 0, 1, 0),
        24,
    )?),
    cidr_v6: None,
};

infrastructure.handle_define_network(network_spec, &identity)?;

// Get uncommitted events
let events = infrastructure.take_uncommitted_events();
for event in events {
    println!("Event: {:?}", event);
    // Persist to event store (NATS JetStream, etc.)
}
```

### Working with Value Objects

```rust
use cim_domain_infrastructure::*;

// Create validated hostname
let hostname = Hostname::new("server.example.com")?;

// Create IPv4 network with CIDR
let network = Ipv4Network::new(
    std::net::Ipv4Addr::new(192, 168, 1, 0),
    24,
)?;

// Create system architecture
let system = SystemArchitecture::new(
    "x86_64".to_string(),
    "linux".to_string(),
);
```

## Domain Model

### Aggregate Root

**`InfrastructureAggregate`** - Maintains consistency boundaries

Handles commands:
- `RegisterComputeResource`
- `AddInterface`
- `DefineNetwork`
- `EstablishConnection`
- `ConfigureSoftware`
- `SetPolicyRule`

Emits events:
- `ComputeResourceRegistered`
- `InterfaceAdded`
- `NetworkDefined`
- `ConnectionEstablished`
- `SoftwareConfigured`
- `PolicyRuleSet`

### Value Objects

All value objects are immutable and validated:

- `InfrastructureId` - UUID v7 aggregate identifier
- `ResourceId` - Compute resource identifier
- `NetworkId` - Network identifier
- `Hostname` - Validated DNS hostname
- `Ipv4Network` / `Ipv6Network` - CIDR notation networks
- `SystemArchitecture` - Platform identifier (x86_64-linux, etc.)
- `ResourceCapabilities` - Compute resource metadata

### Commands

Commands are requests to modify state:

```rust
pub struct ComputeResourceSpec {
    pub id: ResourceId,
    pub resource_type: ComputeType,
    pub hostname: Hostname,
    pub system: SystemArchitecture,
    pub capabilities: ResourceCapabilities,
}
```

### Events

Events are immutable facts that have occurred:

```rust
pub struct ComputeResource {
    pub resource_id: ResourceId,
    pub resource_type: ComputeType,
    pub hostname: Hostname,
    pub system: SystemArchitecture,
    pub capabilities: ResourceCapabilities,
}
```

## Integration

This domain is designed to be used by:

- **cim-domain-nix** - Nix configuration management
- **cim-network** - Network topology visualization
- **cim-domain-policy** - Policy enforcement
- **cim-leaf-*** - Leaf node implementations

## Event Sourcing

All state changes generate events that can be:
- Persisted to NATS JetStream
- Replayed for state reconstruction
- Projected into read models
- Used for audit trails

## Error Handling

The domain uses a custom `Result` type:

```rust
pub type Result<T> = std::result::Result<T, InfrastructureError>;

pub enum InfrastructureError {
    InvalidHostname(String),
    InvalidNetworkId(String),
    ResourceNotFound(String),
    NetworkNotFound(String),
    // ... more variants
}
```

## Testing

```bash
cargo test
```

All domain logic is unit tested with comprehensive coverage.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Copyright

Copyright 2025 Cowboy AI, LLC. All rights reserved.
