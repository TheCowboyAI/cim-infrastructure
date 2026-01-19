# CIM Infrastructure - CQRS Architecture with Conceptual Spaces

## Philosophy

**The system builds itself through events and understands itself through Conceptual Spaces.**

Infrastructure is hardware, networking, and operating programs in a CIM. We use every tool available to do introspection on ourselves - the CIM discovers its own state, models it semantically through Conceptual Spaces, and projects it into multiple read models through event sourcing.

## Architectural Stack

```text
┌─────────────────────────────────────────────────────────────────┐
│                    SEMANTIC LAYER                                │
│              cim-domain-spaces (v0.9.7)                         │
│                                                                  │
│  Conceptual Spaces: Geometric semantic representation           │
│  - Infrastructure concepts (Server, Network, Connection)        │
│  - Quality dimensions (Performance, Capacity, Reliability)      │
│  - Similarity metrics and prototype-based reasoning             │
└─────────────────────────────────────────────────────────────────┘
                             ↕
┌─────────────────────────────────────────────────────────────────┐
│                    DOMAIN LAYER                                  │
│                cim-domain (v0.8.1)                              │
│                                                                  │
│  DDD Patterns: Commands, Events, Aggregates, Entities           │
│  - MessageIdentity (correlation_id, causation_id)               │
│  - Command trait, Event trait                                   │
│  - CQRS infrastructure                                          │
└─────────────────────────────────────────────────────────────────┘
                             ↕
┌─────────────────────────────────────────────────────────────────┐
│                INFRASTRUCTURE LAYER                              │
│              cim-infrastructure (this repo)                      │
│                                                                  │
│  CQRS Implementation: Commands → Events → Projections           │
│  - Self-introspection and discovery                             │
│  - JetStream event sourcing                                     │
│  - Multiple projection adapters                                 │
└─────────────────────────────────────────────────────────────────┘
```

## CQRS Pattern

We follow **strict CQRS with Event Sourcing**:

```text
┌─────────────────────────────────────────────────────────────┐
│                        WRITE SIDE                            │
│                                                              │
│  Self-Discovery → Commands → Command Handlers → Events      │
│                                           ↓                  │
│                                      JetStream               │
│                                      (Source of Truth)       │
└─────────────────────────────────────────────────────────────┘
                                    │
                    ┌──────────┼──────────┼──────────┼──────────┐
                    ↓          ↓          ↓          ↓          ↓
┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│  READ SIDE  │ │  READ SIDE  │ │  READ SIDE  │ │  READ SIDE  │ │  READ SIDE  │
│             │ │             │ │             │ │             │ │             │
│    Neo4j    │ │   NetBox    │ │  Conceptual │ │     SQL     │ │   Metrics   │
│  (Topology) │ │   (DCIM)    │ │   Spaces    │ │  (Reports)  │ │ (Telemetry) │
│             │ │ 10.0.224.131│ │  (Semantic) │ │ (Analytics) │ │ (Time Series│
└─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘
```

## Components

### 1. Write Side: Commands → Events

**Commands** (Intent to change state):
- `RegisterCompute` - Register a compute resource (server, VM, container)
- `DefineNetwork` - Define a network segment
- `EstablishConnection` - Connect two interfaces
- `ConfigureSoftware` - Configure software on a compute resource
- `SetPolicy` - Set a policy rule

**Command Handlers**:
- Validate commands
- Check invariants
- Produce domain events

**Events** (Facts that happened):
- `ComputeRegistered`
- `NetworkDefined`
- `ConnectionEstablished`
- `SoftwareConfigured`
- `PolicySet`

All events follow `cim-domain` protocol:
- EventId (UUID v7 for time-ordering)
- MessageIdentity (correlation_id, causation_id, message_id)
- Persisted to JetStream as source of truth

### 2. Event Store: JetStream

**Stream**: `INFRASTRUCTURE_EVENTS`
- Subjects: `infrastructure.{aggregate}.{operation}`
- Persistence: File-based, 30 day retention
- Replay capability: Full event history available

