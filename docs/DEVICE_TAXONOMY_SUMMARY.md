# Infrastructure Device Taxonomy - Quick Reference

## Visual Taxonomy Map

```
┌─────────────────────────────────────────────────────────────────────┐
│                    INFRASTRUCTURE RESOURCE TYPES                     │
│                      (Domain Value Object)                           │
└─────────────────────────────────────────────────────────────────────┘
                                   │
            ┌──────────────────────┴──────────────────────┐
            │                                             │
    ┌───────┴────────┐                          ┌────────┴────────┐
    │   Categories   │                          │  NetBox Colors  │
    └────────────────┘                          └─────────────────┘
            │                                             │
    ┌───────┴────────────────────────────────┐          │
    │                                        │          │
    ▼                                        ▼          ▼

🟢 COMPUTE (Green #4caf50)          🔵 NETWORK (Blue #2196f3)
├─ Physical Server                  ├─ Router
├─ Virtual Machine                  ├─ Switch
├─ Container Host                   ├─ Layer 3 Switch
└─ Hypervisor                       ├─ Access Point
                                    └─ Load Balancer

🔴 SECURITY (Red #f44336)           🟠 STORAGE (Orange #ff9800)
├─ Firewall                         ├─ Storage Array
├─ IDS/IPS                          ├─ NAS
├─ VPN Gateway                      └─ SAN Switch
└─ WAF

🟣 EDGE/IOT (Purple #9c27b0)       🟡 POWER (Yellow #ffeb3b)
├─ Edge Device                      ├─ PDU
├─ IoT Gateway                      ├─ UPS
└─ Sensor                           └─ Environmental Monitor

🔵 TELECOM (Cyan #00bcd4)          🟤 APPLIANCE (Brown #795548)
├─ PBX                              ├─ Appliance (Generic)
└─ Video Conference                 ├─ Backup Appliance
                                    ├─ Monitoring Appliance
                                    └─ Auth Server

⚪ OTHER (Grey #9e9e9e)
├─ Other
└─ Unknown
```

## Quick Lookup Table

| Type | String | Category | Use For |
|------|--------|----------|---------|
| `PhysicalServer` | `physical_server` | Compute | Bare metal servers |
| `VirtualMachine` | `virtual_machine` | Compute | VMs, cloud instances |
| `Router` | `router` | Network | Network routers |
| `Switch` | `switch` | Network | Layer 2 switches |
| `Layer3Switch` | `layer3_switch` | Network | L3 switches |
| `Firewall` | `firewall` | Security | Firewalls, security gateways |
| `LoadBalancer` | `load_balancer` | Network | Load balancers |
| `StorageArray` | `storage_array` | Storage | SAN, storage systems |
| `AccessPoint` | `access_point` | Network | WiFi APs |
| `PDU` | `pdu` | Power | Power distribution |

## Common Aliases

```
server, bare_metal        → physical_server
vm                        → virtual_machine
l3_switch                 → layer3_switch
ap, wap                   → access_point
lb, balancer              → load_balancer
fw                        → firewall
ips, intrusion_detection  → ids
vpn                       → vpn_gateway
storage                   → storage_array
san                       → san_switch
```

## Event Example

```json
{
  "event_type": "ComputeRegistered",
  "data": {
    "hostname": "core-router-01.dc1.example.com",
    "resource_type": "router",
    "manufacturer": "Cisco",
    "model": "ASR 1001-X"
  }
}
```

Results in NetBox:
- **Device Name**: core-router-01.dc1.example.com
- **Device Role**: Router (🔵 Blue)
- **Device Type**: Cisco ASR 1001-X

## Code Usage

```rust
use cim_infrastructure::ResourceType;

// Parse from event
let rt = ResourceType::from_str("router");

// Get properties
rt.display_name()     // "Router"
rt.as_str()           // "router"
rt.category()         // ResourceCategory::Network
rt.netbox_color()     // "2196f3"

// Check behaviors
rt.is_network_device()    // true
rt.is_compute_resource()  // false
rt.is_security_device()   // false
```

## Testing

Run the device types example to see all types in action:

```bash
# Terminal 1: Start NATS
docker run -p 4222:4222 nats:latest -js

# Terminal 2: Start NetBox projector
source ~/.secrets/cim-env.sh
cargo run --bin netbox-projector --features netbox

# Terminal 3: Publish test events
cargo run --example netbox_device_types --features netbox
```

Check NetBox UI at http://10.0.224.131/dcim/devices/ to see color-coded devices.
