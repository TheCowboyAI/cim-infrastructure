# NetBox Integration

## Overview

NetBox (https://netbox.dev) is an open-source Data Center Infrastructure Management (DCIM) tool used as the source of truth for network infrastructure.

**NetBox Instance**: `http://10.0.224.131`

## Architecture

NetBox serves as a **read model projection** in our CQRS architecture:

```text
Infrastructure Events → NetBox Projection Adapter → NetBox REST API
                                 ↓
                      NetBox Database (PostgreSQL)
```

### Event → NetBox Mapping

| Infrastructure Event | NetBox API Endpoint | NetBox Model |
|---------------------|--------------------|--------------|
| `ComputeRegistered` | `POST /api/dcim/devices/` | Device |
| `NetworkDefined` | `POST /api/ipam/prefixes/` | Prefix |
| `InterfaceAdded` | `POST /api/dcim/interfaces/` | Interface |
| `IPAssigned` | `POST /api/ipam/ip-addresses/` | IP Address |
| `ConnectionEstablished` | `POST /api/dcim/cables/` | Cable |

## NetBox Data Model

### Devices
Physical servers, VMs, network equipment

```json
{
  "name": "web01.example.com",
  "device_type": 1,
  "device_role": 1,
  "site": 1,
  "status": "active",
  "custom_fields": {
    "cim_aggregate_id": "uuid-here"
  }
}
```

### Interfaces
Network interfaces on devices

```json
{
  "device": 1,
  "name": "eth0",
  "type": "1000base-t",
  "enabled": true,
  "mac_address": "00:11:22:33:44:55",
  "mtu": 1500
}
```

### IP Addresses
IPs assigned to interfaces

```json
{
  "address": "192.168.1.10/24",
  "status": "active",
  "assigned_object_type": "dcim.interface",
  "assigned_object_id": 1,
  "description": "Primary interface"
}
```

### Prefixes
Network segments (CIDR blocks)

```json
{
  "prefix": "192.168.1.0/24",
  "site": 1,
  "status": "active",
  "description": "Management network"
}
```

### Cables
Physical connections between interfaces

```json
{
  "termination_a_type": "dcim.interface",
  "termination_a_id": 1,
  "termination_b_type": "dcim.interface",
  "termination_b_id": 2,
  "type": "cat6",
  "status": "connected"
}
```

## Configuration

### Environment Variables

```bash
export NETBOX_URL="http://10.0.224.131"
export NETBOX_API_TOKEN="your-api-token-here"
export NETBOX_SITE_ID="1"  # Default site for devices
```

### Rust Configuration

```rust
use cim_infrastructure::adapters::NetBoxConfig;

let config = NetBoxConfig {
    base_url: "http://10.0.224.131".to_string(),
    api_token: std::env::var("NETBOX_API_TOKEN")?,
    default_site_id: Some(1),
    timeout_secs: 30,
};
```

## API Authentication

NetBox uses **token-based authentication**:

```http
Authorization: Token 0123456789abcdef0123456789abcdef01234567
```

### Obtaining an API Token

1. Log into NetBox web UI at http://10.0.224.131
2. Navigate to: User → API Tokens
3. Click "Add a Token"
4. Copy the generated token
5. Set environment variable: `export NETBOX_API_TOKEN="..."`

## Usage

### As a Projection Adapter

```rust
use cim_infrastructure::{
    adapters::NetBoxProjectionAdapter,
    projection::ProjectionAdapter,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetBoxConfig {
        base_url: "http://10.0.224.131".to_string(),
        api_token: std::env::var("NETBOX_API_TOKEN")?,
        default_site_id: Some(1),
        timeout_secs: 30,
    };

    let mut adapter = NetBoxProjectionAdapter::new(config).await?;

    // Initialize (creates schemas, verifies connectivity)
    adapter.initialize().await?;

    // Health check
    adapter.health_check().await?;

    // Project events
    let event = InfrastructureEvent {
        event_id: Uuid::now_v7(),
        aggregate_id: Uuid::now_v7(),
        event_type: "ComputeRegistered".to_string(),
        data: serde_json::json!({
            "id": "server-01",
            "hostname": "web01.example.com",
            "resource_type": "physical_server",
        }),
    };

    adapter.project(event).await?;

    Ok(())
}
```

### Cargo Features

Enable NetBox support with the `netbox` feature:

```toml
[dependencies]
cim-infrastructure = { version = "0.1", features = ["netbox"] }
```

Or in Cargo.toml:

```bash
cargo build --features netbox
cargo test --features netbox
```

## Event Replay

NetBox projections can be rebuilt from the event stream:

```rust
async fn rebuild_netbox_projection(
    jetstream: &JetStream,
    netbox: &mut NetBoxProjectionAdapter,
) -> Result<(), Error> {
    // WARNING: This doesn't actually reset NetBox
    // Manual cleanup required or use NetBox's native tools

    // Subscribe to all infrastructure events
    let mut stream = jetstream
        .stream("INFRASTRUCTURE_EVENTS")
        .consumer_from_beginning()
        .await?;

    // Replay all events
    while let Some(event) = stream.next().await {
        netbox.project(event?).await?;
    }

    Ok(())
}
```

**Note**: NetBox projection does NOT support automatic reset via `reset()` method to prevent accidental data loss.

## Bi-Directional Sync

NetBox can serve as both a **source** and **target**:

### NetBox → Events (Discovery)

```rust
// Poll NetBox for changes
async fn discover_from_netbox(
    netbox_client: &NetBoxClient,
) -> Vec<InfrastructureEvent> {
    let devices = netbox_client.get_devices().await?;

    devices.into_iter().map(|device| {
        InfrastructureEvent::ComputeRegistered {
            id: device.custom_fields.cim_aggregate_id,
            hostname: device.name,
            // ...
        }
    }).collect()
}
```

### Events → NetBox (Projection)

```rust
// Already implemented via NetBoxProjectionAdapter
adapter.project(event).await?;
```

## NetBox API Endpoints

### Core Endpoints Used

- **Devices**: `/api/dcim/devices/`
- **Interfaces**: `/api/dcim/interfaces/`
- **Cables**: `/api/dcim/cables/`
- **IP Addresses**: `/api/ipam/ip-addresses/`
- **Prefixes**: `/api/ipam/prefixes/`
- **Status**: `/api/status/` (health check)

### Query Parameters

NetBox API supports filtering:

```http
GET /api/dcim/devices/?site_id=1&status=active
GET /api/ipam/ip-addresses/?parent=192.168.1.0/24
```

### Pagination

Large result sets are paginated:

```json
{
  "count": 150,
  "next": "http://10.0.224.131/api/dcim/devices/?limit=50&offset=50",
  "previous": null,
  "results": [...]
}
```

## Error Handling

### Common Errors

| Error | Cause | Resolution |
|-------|-------|------------|
| `401 Unauthorized` | Invalid API token | Check `NETBOX_API_TOKEN` |
| `404 Not Found` | Invalid endpoint | Verify NetBox version/API |
| `400 Bad Request` | Invalid data | Check event payload format |
| `500 Server Error` | NetBox internal error | Check NetBox logs |

### Projection Error Types

```rust
ProjectionError::TargetUnavailable(_) // NetBox is down
ProjectionError::InvalidEvent(_)       // Event data is malformed
ProjectionError::DatabaseError(_)      // NetBox API returned error
```

## NetBox Schema Requirements

### Device Types

Before projecting compute resources, ensure device types exist:

```bash
# Create a device type via NetBox UI or API
POST /api/dcim/device-types/
{
  "manufacturer": 1,
  "model": "Generic Server",
  "slug": "generic-server"
}
```

### Device Roles

Devices require roles:

```bash
POST /api/dcim/device-roles/
{
  "name": "Server",
  "slug": "server",
  "color": "9e9e9e"
}
```

### Sites

All devices belong to a site:

```bash
POST /api/dcim/sites/
{
  "name": "Main Data Center",
  "slug": "main-dc"
}
```

## Custom Fields

CIM uses custom fields to link NetBox objects to event-sourced aggregates:

```python
# In NetBox admin
Custom Field:
  Name: cim_aggregate_id
  Type: Text
  Object Type: Device
  Description: CIM aggregate UUID
```

## Monitoring

### Health Checks

```rust
// Periodic health check
tokio::spawn(async move {
    loop {
        match adapter.health_check().await {
            Ok(_) => info!("NetBox healthy"),
            Err(e) => error!("NetBox unhealthy: {}", e),
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
});
```

### Metrics

Track projection performance:

- Events projected per second
- API request latency
- Error rates
- Queue depth

## Best Practices

1. **Idempotency**: Events may be replayed - check if object exists before creating
2. **Rate Limiting**: NetBox may rate limit API requests
3. **Batching**: Batch multiple events when possible
4. **Error Recovery**: Implement retry logic with exponential backoff
5. **Validation**: Validate event data before calling NetBox API
6. **Monitoring**: Monitor projection lag and error rates
7. **Custom Fields**: Always populate `cim_aggregate_id` for traceability

## Troubleshooting

### Cannot connect to NetBox

```bash
# Test connectivity
curl http://10.0.224.131/api/status/

# Check API token
curl -H "Authorization: Token $NETBOX_API_TOKEN" \
  http://10.0.224.131/api/dcim/devices/
```

### Events not appearing in NetBox

1. Check projection adapter logs
2. Verify event format matches expected schema
3. Check NetBox API response for validation errors
4. Ensure device types and roles exist

### Performance Issues

1. Enable batching for bulk imports
2. Use async requests for parallel processing
3. Implement connection pooling
4. Consider caching device type/role lookups

## Future Enhancements

- [ ] Bulk import support
- [ ] Change detection (diff events)
- [ ] Bidirectional sync
- [ ] Webhook support (NetBox → Events)
- [ ] Custom field automation
- [ ] Virtual machine support
- [ ] Cluster support
- [ ] VLAN/VXLAN mapping
- [ ] Power tracking
- [ ] Environmental monitoring integration
