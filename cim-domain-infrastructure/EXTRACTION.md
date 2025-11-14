# Infrastructure Domain Extraction

**Date**: 2025-11-13
**Status**: ✅ COMPLETE

## Overview

The Infrastructure domain has been successfully extracted from `cim-domain-nix` into its own standalone crate `cim-domain-infrastructure`. This allows the infrastructure domain to be shared across multiple CIM modules.

## Why Extract?

### Problem
The infrastructure domain was accidentally built inside `cim-domain-nix`, but it's a general-purpose domain that should be:
- **Reusable** across multiple CIM modules
- **Independent** of Nix-specific concerns
- **Versioned** separately from format-specific implementations

### Solution
Extract into `cim-domain-infrastructure` as a standalone crate that:
- ✅ Contains only domain logic (no Nix knowledge)
- ✅ Can be depended on by any CIM module
- ✅ Follows DDD and event sourcing principles
- ✅ Is framework-agnostic

## What Was Extracted

### Files Moved
From `cim-domain-nix/src/infrastructure/` to `cim-domain-infrastructure/src/`:

| File | Lines | Purpose |
|------|-------|---------|
| `aggregate.rs` | 24,868 | InfrastructureAggregate (event-sourced root) |
| `commands.rs` | 17,826 | Command types and specs |
| `events.rs` | 16,904 | Domain events |
| `value_objects.rs` | 17,148 | Value objects (IDs, hostnames, networks, etc.) |
| `lib.rs` (was mod.rs) | 2,670 | Module exports and documentation |

**Total**: ~79KB of domain code

### Domain Model

```
cim-domain-infrastructure/
├── Cargo.toml           # Minimal dependencies (serde, uuid, chrono, thiserror)
├── README.md            # Usage documentation
├── src/
│   ├── lib.rs           # Public API
│   ├── aggregate.rs     # InfrastructureAggregate
│   ├── commands.rs      # Commands and specs
│   ├── events.rs        # Domain events
│   └── value_objects.rs # Value objects and errors
```

### Domain Concepts

**Aggregates:**
- `InfrastructureAggregate` - Event-sourced root entity

**Commands:**
- `RegisterComputeResource`
- `AddInterface`
- `DefineNetwork`
- `EstablishConnection`
- `ConfigureSoftware`
- `SetPolicyRule`

**Events:**
- `ComputeResourceRegistered`
- `InterfaceAdded`
- `NetworkDefined`
- `ConnectionEstablished`
- `SoftwareConfigured`
- `PolicyRuleSet`

**Value Objects:**
- `InfrastructureId`, `ResourceId`, `NetworkId`, `InterfaceId`
- `Hostname` (validated)
- `Ipv4Network`, `Ipv6Network` (CIDR notation)
- `SystemArchitecture` (platform identifier)
- `ResourceCapabilities` (metadata)

## Changes to cim-domain-nix

### Before

```rust
// cim-domain-nix/src/infrastructure/
mod aggregate;
mod commands;
mod events;
mod value_objects;
```

### After

```rust
// cim-domain-nix/Cargo.toml
[dependencies]
cim-domain-infrastructure = { path = "../cim-domain-infrastructure" }

// cim-domain-nix/src/infrastructure.rs
pub use cim_domain_infrastructure::*;
```

The infrastructure module now simply re-exports from the external crate.

## Dependencies

### cim-domain-infrastructure

Minimal dependencies:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.11", features = ["v4", "v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
```

No NATS, no Nix, no external frameworks.

### cim-domain-nix

Now depends on:
```toml
cim-domain-infrastructure = { path = "../cim-domain-infrastructure" }
```

## Test Results

### cim-domain-infrastructure
```bash
$ cd cim-domain-infrastructure
$ cargo build
   Compiling cim-domain-infrastructure v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.58s
```

✅ Builds successfully with minimal dependencies

### cim-domain-nix
```bash
$ cd cim-domain-nix
$ cargo test --lib
test result: ok. 145 passed; 0 failed; 1 ignored
```

✅ All tests pass with external dependency

## Usage

### In cim-domain-nix

```rust
// Works exactly as before
use cim_domain_nix::infrastructure::*;

let mut infra = InfrastructureAggregate::new(InfrastructureId::new());
let identity = MessageIdentity::new_root();

let spec = ComputeResourceSpec { /* ... */ };
infra.handle_register_compute_resource(spec, &identity)?;
```

### In Other CIM Modules

```rust
// Add to Cargo.toml
[dependencies]
cim-domain-infrastructure = { path = "../cim-domain-infrastructure" }

// Use directly
use cim_domain_infrastructure::*;

let mut infra = InfrastructureAggregate::new(InfrastructureId::new());
```

## Benefits

### 1. **Separation of Concerns**
- Infrastructure domain: pure domain logic
- cim-domain-nix: Nix-specific adapters and functors

### 2. **Reusability**
Can be used by:
- `cim-domain-nix` - Nix integration
- `cim-network` - Network visualization
- `cim-domain-policy` - Policy enforcement
- `cim-leaf-*` - Leaf implementations
- Any other CIM module needing infrastructure

### 3. **Independent Versioning**
- Infrastructure domain can evolve independently
- Semantic versioning for API stability
- Clear upgrade paths

### 4. **Reduced Dependencies**
- Infrastructure: 5 dependencies
- cim-domain-nix: still has NATS, rnix, etc. but infrastructure is clean

### 5. **Better Testing**
- Domain logic tested in isolation
- No need for Nix or NATS in infrastructure tests
- Clear boundaries

## Migration Guide

For code using `cim-domain-nix::infrastructure`:

**Option 1: No Changes Required**
```rust
// Still works
use cim_domain_nix::infrastructure::*;
```

**Option 2: Use Direct Dependency**
```rust
// In Cargo.toml
[dependencies]
cim-domain-infrastructure = { path = "../cim-domain-infrastructure" }

// In code
use cim_domain_infrastructure::*;
```

Both approaches work identically.

## Next Steps

### 1. Set up cim-infrastructure Repository

Create a top-level repository to host infrastructure-related modules:
```
cim-infrastructure/
├── README.md
├── Cargo.toml (workspace)
├── cim-domain-infrastructure/  # This crate
└── future modules...
```

### 2. Publish to Crates.io

Once stable:
```bash
cd cim-domain-infrastructure
cargo publish
```

### 3. Update cim-domain-nix

Change from path dependency to published version:
```toml
cim-domain-infrastructure = "0.1"
```

### 4. Use in Other Modules

Any CIM module can now depend on infrastructure:
```toml
cim-domain-infrastructure = "0.1"
```

## Validation

✅ **Extraction Complete**
- All files moved
- Minimal dependencies
- Builds successfully

✅ **Integration Works**
- cim-domain-nix uses external crate
- All 145 tests pass
- No breaking changes

✅ **Documentation Complete**
- README with examples
- API documentation
- Migration guide

## Status

**Infrastructure Domain**: ✅ Extracted and standalone
**cim-domain-nix**: ✅ Updated and tested
**All Tests**: ✅ 145/145 passing

The infrastructure domain is now a proper, reusable CIM module!
