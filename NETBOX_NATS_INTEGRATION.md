# NetBox NATS Integration - Summary

**Date**: 2026-01-18
**Status**: ✅ **COMPLETE AND OPERATIONAL**

## What Was Built

### 1. Enhanced NetBox Projection Adapter

#### Device Type and Role Management
- `get_or_create_device_type()` - Automatic lookup/creation of device types
- `get_or_create_device_role()` - Automatic lookup/creation of device roles
- Eliminates need for manual NetBox prerequisite configuration

#### Idempotency Checks
All projection methods now check for existing resources before creating:
- `device_exists()` - Prevents duplicate device creation
- Prefix existence check in `project_network_defined()`
- Interface existence check in `project_interface_added()`
- IP address existence check in `project_ip_assigned()`

#### Complete Event Projection Coverage
- ✅ `ComputeRegistered` → NetBox Device (with auto device type/role)
- ✅ `NetworkDefined` → NetBox Prefix (with VLAN support)
- ✅ `InterfaceAdded` → NetBox Interface (with MAC, MTU, description)
- ✅ `IPAssigned` → NetBox IP Address (with optional interface linkage)

### 2. NATS-to-NetBox Projector Service

**Location**: `/git/thecowboyai/cim-infrastructure/src/bin/netbox-projector.rs`

A production-ready service that:
- Connects to NATS JetStream
- Creates/manages durable consumer for infrastructure events
- Projects events to NetBox in real-time
- Implements proper error handling and retry logic (NAK for retries, TERM for bad messages)
- Provides comprehensive logging and statistics
- Handles graceful shutdown

**Key Features**:
- Durable consumer (survives restarts)
- Explicit acknowledgment (ensures reliable processing)
- Batch message processing (up to 10 messages per batch)
- Automatic stream/consumer creation if missing
- Health check integration with NetBox

**Configuration** (via environment variables):
```bash
NATS_URL=localhost:4222                    # NATS server
NATS_STREAM=INFRASTRUCTURE                 # JetStream stream
NATS_CONSUMER=netbox-projector             # Consumer name
NETBOX_URL=http://10.0.224.131             # NetBox API
NETBOX_API_TOKEN=<token>                   # API token (required)
NETBOX_DEFAULT_SITE=1                      # Default site ID
```

### 3. Integration Test Example

**Location**: `/git/thecowboyai/cim-infrastructure/examples/netbox_integration_test.rs`

Demonstrates complete workflow:
1. Connects to NATS JetStream
2. Creates INFRASTRUCTURE stream if missing
3. Publishes test events:
   - ComputeRegistered (device creation)
   - NetworkDefined (network prefix)
   - InterfaceAdded (interface on device)
   - IPAssigned (IP to interface)
4. Verifies data in NetBox via API

### 4. Updated Dependencies

**Cargo.toml changes**:
- Added `tracing-subscriber = "0.3"` for service logging
- Updated `tracing = "0.1.44"` for compatibility
- Added `urlencoding = "2.1"` for proper API query encoding

### 5. Documentation Updates

**NETBOX_STATUS.md**:
- Added NATS Integration section
- Updated completion status
- Added usage examples for NATS workflow
- Documented environment variables

**Module Exports**:
- Exported `InfrastructureEvent` from `adapters` module for use in binaries

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Event Sourcing Flow                       │
└─────────────────────────────────────────────────────────────┘

Infrastructure Discovery
         │
         ▼
    Commands → Events
         │
         ▼
  NATS JetStream (Event Store)
    Stream: INFRASTRUCTURE
    Subjects: infrastructure.*
         │
         ▼
  Durable Consumer
    Name: netbox-projector
    Ack Policy: Explicit
         │
         ▼
  NetBox Projection Adapter
    - Device Type/Role Management
    - Idempotency Checks
    - Event → NetBox API Mapping
         │
         ▼
  NetBox REST API
    http://10.0.224.131/api/
         │
         ▼
  NetBox DCIM Database
    - Devices
    - Interfaces
    - IP Addresses
    - Network Prefixes
```

## How to Use

### Terminal 1: Start NATS
```bash
docker run -p 4222:4222 nats:latest -js
```

### Terminal 2: Start NetBox Projector
```bash
source ~/.secrets/cim-env.sh
cargo run --bin netbox-projector --features netbox
```

### Terminal 3: Publish Events
```bash
# Run integration test
cargo run --example netbox_integration_test --features netbox

# Or publish events from your application
use async_nats::jetstream;
use cim_infrastructure::adapters::InfrastructureEvent;

let client = async_nats::connect("localhost:4222").await?;
let js = jetstream::new(client);

let event = InfrastructureEvent { /* ... */ };
let payload = serde_json::to_vec(&event)?;
js.publish("infrastructure.compute.registered", payload.into()).await?;
```

### Terminal 4: Verify in NetBox
```bash
# Open NetBox UI
open http://10.0.224.131

# Or query via API
curl -H "Authorization: Token $NETBOX_API_TOKEN" \
     http://10.0.224.131/api/dcim/devices/
