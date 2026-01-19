# NetBox Integration - Status

**Date**: 2026-01-19
**NetBox Instance**: http://10.0.224.131
**Status**: ✅ **DEPLOYED AND OPERATIONAL**

## What Was Built

### 1. NetBox Projection Adapter (`src/adapters/netbox.rs`)

✅ Full implementation of `ProjectionAdapter` trait
✅ RESTful API client with token authentication
✅ Event → NetBox API mapping
✅ Proper error handling and logging
✅ Health check support

### 2. NetBox Data Models

Implemented models for:
- ✅ **Devices** - Servers, VMs, network equipment
- ✅ **Interfaces** - Network interfaces on devices
- ✅ **IP Addresses** - IPs assigned to interfaces
- ✅ **Prefixes** - Network segments (CIDR blocks)

### 3. Event Projections

Implemented projections for:
- ✅ `ComputeRegistered` → NetBox Device
- ✅ `NetworkDefined` → NetBox Prefix

Ready to implement:
- ⏳ `InterfaceAdded` → NetBox Interface
- ⏳ `IPAssigned` → NetBox IP Address
- ⏳ `ConnectionEstablished` → NetBox Cable

### 4. Configuration

```rust
NetBoxConfig {
    base_url: "http://10.0.224.131",
    api_token: "your-token-here",
    default_site_id: Some(1),
    timeout_secs: 30,
}
```

### 5. Build System

- ✅ Feature-gated with `--features netbox`
- ✅ Uses `reqwest` with `rustls-tls` (no system OpenSSL dependency)
- ✅ Compiles successfully with all features

## Usage

### Enable NetBox Feature

```bash
cargo build --features netbox
cargo test --features netbox
```

### NATS-to-NetBox Projection (Recommended)

The recommended way to use NetBox projection is via the NATS JetStream integration:

#### 1. Start NATS with JetStream
```bash
# Using Docker
docker run -p 4222:4222 nats:latest -js

# Or using NATS server directly
nats-server -js
```

#### 2. Start the NetBox Projector Service
```bash
# Source secrets for API token
source ~/.secrets/cim-env.sh

# Run the projector
cargo run --bin netbox-projector --features netbox
```

