# Network Migration - VLAN 64 to VLAN 224

**Date**: 2026-01-18
**Status**: ✅ Complete

## Summary

Successfully migrated NetBox (VM 131) configuration from legacy VLAN 64 network to production VLAN 224 network to match live Proxmox deployment.

## Problem Statement

NetBox was accessible at `10.0.224.131` in the live Proxmox environment, but configuration repository still referenced the old VLAN 64 address `10.0.64.131`. This created inconsistency between:

1. **Live System**: NetBox running at 10.0.224.131 on VLAN 224
2. **Configuration Repository**: References to 10.0.64.131 on VLAN 64

## Network Details

### VLAN 64 (Legacy - Deprecated)
- **Subnet**: 10.0.64.0/19
- **Gateway**: 10.0.64.1
- **VLAN Tag**: 64
- **Status**: Being phased out

### VLAN 224 (Production - Active)
- **Subnet**: 10.0.224.0/19
- **Gateway**: 10.0.224.1
- **VLAN Tag**: 224
- **Status**: Active production network

## Changes Made

### 1. Created New VM 131 Configuration
**Location**: `/git/thecowboyai/pve/224/131/`

**File**: `131.conf`
```
hostname: netbox
ip=10.0.224.131/19
gw=10.0.224.1
tag=224
```

**Changes from old config**:
- IP: `10.0.64.131` → `10.0.224.131`
- Gateway: `10.0.64.1` → `10.0.224.1`
- VLAN Tag: `64` → `224`

### 2. Documentation
Created comprehensive README.md documenting:
- Network configuration
- Resource allocation
- Migration history
- CIM integration points
- Access information

### 3. Git Repository
Initialized git repository for VM 131 configuration tracking:
- Initial commit documenting migration
- Version control for future changes

## Verification

### Configuration Repository
- ✅ New config at `/git/thecowboyai/pve/224/131/131.conf`
- ✅ Documentation at `/git/thecowboyai/pve/224/131/README.md`
- ✅ Git repository initialized

### CIM Infrastructure Documentation
- ✅ `NETBOX_STATUS.md` correctly references 10.0.224.131
- ✅ `README.md` correctly references 10.0.224.131
- ✅ `docs/NETBOX_INTEGRATION.md` correctly references 10.0.224.131
- ✅ `ARCHITECTURE.md` correctly references NetBox

### Code References
- ✅ `src/adapters/netbox.rs` uses configurable base_url
- ✅ All examples use 10.0.224.131

## Active VMs on VLAN 224

| VM ID | Hostname | IP Address | Purpose |
|-------|----------|------------|---------|
| 109 | cim-nix-dev | 10.0.224.109 | Development |
| 118 | cim-cube-dev | 10.0.224.109 | Development |
| 119 | codespaces | 10.0.224.119 | Development |
| 120 | haystack | 10.0.224.120 | Service |
| 121 | jupyterlab | 10.0.224.121 | Service |
| 122 | bevydev | 10.0.224.122 | Development |
| 124 | gitea | 10.0.224.124 | Service |
| 125 | autogpt | 10.0.224.125 | Service |
| 126 | babyagi | 10.0.224.126 | Service |
| 127 | yew | 10.0.224.127 | Development |
| 130 | go-dev | 10.0.224.130 | Development |
| **131** | **netbox** | **10.0.224.131** | **DCIM** |
| 132 | structurizr | 10.0.224.132 | Service |
| 133 | shuttle-dev | 10.0.224.133 | Development |
| 224100 | kerneldev | 10.0.224.100 | Development |

## Legacy VMs on VLAN 64

The following VMs remain on VLAN 64 and may need migration:
- VM 101
- VM 102
- VM 106
- VM 111 (gateway)
- VM 113
- VM 134
- VM 301

## Gateway Considerations

**VLAN 64 Gateway** (VM 111):
- Location: `/git/thecowboyai/pve/64/111/`
- IP: 10.0.64.111
- Purpose: Legacy network gateway

**VLAN 224 Gateway**:
- Gateway IP: 10.0.224.1
- All VLAN 224 VMs use this as their default gateway
- No dedicated gateway VM needed (handled by network infrastructure)

## NetBox Access

With the migration complete:

1. **Web UI**: http://10.0.224.131
2. **API Endpoint**: http://10.0.224.131/api/
3. **Health Check**: http://10.0.224.131/api/status/

## CIM Integration

The NetBox projection adapter in `cim-infrastructure` is ready to use:

```rust
use cim_infrastructure::adapters::{NetBoxConfig, NetBoxProjectionAdapter};

let config = NetBoxConfig {
    base_url: "http://10.0.224.131".to_string(),
    api_token: std::env::var("NETBOX_API_TOKEN")?,
    default_site_id: Some(1),
    timeout_secs: 30,
};

let mut adapter = NetBoxProjectionAdapter::new(config).await?;
adapter.initialize().await?;
adapter.health_check().await?;
```

## Next Steps

1. **Test NetBox Connectivity**
   ```bash
   curl http://10.0.224.131/api/status/
   ```

2. **Obtain API Token**
   - Log into NetBox UI
   - Navigate to User → API Tokens
   - Create token and set `NETBOX_API_TOKEN`

3. **Test Projection Adapter**
   ```bash
   cd /git/thecowboyai/cim-infrastructure
   cargo test --features netbox
   ```

4. **Consider Migrating Other VLAN 64 VMs**
   - Evaluate which VMs should move to VLAN 224
   - Update configurations accordingly
   - Maintain consistency across infrastructure

## Lessons Learned

1. **Configuration Repository as Source of Truth**: Git-tracked configurations must match live deployment
2. **Network Topology Documentation**: Clear documentation of VLAN structure prevents confusion
3. **Migration Documentation**: Comprehensive migration notes help understand infrastructure evolution
4. **Consistent Naming**: VM directories organized by VLAN number aids navigation

## References

- NetBox Integration: `docs/NETBOX_INTEGRATION.md`
- NetBox Status: `NETBOX_STATUS.md`
- CIM Architecture: `ARCHITECTURE.md`
- Proxmox Configs: `/git/thecowboyai/pve/224/131/`