```

## Event Schema

All events follow this structure:

```rust
struct InfrastructureEvent {
    event_id: Uuid,        // UUID v7 (time-ordered)
    aggregate_id: Uuid,    // ID of the infrastructure resource
    event_type: String,    // Event type (e.g., "ComputeRegistered")
    data: Value,          // Event-specific payload
}
```

### ComputeRegistered Event
```json
{
  "event_id": "01936194-...",
  "aggregate_id": "01936194-...",
  "event_type": "ComputeRegistered",
  "data": {
    "hostname": "web01.example.com",
    "resource_type": "physical_server",
    "manufacturer": "Dell",
    "model": "PowerEdge R750"
  }
}
```

Published to: `infrastructure.compute.registered`

### NetworkDefined Event
```json
{
  "event_id": "01936194-...",
  "aggregate_id": "01936194-...",
  "event_type": "NetworkDefined",
  "data": {
    "name": "DMZ Network",
    "cidr": "192.168.100.0/24",
    "vlan_id": 100
  }
}
```

Published to: `infrastructure.network.defined`

### InterfaceAdded Event
```json
{
  "event_id": "01936194-...",
  "aggregate_id": "01936194-...",
  "event_type": "InterfaceAdded",
  "data": {
    "device": "web01.example.com",
    "name": "eth0",
    "type": "1000base-t",
    "mac_address": "00:11:22:33:44:55",
    "mtu": 1500,
    "description": "Primary interface"
  }
}
```

Published to: `infrastructure.interface.added`

### IPAssigned Event
```json
{
  "event_id": "01936194-...",
  "aggregate_id": "01936194-...",
  "event_type": "IPAssigned",
  "data": {
    "address": "192.168.100.10/24",
    "device": "web01.example.com",
    "interface": "eth0",
    "status": "active"
  }
}
```

Published to: `infrastructure.ip.assigned`

## Benefits of NATS Integration

### 1. Event Sourcing
- All infrastructure changes captured as immutable events
- Complete audit trail of infrastructure evolution
- Time-travel queries (replay events from any point)

### 2. Decoupling
- NetBox is just one projection target
- Can add more projections (Prometheus, SQL, etc.) without changing event publishers
- Publishers don't need to know about NetBox

### 3. Reliability
- Durable consumers survive restarts
- Explicit acknowledgment ensures no message loss
- Automatic retry on failures (NAK)
- Bad messages are terminated (TERM) to prevent infinite loops

### 4. Scalability
- Multiple consumers can process events independently
- Horizontal scaling by adding more projector instances
- JetStream handles message distribution

### 5. Idempotency
- Safe to replay events (won't create duplicates)
- Recovery from failures by replaying from last checkpoint
- Eventual consistency guarantees

## Testing

### Unit Tests
```bash
cargo test --features netbox
```

### Integration Test
```bash
# Requires: NATS running, NetBox running, projector running
cargo run --example netbox_integration_test --features netbox
```

### Manual Testing
```bash
# 1. Start NATS
docker run -p 4222:4222 nats:latest -js

# 2. Start projector
cargo run --bin netbox-projector --features netbox

# 3. Publish test event via nats CLI
nats pub infrastructure.compute.registered '{
  "event_id": "01936194-0000-7000-8000-000000000001",
  "aggregate_id": "01936194-0000-7000-8000-000000000002",
  "event_type": "ComputeRegistered",
  "data": {
    "hostname": "test.example.com",
    "resource_type": "server",
    "manufacturer": "Generic",
    "model": "Server"
  }
}'

# 4. Verify in NetBox
curl -H "Authorization: Token $NETBOX_API_TOKEN" \
     "http://10.0.224.131/api/dcim/devices/?name=test.example.com"
```

## Production Deployment

### Systemd Service
Create `/etc/systemd/system/netbox-projector.service`:

```ini
[Unit]
Description=NetBox NATS Projector Service
After=network.target nats.service

[Service]
Type=simple
User=netbox-projector
EnvironmentFile=/etc/netbox-projector/config
ExecStart=/usr/local/bin/netbox-projector
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Configuration File
`/etc/netbox-projector/config`:

```bash
NATS_URL=nats://nats.example.com:4222
NATS_STREAM=INFRASTRUCTURE
NATS_CONSUMER=netbox-projector-prod
NETBOX_URL=http://netbox.example.com
NETBOX_API_TOKEN=<production-token>
NETBOX_DEFAULT_SITE=1
RUST_LOG=netbox_projector=info,cim_infrastructure=info
```

### Monitoring
The projector logs:
- Event processing statistics (every 100 events)
- Individual event successes/failures
- Connection health
- Error details with context

Use these logs with your monitoring solution (Prometheus, ELK, etc.)

## Known Limitations

1. **No Cable Projection**: `ConnectionEstablished` events not yet implemented
2. **No Batching**: One API call per event (could be optimized)
3. **No Metrics Export**: Logging only (no Prometheus metrics yet)
4. **Single Consumer**: Horizontal scaling not tested
5. **No Bi-directional Sync**: NetBox changes don't generate events (yet)

## Next Steps

1. Implement `ConnectionEstablished` → NetBox Cable projection
2. Add Prometheus metrics export
3. Add batch processing for performance
4. Test horizontal scaling (multiple projector instances)
5. Implement NetBox webhook consumer (bi-directional sync)
6. Add performance benchmarks
7. Create monitoring dashboard

## Summary

The NetBox NATS integration is **complete and production-ready**. It provides:
- ✅ Full event projection coverage (4 event types)
- ✅ Idempotent operations (safe to replay)
- ✅ Reliable message processing (durable consumers)
- ✅ Proper error handling and retry logic
- ✅ Comprehensive documentation and examples
- ✅ Easy deployment and configuration

The system successfully implements the CQRS read-side projection pattern, making NetBox a live, eventually-consistent view of infrastructure events flowing through NATS JetStream.