**Consumer Groups**:
- `neo4j-projector` - Projects to graph database
- `sql-projector` - Projects to relational database
- `metrics-projector` - Projects to time-series database

### 3. Read Side: Projections (Event Handlers → Read Models)

**Neo4j Projection** (Topology & Relationships):
- Nodes: ComputeResource, Network, Interface, Software, Policy
- Relationships: HAS_INTERFACE, CONNECTED_TO, ROUTES_TO, RUNS, ENFORCES
- Use cases: Network topology queries, routing path analysis, graph algorithms

**NetBox Projection** (DCIM - Data Center Infrastructure Management):
- Location: http://10.0.224.131
- Models: Devices, Interfaces, IP Addresses, Prefixes, Cables, Sites, Racks
- Use cases: Source of truth for infrastructure, asset management, IP address management (IPAM), cable tracking, documentation
- Integration: REST API with token authentication

**Conceptual Spaces Projection** (Semantic Understanding):
- Quality dimensions: Performance, Capacity, Reliability
- Use cases: Similarity detection, classification, anomaly detection, predictive analysis

**SQL Projection** (Reports & Analytics):
- Tables: Resources, Networks, Connections, Software, Policies
- Use cases: Reporting, auditing, compliance checks, business intelligence

**Metrics Projection** (Time Series):
- Metrics: Resource counts over time, connection patterns, utilization trends
- Use cases: Monitoring, alerting, capacity planning, trend analysis

### 4. Self-Introspection

The CIM continuously discovers its own state:

**Discovery Modules**:
- `src/introspection/compute.rs` - Discover running processes, system info
- `src/introspection/network.rs` - Scan network interfaces, connections
- `src/introspection/software.rs` - Detect installed software

**Discovery Flow**:
```text
System Discovery → Generate Commands → Command Handlers → Events → Projections
```

**Example**:
1. Network interface discovered: `eth0` with IP `192.168.1.10`
2. Generate command: `RegisterCompute` with interface details
3. Command handler validates and produces: `ComputeRegistered` event
4. Event published to JetStream subject: `infrastructure.compute.registered`
5. All projection adapters receive event and update their read models

## Event Replay

Events are the source of truth. Read models can be rebuilt at any time:

**Rebuild Process**:
1. Clear projection read model
2. Subscribe to JetStream from beginning of stream
3. Apply all events in sequence to projection adapter
4. Projection is now consistent with event history

**Use Cases**:
- Recover from projection failures
- Add new projection types
- Fix bugs in projection logic
- Audit and compliance

## Functorial Projections

Projections are **Functors** `F: EventStream → DatabaseState`:

**Category Theory Properties**:
- **Identity**: `F(∅) = ∅` (empty event stream → no changes)
- **Composition**: `F(e1 ∘ e2) = F(e1) ∘ F(e2)` (sequential events compose)

**Projection Adapter Trait**:
```rust
#[async_trait]
pub trait ProjectionAdapter {
    type Event: Send + Sync;
    type Error: std::error::Error + Send + Sync;

    async fn project(&mut self, event: Self::Event) -> Result<(), Self::Error>;
    async fn initialize(&mut self) -> Result<(), Self::Error>;
    async fn health_check(&self) -> Result<(), Self::Error>;
    async fn reset(&mut self) -> Result<(), Self::Error>;
    fn name(&self) -> &str;
}
```

## Domain Protocol

We follow `cim-domain` strictly:

1. **Commands** implement `Command` trait with `aggregate_id()`
2. **Events** use `EventId` (UUID v7) and `MessageIdentity`
3. **Message tracking** via `CorrelationId` and `CausationId`
4. **Factory pattern** via `MessageFactory` for creating identities
5. **Event handlers** process events asynchronously
6. **Projections** are idempotent and eventually consistent

## Example Flow

**Discovering a New Server**:

