// Copyright 2025 Cowboy AI, LLC.

//! Infrastructure Domain Module
//!
//! This module implements the Infrastructure domain using Domain-Driven Design
//! and Event Sourcing principles. The Infrastructure domain models compute
//! resources, network topology, software configurations, and policy rules.
//!
//! ## Architecture
//!
//! The domain follows these principles:
//!
//! 1. **Event Sourcing**: All state changes are represented as immutable events
//! 2. **CQRS**: Commands modify state, events represent what happened
//! 3. **Aggregate Root**: `InfrastructureAggregate` maintains consistency
//! 4. **Value Objects**: Immutable, validated data types
//! 5. **Domain Independence**: No knowledge of Nix or external formats
//!
//! ## Key Concepts
//!
//! - **Compute Resources**: Physical servers, VMs, containers
//! - **Network Topology**: Networks, interfaces, physical connections
//! - **Software Configurations**: Deployed software and its configuration
//! - **Policy Rules**: Security, access, compliance, performance policies
//!
//! ## Usage
//!
//! ```rust
//! use cim_domain_infrastructure::*;
//!
//! // Create aggregate
//! let mut infrastructure = InfrastructureAggregate::new(InfrastructureId::new());
//!
//! // Register a compute resource
//! let identity = MessageIdentity::new_root();
//! let spec = ComputeResourceSpec {
//!     id: ResourceId::new("server01").unwrap(),
//!     resource_type: ComputeType::Physical,
//!     hostname: Hostname::new("server01.example.com").unwrap(),
//!     system: SystemArchitecture::x86_64_linux(),
//!     capabilities: ResourceCapabilities::new(),
//! };
//!
//! infrastructure.handle_register_compute_resource(spec, &identity).unwrap();
//!
//! // Get events
//! let events = infrastructure.take_uncommitted_events();
//! ```

pub mod aggregate;
pub mod cim_graph_integration;
pub mod commands;
pub mod events;

#[cfg(feature = "policy")]
pub mod policy_integration;

pub mod value_objects;

// Re-export commonly used types
pub use aggregate::InfrastructureAggregate;
pub use cim_graph_integration::InfrastructureFunctor;
pub use commands::{
    ConnectionSpec, InfrastructureCommand, InterfaceSpec, MessageIdentity, NetworkSpec,
    NetworkTopologySpec, PolicyRuleSpec, SoftwareArtifactSpec, SoftwareConfigurationSpec,
    ComputeResourceSpec,
};
pub use events::{
    ComputeResource, InfrastructureEvent, Network, NetworkInterface, PhysicalConnection,
    PolicyRule, Rule, RuleType, SoftwareArtifact, SoftwareConfiguration, ResourceUpdates,
};
pub use value_objects::{
    ComputeType, ConfigurationId, Hostname, InfrastructureError, InfrastructureId, InterfaceId,
    Ipv4Network, Ipv6Network, NetworkId, PolicyId, PolicyScope, ResourceCapabilities, ResourceId,
    Result, SoftwareId, SystemArchitecture, TopologyId, Version,
};
