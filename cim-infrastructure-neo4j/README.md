# CIM Infrastructure Neo4j Projection

Neo4j graph database projection for CIM Infrastructure events. This module provides a functor from the Infrastructure domain category to the Neo4j graph category, enabling powerful relationship queries for networks, routing, and policies.

## Features

- **Event-Driven Projection**: Subscribes to infrastructure events via NATS and projects to Neo4j
- **Graph Visualization**: Represents infrastructure as a connected graph with nodes and relationships
- **Specialized Queries**: Pre-built queries for routing paths, network topology, and policy analysis
- **Functor Pattern**: Category theory-based projection preserving domain structure
- **Real-Time Updates**: Infrastructure changes immediately reflected in graph database

## Graph Model

### Node Types

- **ComputeResource**: Physical servers, VMs, containers
- **Network**: Network segments with CIDR ranges
- **Interface**: Network interfaces on compute resources
- **Software**: Software artifacts and configurations
- **Policy**: Security and compliance policies

### Relationships

- `(ComputeResource)-[:HAS_INTERFACE]->(Interface)`
- `(Interface)-[:CONNECTED_TO]->(Network)`
- `(Interface)-[:ROUTES_TO]->(Interface)` - for physical connections
- `(ComputeResource)-[:RUNS]->(Software)`
- `(ComputeResource)-[:ENFORCES]->(Policy)`
- `(Network)-[:APPLIES]->(Policy)`

## Usage

### Prerequisites

1. **Neo4j Database**: Running instance (bolt://localhost:7687)
2. **NATS Server**: With JetStream enabled for event streaming

```bash
# Start Neo4j with Docker
docker run -d \
  --name neo4j \
  -p 7474:7474 -p 7687:7687 \
  -e NEO4J_AUTH=neo4j/password \
  neo4j:latest

# Start NATS server
nats-server -js
```

### Basic Example

```rust
use cim_infrastructure_neo4j::{Neo4jProjection, Neo4jConfig, InfrastructureQueries};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure connection
    let config = Neo4jConfig {
        uri: "bolt://localhost:7687".to_string(),
        user: "neo4j".to_string(),
        password: "password".to_string(),
        database: None,
    };

    // Connect and initialize
    let mut projection = Neo4jProjection::connect(config).await?;
    projection.initialize_schema().await?;

    // Start event subscriber (projects infrastructure.> events)
    projection.start_event_subscriber("nats://localhost:4222").await?;

    // Query the graph
    let queries = InfrastructureQueries::new(projection.graph().clone());

    // Find routing path between resources
    let path = queries.find_routing_path("web-01", "db-01").await?;

    // Get topology summary
    let summary = queries.get_topology_summary().await?;
    println!("Total resources: {}", summary.total_resources);
    println!("Total networks: {}", summary.total_networks);

    Ok(())
}
```

### Running the Example

```bash
# Run the demo example
cargo run --example neo4j_projection_demo

# View in Neo4j Browser
# Open: http://localhost:7474
# Query: MATCH (n) RETURN n LIMIT 25
```

## Specialized Queries

### Routing Path Analysis

```rust
// Find shortest path between two resources
let path = queries.find_routing_path("server-a", "server-b").await?;
for node in path {
    println!("{} -> ", node.hostname.unwrap_or(node.id));
}
```

### Network Topology

```rust
// Find all resources in a network
let resources = queries.find_resources_in_network("frontend-net").await?;

// Find all networks connected to a resource
let networks = queries.find_resource_networks("web-server-01").await?;

// Find network segments
let segments = queries.find_network_segments().await?;
for segment in segments {
    println!("{}: {} resources", segment.network_name, segment.resource_count);
}
```

### Policy Analysis

```rust
// Find all policies affecting a resource (direct and network-based)
let policies = queries.find_resource_policies("web-server-01").await?;
for policy in policies {
    println!("Policy: {} (from: {})", policy.name, policy.source);
}
```

### Topology Summary

```rust
let summary = queries.get_topology_summary().await?;
println!("Resources: {}", summary.total_resources);
println!("Networks: {}", summary.total_networks);
println!("Interfaces: {}", summary.total_interfaces);
println!("Connections: {}", summary.total_connections);
```

## Category Theory Foundation

The projection is a functor `F: Infrastructure → Neo4jGraph`:

- **Objects**: Domain entities (ComputeResource, Network, etc.) map to graph nodes
- **Morphisms**: Relationships and events map to graph edges
- **Identity Preservation**: `F(id) = id` - empty infrastructure → empty graph
- **Composition Preservation**: `F(g ∘ f) = F(g) ∘ F(f)` - connected paths preserved

This ensures the graph structure faithfully represents the domain structure.

## Cypher Examples

Once infrastructure is projected, query directly in Neo4j Browser:

```cypher
// View all infrastructure
MATCH (n) RETURN n LIMIT 25;

// Find routing relationships
MATCH p=()-[r:ROUTES_TO]->() RETURN p;

// Find all resources in a specific network
MATCH (n:Network {name: "Frontend DMZ"})
      <-[:CONNECTED_TO]-(:Interface)
      <-[:HAS_INTERFACE]-(r:ComputeResource)
RETURN r;

// Find shortest path between two resources
MATCH path = shortestPath(
  (from:ComputeResource {id: "web-01"})
  -[:HAS_INTERFACE|ROUTES_TO|CONNECTED_TO*]->
  (to:ComputeResource {id: "db-01"})
)
RETURN path;

// Find isolated resources (not connected to any network)
MATCH (r:ComputeResource)
WHERE NOT (r)-[:HAS_INTERFACE]->(:Interface)-[:CONNECTED_TO]->(:Network)
RETURN r;
```

## Architecture

```
Infrastructure Events (NATS)
         ↓
   Event Subscriber
         ↓
   Projection Builder ← Functor F: Domain → Graph
         ↓
   Neo4j Database
         ↓
   Query Interface
         ↓
   Graph Visualizations
```

## Configuration

```rust
pub struct Neo4jConfig {
    /// Neo4j URI (e.g., "bolt://localhost:7687")
    pub uri: String,
    /// Username for authentication
    pub user: String,
    /// Password for authentication
    pub password: String,
    /// Optional database name (defaults to "neo4j")
    pub database: Option<String>,
}
```

## Dependencies

- **neo4rs**: Async Neo4j driver
- **async-nats**: NATS messaging
- **tokio**: Async runtime
- **cim-domain-infrastructure**: Domain events
- **cim-infrastructure-nats**: NATS integration

## Testing

```bash
# Run unit tests
cargo test -p cim-infrastructure-neo4j

# Run with Neo4j integration (requires running Neo4j)
docker run -d -p 7687:7687 -e NEO4J_AUTH=neo4j/password neo4j:latest
cargo test -p cim-infrastructure-neo4j -- --ignored
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## Copyright

Copyright 2025 Cowboy AI, LLC. All rights reserved.
