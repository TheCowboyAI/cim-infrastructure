# CIM Infrastructure

Infrastructure utilities library for the CIM (Composable Information Machine) architecture.

## Overview

This library provides core infrastructure abstractions for CIM systems, including NATS messaging client, error handling, and cross-cutting concerns. It is designed to be used as a dependency by other CIM modules that need to connect to external NATS and Neo4j services.

## Library Structure

```
cim-infrastructure/
├── Cargo.toml                      # Package configuration
├── flake.nix                       # Nix development environment
├── README.md                       # This file
├── src/
│   ├── lib.rs                     # Public API
│   ├── nats.rs                    # NATS client abstraction
│   ├── errors.rs                  # Error types
└── tests/                          # Integration tests
```

## Features

### NATS Client

A high-level NATS client wrapper providing:
- **Connection Management**: Configurable connection with timeouts
- **Publish-Subscribe**: Simple pub/sub patterns
- **Request-Reply**: Synchronous request-reply patterns
- **Message Handlers**: Trait-based message processing
- **Typed Messages**: Serde-based serialization/deserialization

### Error Handling

Comprehensive error types for:
- NATS connection and communication errors
- Serialization/deserialization errors
- Configuration errors
- Timeout errors

### Projection Adapters

Multiple projection adapters for different read models:

**Neo4j** (Graph Database):
- Feature: `--features neo4j`
- Purpose: Topology visualization and graph queries
- See: `src/adapters/neo4j.rs`

**NetBox** (DCIM):
- Feature: `--features netbox`
- Purpose: Data Center Infrastructure Management
- Location: `http://10.0.224.131`
- Documentation: `docs/NETBOX_INTEGRATION.md`
- See: `src/adapters/netbox.rs`

## Usage

### As a Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
cim-infrastructure = { path = "../cim-infrastructure" }
```

Or when published to crates.io:

```toml
[dependencies]
cim-infrastructure = "0.1"
```

### Example: NATS Client

```rust
use cim_infrastructure::{NatsClient, NatsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create NATS client configuration
    let config = NatsConfig {
        servers: vec!["nats://localhost:4222".to_string()],
        name: "my-service".to_string(),
        ..Default::default()
    };

    // Connect to NATS
    let client = NatsClient::new(config).await?;

    // Publish a message
    #[derive(serde::Serialize)]
    struct MyMessage {
        data: String,
    }

    client.publish("my.subject", &MyMessage {
        data: "Hello, NATS!".to_string(),
    }).await?;

    // Subscribe to messages
    let mut subscriber = client.subscribe("my.subject").await?;

    Ok(())
}
```

### Example: Message Handler

```rust
use cim_infrastructure::{MessageHandler, InfrastructureResult};
use async_trait::async_trait;
use serde_json::Value;

struct MyHandler;

#[async_trait]
impl MessageHandler for MyHandler {
    type Message = Value;

    async fn handle(&self, message: Self::Message) -> InfrastructureResult<()> {
        println!("Received: {:?}", message);
        Ok(())
    }

    fn subject(&self) -> &str {
        "my.subject"
    }
}
```

## Architecture Principles

This library follows these principles:

1. **NATS-First Architecture**: All messaging through NATS
2. **Type Safety**: Strongly typed messages with serde serialization
3. **Async by Default**: Built on tokio for async operations
4. **Framework Independence**: Minimal dependencies, composable design
5. **Testability**: Comprehensive error handling and testing support

## Development

### Prerequisites

- Rust 1.70+
- NATS server (for testing)
- Cargo

### Building

```bash
# Build the library
cargo build

# Build for release
cargo build --release

# Check without building
cargo check
```

### Testing

```bash
# Run tests (requires NATS server running)
cargo test

# Run with specific test
cargo test --test integration_test
```

### Running NATS Server for Testing

```bash
# Using Docker
docker run -p 4222:4222 nats:latest

# Or using nats-server directly
nats-server
```

## Integration with CIM

This library is used by other CIM modules for:

- **NATS Connectivity**: Connect to external NATS servers
- **Event Messaging**: Publish and subscribe to domain events
- **Infrastructure Utilities**: Common error handling and abstractions

## Contributing

Contributions welcome! When contributing:

1. Follow async-first patterns with tokio
2. Maintain minimal dependencies
3. Provide comprehensive tests
4. Document the public API with examples
5. Ensure error types are descriptive

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Copyright

Copyright 2025 Cowboy AI, LLC. All rights reserved.