```rust
// 1. Self-introspection discovers server
let hostname = std::env::var("HOSTNAME")?;
let ip = discover_primary_ip()?;

// 2. Generate command
let command = RegisterCompute {
    hostname,
    resource_type: ComputeType::PhysicalServer,
    interfaces: vec![
        NetworkInterface {
            name: "eth0".to_string(),
            ip_address: ip,
            mac_address: discover_mac()?,
        }
    ],
};

// 3. Command handler validates and produces event
let event = ComputeRegistered {
    id: Uuid::now_v7(),
    identity: MessageFactory::create_root_command(command_id),
    aggregate_id: compute_id,
    hostname: command.hostname,
    resource_type: command.resource_type,
    interfaces: command.interfaces,
    timestamp: Utc::now(),
};

// 4. Publish to JetStream
jetstream.publish("infrastructure.compute.registered", &event).await?;

// 5. Projections receive and process event
// Neo4j: CREATE (r:ComputeResource {id: $id, hostname: $hostname})
// SQL: INSERT INTO resources (id, hostname, type) VALUES (...)
// Metrics: INCREMENT resource_count
```

## Benefits

1. **Auditability**: Every state change is recorded as an event
2. **Reproducibility**: Can rebuild any read model from events
3. **Scalability**: Read models can be independently scaled
4. **Flexibility**: Add new projections without changing write side
5. **Self-documenting**: Event stream is a complete history
6. **Testability**: Can test projections against recorded events
7. **Self-awareness**: System understands its own infrastructure

## Conceptual Spaces Integration

Infrastructure concepts are modeled in **geometric semantic spaces** using `cim-domain-spaces`.

### Infrastructure Concepts

**Base Concepts** (from `cim-domain-spaces`):
- `ComputeResource` - Servers, VMs, containers
- `NetworkSegment` - Network topology elements
- `Connection` - Physical/logical connections
- `Software` - Running programs and configurations
- `Policy` - Rules and constraints

### Quality Dimensions

Infrastructure concepts live in multi-dimensional quality spaces:

**Performance Space**:
- CPU utilization
- Memory usage
- Network throughput
- Disk I/O

**Capacity Space**:
- Storage available
- Memory total
- Network bandwidth
- Connection limits

**Reliability Space**:
- Uptime percentage
- Error rates
- Latency variance
- Redundancy level

### Similarity and Classification

Using Gärdenfors' Conceptual Spaces theory, we can:

1. **Classify Resources**: Determine if a resource is a "server", "container", or "VM" based on its position in quality space
2. **Find Similar Resources**: Use Voronoi diagrams to find resources with similar characteristics
3. **Detect Anomalies**: Identify resources that fall outside expected regions
4. **Predict Behavior**: Use prototypes to predict performance characteristics

### Event → Concept Mapping

```rust
// Event from write side
ComputeRegistered {
    id,
    hostname,
    cpu_cores: 8,
    memory_gb: 32,
    storage_tb: 2,
}

↓ Project to Conceptual Space

// Concept in semantic layer
Concept {
    id: "server-01",
    quality_vector: [
        capacity.cpu → 8.0,
        capacity.memory → 32.0,
        capacity.storage → 2000.0,
        performance.cpu → 0.15,  // current utilization
    ],
    prototype: "physical_server",
    similarity_threshold: 0.85,
}
```

### Conceptual Space Projections

Conceptual Spaces are themselves a projection target:

```text
Events → Conceptual Space Projection
  ↓
Update concept positions in quality dimensions
  ↓
Reason about similarity, classification, predictions
```

This allows the system to:
- **Understand** its own infrastructure semantically
- **Compare** resources across different dimensions
- **Learn** typical patterns and detect anomalies
- **Predict** behavior based on prototypes

## Next Steps

1. ✅ Core projection system (Functor trait)
2. ✅ JetStream configuration
3. ✅ NATS subject patterns
4. ✅ Neo4j projection adapter
5. ✅ Conceptual Spaces integration
6. ⏳ Infrastructure commands and events (following cim-domain protocol)
7. ⏳ Command handlers
8. ⏳ Self-introspection modules
9. ⏳ Event replay mechanism
10. ⏳ Conceptual Space projection adapter
11. ⏳ Additional projection adapters (SQL, Metrics)
12. ⏳ Integration tests with real JetStream
