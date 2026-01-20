// Copyright (c) 2025 - Cowboy AI, Inc.
//! Infrastructure Domain Models
//!
//! Core domain concepts for infrastructure management including resource types,
//! network topology, and value objects with validation invariants that compose
//! with existing CIM domains (Organization, Person, Location, Nix).
//!
//! # Value Objects with Invariants
//!
//! - [`Hostname`] - DNS-validated hostnames (RFC 1123)
//! - [`IpAddressWithCidr`] - IPv4/IPv6 with CIDR notation
//! - [`MacAddress`] - 48-bit MAC address validation
//! - [`VlanId`] - IEEE 802.1Q VLAN ID (1-4094)
//! - [`Mtu`] - Maximum Transmission Unit (68-9000 bytes)
//! - [`ResourceType`] - Infrastructure resource taxonomy
//!
//! # Entities with Domain Composition
//!
//! - [`ComputeResource`] - Compute infrastructure with Organization/Person/Location references
//!
//! # Domain Relationships
//!
//! All entities reference existing CIM domains via `AggregateId`:
//! - `organization_id` → cim-domain-organization
//! - `owner_id` → cim-domain-person
//! - `location_id` → cim-domain-location
//! - NixOS topology integration via cim-domain-nix

pub mod compute_resource;
pub mod hostname;
pub mod invariants;
pub mod network;
pub mod resource_type;

// Re-export value objects
pub use compute_resource::{ComputeResource, ComputeResourceBuilder, ComputeResourceError};
pub use hostname::{Hostname, HostnameError};
pub use invariants::{ValidationError, ValidationResult};
pub use network::{
    IpAddressWithCidr, MacAddress, Mtu, NetworkError, VlanId,
};
pub use resource_type::{ResourceCategory, ResourceType};