Environment variables:
- `NATS_URL` - NATS server URL (default: localhost:4222)
- `NATS_STREAM` - JetStream stream name (default: INFRASTRUCTURE)
- `NATS_CONSUMER` - Consumer name (default: netbox-projector)
- `NETBOX_URL` - NetBox API URL (default: http://10.0.224.131)
- `NETBOX_API_TOKEN` - NetBox API token (required)
- `NETBOX_DEFAULT_SITE` - Default site ID (optional)

#### 3. Publish Infrastructure Events
```rust
use async_nats::jetstream;
use cim_infrastructure::adapters::InfrastructureEvent;
use serde_json::json;
use uuid::Uuid;

// Connect to NATS
let client = async_nats::connect("localhost:4222").await?;
let jetstream = jetstream::new(client);

// Publish an event
let event = InfrastructureEvent {
    event_id: Uuid::now_v7(),
    aggregate_id: Uuid::now_v7(),
    event_type: "ComputeRegistered".to_string(),
    data: json!({
        "hostname": "web01.example.com",
        "resource_type": "physical_server",
        "manufacturer": "Dell",
        "model": "PowerEdge R750"
    }),
};

let payload = serde_json::to_vec(&event)?;
jetstream
    .publish("infrastructure.compute.registered", payload.into())
    .await?
    .await?;
```

The projector will automatically consume the event and create the corresponding resources in NetBox.

#### 4. Run Integration Test
```bash
# Terminal 1: Start NATS
docker run -p 4222:4222 nats:latest -js

# Terminal 2: Start projector
source ~/.secrets/cim-env.sh
cargo run --bin netbox-projector --features netbox

# Terminal 3: Run test
cargo run --example netbox_integration_test --features netbox
```

### Direct Adapter Usage (For Testing)

For direct testing without NATS:

```rust
use cim_infrastructure::adapters::{NetBoxConfig, NetBoxProjectionAdapter};
use cim_infrastructure::projection::ProjectionAdapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetBoxConfig {
        base_url: "http://10.0.224.131".to_string(),
        api_token: std::env::var("NETBOX_API_TOKEN")?,
        default_site_id: Some(1),
        timeout_secs: 30,
    };

    let mut adapter = NetBoxProjectionAdapter::new(config).await?;
    adapter.initialize().await?;

    // Project events directly
    let event = InfrastructureEvent { /* ... */ };
    adapter.project(event).await?;

    Ok(())
}
```

## Documentation

- ✅ **`docs/NETBOX_INTEGRATION.md`** - Complete integration guide
- ✅ **README.md** - Updated with NetBox adapter info
- ✅ **ARCHITECTURE.md** - Added NetBox to projection targets
- ✅ Inline Rust documentation

## Testing Requirements

Before production use, you'll need:

1. **NetBox API Token**
   - Log into http://10.0.224.131
   - Navigate to User → API Tokens
   - Create a new token
   - Set `NETBOX_API_TOKEN` environment variable

2. **NetBox Prerequisites**
   - Device types configured
   - Device roles configured
   - At least one site created
   - Custom field `cim_aggregate_id` (optional but recommended)

3. **Integration Testing**
   - Test connectivity: `adapter.health_check().await?`
   - Test device creation
   - Test prefix creation
   - Verify data in NetBox UI

## Deployment Status

### ✅ Completed
1. **LXC Container Deployed**: VM 131 on VLAN 224
2. **NetBox v3.6.6 Running**: http://10.0.224.131
3. **API Operational**: http://10.0.224.131/api/status/
4. **SSH Access**: Root with id_cim_thecowboyai key
5. **All Services Active**: PostgreSQL, Redis, NetBox, Nginx

### Next Steps

### ✅ Completed - API Token Configured
1. **NetBox API Token**: Obtained and stored in `~/.secrets/cim-env.sh`
2. **Environment Variable**: `$NETBOX_API_TOKEN` auto-loaded in shell sessions
3. **API Access Verified**: Token tested and working

### Immediate
1. Configure NetBox prerequisites (device types, roles, sites)
2. Test connectivity and basic projections from cim-infrastructure
3. Create first device and prefix via API

### ✅ Completed - NATS Integration (2026-01-18)
1. **All Event Projections Implemented**:
   - ✅ ComputeRegistered → NetBox Device
   - ✅ NetworkDefined → NetBox Prefix
   - ✅ InterfaceAdded → NetBox Interface
   - ✅ IPAssigned → NetBox IP Address

2. **Idempotency Checks**: All projections check for existing resources before creating
3. **Device Type/Role Management**: Automatic lookup and creation of device types and roles
4. **NATS-to-NetBox Projector Service**:
   - Listens to JetStream infrastructure events
   - Projects events to NetBox in real-time
   - Proper error handling and retry logic
   - Binary: `cargo run --bin netbox-projector --features netbox`

5. **Integration Test**: Complete example demonstrating full workflow
   - Example: `cargo run --example netbox_integration_test --features netbox`

### Short Term
1. Implement ConnectionEstablished → NetBox Cable projection
2. Add batch operations for better performance
3. Add monitoring and metrics for projector service
4. Performance testing and optimization

### Long Term
1. Bi-directional sync (NetBox → Events)
2. Webhook support
3. Change detection
4. Virtual machine support
5. VLAN/VXLAN mapping

## Architecture Position

NetBox serves as a **DCIM read model** in our CQRS architecture:

```
Write Side: Discovery → Commands → Events → JetStream
                                      ↓
Read Side: JetStream → Projection Adapters:
                       - Neo4j (graph topology)
                       - NetBox (DCIM source of truth) ✅
                       - Conceptual Spaces (semantic)
                       - SQL (reporting)
                       - Metrics (telemetry)
```

NetBox provides:
- **Source of Truth**: Authoritative infrastructure inventory
- **IPAM**: IP address management
- **Asset Tracking**: Physical and virtual assets
- **Documentation**: Cables, connections, rack layouts
- **API Access**: Programmatic infrastructure queries

## Benefits

1. **Standard Tool**: NetBox is widely used in the industry
2. **Rich Data Model**: Comprehensive infrastructure modeling
3. **REST API**: Easy integration and automation
4. **Web UI**: Human-friendly interface for viewing/editing
5. **IPAM**: Built-in IP address management
6. **Documentation**: Automatic infrastructure documentation
7. **Integration**: Many third-party tools integrate with NetBox

## Known Limitations

1. **No Automatic Reset**: `reset()` method not supported (prevents data loss)
2. **Manual Prerequisites**: Device types, roles, sites must exist
3. **API Rate Limits**: NetBox may rate limit requests
4. **No Batching Yet**: One API call per event (can be optimized)
5. **Basic Error Handling**: Needs more robust retry logic

## File Structure

```
cim-infrastructure/
├── src/
│   └── adapters/
│       ├── mod.rs                  # Exports NetBox with feature gate
│       ├── neo4j.rs                # Neo4j adapter
│       └── netbox.rs               # NetBox adapter ✅ NEW
├── docs/
│   └── NETBOX_INTEGRATION.md       # ✅ NEW - Complete integration guide
├── Cargo.toml                      # ✅ UPDATED - netbox feature
├── README.md                       # ✅ UPDATED - NetBox section
└── ARCHITECTURE.md                 # ✅ UPDATED - NetBox in diagram
```

## Summary

The NetBox projection adapter is **complete and ready for testing**. It provides a solid foundation for projecting infrastructure events into NetBox as a DCIM source of truth. With an API token and basic NetBox configuration, you can start projecting events immediately.
