# cim-infrastructure-nats

NATS JetStream integration for CIM Infrastructure domain events.

## Overview

This crate provides complete NATS JetStream integration for the `cim-domain-infrastructure` module, enabling event sourcing with persistent event stores, event publishing/subscription, and read model projections.

## Features

- **Event Store**: JetStream-backed persistent event storage with replay capabilities
- **Publisher**: Type-safe event publishing to NATS subjects
- **Subscriber**: Event subscription with async handlers
- **Projections**: Read model projections from event streams
- **Subject Hierarchy**: Type-safe NATS subject patterns

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                Infrastructure Domain                   │
│                                                        │
│  ┌─────────────┐         ┌──────────────────┐       │
│  │  Aggregate  │────────▶│   Domain Events   │       │
│  └─────────────┘         └──────────────────┘       │
└──────────────────────┬───────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│            cim-infrastructure-nats                     │
│                                                        │
│  ┌──────────────┐                                     │
│  │   Publisher  │────────┐                            │
│  └──────────────┘        │                            │
│                          ▼                             │
│              ┌─────────────────────┐                  │
│              │  JetStream Store    │                  │
│              │  (Event Persistence) │                  │
│              └─────────────────────┘                  │
│                          │                             │
│                          ▼                             │
│  ┌──────────────┐  ┌──────────────┐                  │
│  │  Subscriber  │  │  Projections │                  │
│  └──────────────┘  └──────────────┘                  │
└──────────────────────────────────────────────────────┘
```

## NATS Subject Hierarchy

All infrastructure events follow the pattern:

```
infrastructure.{aggregate}.{operation}
```

### Examples

- `infrastructure.compute.registered`
- `infrastructure.compute.decommissioned`
- `infrastructure.compute.updated`
- `infrastructure.network.defined`
- `infrastructure.network.removed`
- `infrastructure.connection.established`
- `infrastructure.connection.severed`
- `infrastructure.software.configured`
- `infrastructure.policy.set`

### Wildcard Subscriptions

- `infrastructure.compute.>` - All compute events
- `infrastructure.network.>` - All network events
- `infrastructure.>` - All infrastructure events

## Usage

### Basic Event Publishing

```rust
use async_nats;
use cim_domain_infrastructure::{
    InfrastructureAggregate, InfrastructureId, MessageIdentity,
    ComputeResourceSpec, ComputeType, Hostname, SystemArchitecture,
    ResourceCapabilities,
};
use cim_infrastructure_nats::{
    event_store::{JetStreamEventStore, EventStoreConfig},
    publisher::EventPublisher,
    EventStore,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Create event store
    let config = EventStoreConfig::default();
    let event_store = Arc::new(
        JetStreamEventStore::new(client.clone(), config).await?
    );

    // Create publisher
    let publisher = EventPublisher::new(
        client,
        Arc::clone(&event_store) as Arc<dyn EventStore>
    );

    // Create infrastructure aggregate
    let infra_id = InfrastructureId::new();
    let mut aggregate = InfrastructureAggregate::new(infra_id.clone());

    // Register a compute resource
    let identity = MessageIdentity::new_from_user("my-service");
    let spec = ComputeResourceSpec {
        id: cim_domain_infrastructure::ResourceId::new(),
        resource_type: ComputeType::Physical,
        hostname: Hostname::new("server01")?,
        system: SystemArchitecture::new("x86_64-linux")?,
        capabilities: ResourceCapabilities::default(),
    };

    let events = aggregate.handle_register_compute_resource(spec, &identity)?;

    // Publish events
    for (seq, event) in events.iter().enumerate() {
        publisher.publish(event, infra_id.into(), (seq + 1) as u64).await?;
    }

    Ok(())
}
```

### Event Subscription

```rust
use cim_infrastructure_nats::{
    subscriber::{EventSubscriberBuilder, FnEventHandler},
    subjects::subjects,
    StoredEvent,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Create event handler
    let handler = Arc::new(FnEventHandler::new(|event: StoredEvent| {
        println!("Received: {} at {}", event.event_type, event.timestamp);
        Ok(())
    }));

    // Subscribe to all compute events
    let subscriber = EventSubscriberBuilder::new()
        .client(client)
        .subject(subjects::all_compute_events())
        .handler(handler)
        .build()?;

    let handle = subscriber.subscribe().await?;

    // Wait for events...
    handle.await?;

    Ok(())
}
```

### Read Model Projections

```rust
use cim_infrastructure_nats::{
    event_store::{JetStreamEventStore, EventStoreConfig, EventStore},
    projections::ProjectionManager,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Create event store
    let config = EventStoreConfig::default();
    let event_store = Arc::new(
        JetStreamEventStore::new(client.clone(), config).await?
    );

    // Create projection manager
    let projection_manager = ProjectionManager::new();

    // Read all events and project them
    let events = event_store.read_all().await?;
    projection_manager.rebuild(events).await?;

    // Query the topology
    let topology = projection_manager.get_topology().await;

    println!("Active compute resources: {}",
        topology.active_compute_resources().len());
    println!("Active networks: {}",
        topology.active_networks().len());
    println!("Active connections: {}",
        topology.active_connections().len());

    Ok(())
}
```

## Configuration

### Event Store Configuration

```rust
use cim_infrastructure_nats::event_store::EventStoreConfig;
use std::time::Duration;

let config = EventStoreConfig {
    stream_name: "INFRASTRUCTURE_EVENTS".to_string(),
    max_age: Duration::from_secs(365 * 24 * 60 * 60), // 1 year retention
    max_messages: 1_000_000,                            // Max 1M messages
    replicas: 3,                                        // 3 replicas for HA
};
```

## Integration Tests

The crate includes comprehensive integration tests that require a running NATS server:

```bash
# Start NATS with JetStream
nats-server -js

# Run integration tests
cargo test --test integration_test -- --ignored
```

### Available Integration Tests

- `test_complete_event_workflow` - Full publish/store/retrieve cycle
- `test_event_projection` - Projection building from events
- `test_event_subscription` - Event subscription and handling
- `test_event_replay` - Event replay and state reconstruction

## Development

### Prerequisites

- Rust 1.70+
- NATS Server 2.10+ with JetStream enabled

### Building

```bash
cargo build --workspace
```

### Testing

```bash
# Unit tests
cargo test -p cim-infrastructure-nats

# Integration tests (requires NATS server)
cargo test -p cim-infrastructure-nats --test integration_test -- --ignored
```

## Module Organization

- **`event_store.rs`** - JetStream event persistence
- **`publisher.rs`** - Event publishing
- **`subscriber.rs`** - Event subscription and handlers
- **`projections.rs`** - Read model projections
- **`subjects.rs`** - NATS subject patterns

## Dependencies

- `async-nats` - NATS client library
- `tokio` - Async runtime
- `serde`/`serde_json` - Serialization
- `uuid` - Event IDs
- `chrono` - Timestamps
- `tracing` - Logging
- `cim-domain-infrastructure` - Infrastructure domain

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## Copyright

Copyright 2025 Cowboy AI, LLC. All rights reserved.
