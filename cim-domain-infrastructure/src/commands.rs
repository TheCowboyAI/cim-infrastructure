// Copyright 2025 Cowboy AI, LLC.

//! Infrastructure Domain Commands
//!
//! Commands represent the intent to perform an action in the Infrastructure domain.
//! They are validated before execution and result in events being emitted.

use super::events::*;
use super::value_objects::*;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

// ============================================================================
// Message Identity (for Command/Event correlation)
// ============================================================================

/// Message identity for tracking command lineage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageIdentity {
    /// Unique command ID
    pub command_id: Uuid,
    /// Correlation ID - groups related commands/events
    pub correlation_id: Uuid,
    /// Causation ID - the event that caused this command (if any)
    pub causation_id: Option<Uuid>,
}

impl MessageIdentity {
    /// Create a new root message identity (no parent)
    pub fn new_root() -> Self {
        let id = Uuid::now_v7();
        Self {
            command_id: id,
            correlation_id: id,
            causation_id: None,
        }
    }

    /// Create a child message identity caused by an event
    pub fn caused_by(correlation_id: Uuid, causation_id: Uuid) -> Self {
        Self {
            command_id: Uuid::now_v7(),
            correlation_id,
            causation_id: Some(causation_id),
        }
    }
}

// ============================================================================
// Command Specifications
// ============================================================================

/// Specification for registering a compute resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeResourceSpec {
    /// Unique identifier for the resource
    pub id: ResourceId,
    /// Type of compute resource (Physical, VirtualMachine, Container)
    pub resource_type: ComputeType,
    /// Hostname for the resource
    pub hostname: Hostname,
    /// System architecture (x86_64-linux, aarch64-darwin, etc.)
    pub system: SystemArchitecture,
    /// Resource capabilities and metadata
    pub capabilities: ResourceCapabilities,
}

/// Specification for adding an interface
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceSpec {
    /// Unique identifier for the interface
    pub id: InterfaceId,
    /// Resource this interface belongs to
    pub resource_id: ResourceId,
    /// Optional network this interface is connected to
    pub network_id: Option<NetworkId>,
    /// IP addresses assigned to this interface
    pub addresses: Vec<IpAddr>,
}

/// Specification for defining a network
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSpec {
    /// Unique identifier for the network
    pub id: NetworkId,
    /// Human-readable name for the network
    pub name: String,
    /// Optional IPv4 CIDR block for the network
    pub cidr_v4: Option<Ipv4Network>,
    /// Optional IPv6 CIDR block for the network
    pub cidr_v6: Option<Ipv6Network>,
}

/// Specification for network topology
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkTopologySpec {
    /// Unique identifier for the topology
    pub topology_id: TopologyId,
    /// Networks in this topology
    pub networks: Vec<NetworkSpec>,
    /// Connections between resources in this topology
    pub connections: Vec<ConnectionSpec>,
}

/// Specification for physical connection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionSpec {
    /// Source resource for the connection
    pub from_resource: ResourceId,
    /// Source interface for the connection
    pub from_interface: InterfaceId,
    /// Destination resource for the connection
    pub to_resource: ResourceId,
    /// Destination interface for the connection
    pub to_interface: InterfaceId,
}

/// Specification for software configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareConfigurationSpec {
    /// Resource to configure software on
    pub resource_id: ResourceId,
    /// Software artifact to configure
    pub software: SoftwareArtifactSpec,
    /// Configuration data in JSON format
    pub configuration_data: serde_json::Value,
    /// Software dependencies
    pub dependencies: Vec<SoftwareId>,
}

/// Specification for software artifact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareArtifactSpec {
    /// Unique identifier for the software
    pub id: SoftwareId,
    /// Name of the software package
    pub name: String,
    /// Version of the software
    pub version: Version,
    /// Optional Nix derivation path
    pub derivation_path: Option<String>,
}

/// Specification for policy rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRuleSpec {
    pub name: String,
    pub scope: PolicyScope,
    pub rules: Vec<Rule>,
}

// ============================================================================
// Infrastructure Commands
// ============================================================================

/// Domain commands for Infrastructure aggregate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InfrastructureCommand {
    /// Register a new compute resource
    RegisterComputeResource {
        identity: MessageIdentity,
        spec: ComputeResourceSpec,
    },

    /// Add an interface to a resource
    AddInterface {
        identity: MessageIdentity,
        spec: InterfaceSpec,
    },

    /// Define a network
    DefineNetwork {
        identity: MessageIdentity,
        spec: NetworkSpec,
    },

    /// Define complete network topology
    DefineNetworkTopology {
        identity: MessageIdentity,
        spec: NetworkTopologySpec,
    },

    /// Connect two resources physically
    ConnectResources {
        identity: MessageIdentity,
        spec: ConnectionSpec,
    },

    /// Configure software on a resource
    ConfigureSoftware {
        identity: MessageIdentity,
        spec: SoftwareConfigurationSpec,
    },

    /// Apply a policy rule
    ApplyPolicy {
        identity: MessageIdentity,
        spec: PolicyRuleSpec,
    },

    /// Update a resource
    UpdateResource {
        identity: MessageIdentity,
        resource_id: ResourceId,
        updates: ResourceUpdates,
    },

    /// Remove a resource
    RemoveResource {
        identity: MessageIdentity,
        resource_id: ResourceId,
        reason: String,
    },
}

impl InfrastructureCommand {
    /// Get command ID
    pub fn command_id(&self) -> Uuid {
        match self {
            InfrastructureCommand::RegisterComputeResource { identity, .. } => identity.command_id,
            InfrastructureCommand::AddInterface { identity, .. } => identity.command_id,
            InfrastructureCommand::DefineNetwork { identity, .. } => identity.command_id,
            InfrastructureCommand::DefineNetworkTopology { identity, .. } => identity.command_id,
            InfrastructureCommand::ConnectResources { identity, .. } => identity.command_id,
            InfrastructureCommand::ConfigureSoftware { identity, .. } => identity.command_id,
            InfrastructureCommand::ApplyPolicy { identity, .. } => identity.command_id,
            InfrastructureCommand::UpdateResource { identity, .. } => identity.command_id,
            InfrastructureCommand::RemoveResource { identity, .. } => identity.command_id,
        }
    }

    /// Get correlation ID
    pub fn correlation_id(&self) -> Uuid {
        match self {
            InfrastructureCommand::RegisterComputeResource { identity, .. } => identity.correlation_id,
            InfrastructureCommand::AddInterface { identity, .. } => identity.correlation_id,
            InfrastructureCommand::DefineNetwork { identity, .. } => identity.correlation_id,
            InfrastructureCommand::DefineNetworkTopology { identity, .. } => identity.correlation_id,
            InfrastructureCommand::ConnectResources { identity, .. } => identity.correlation_id,
            InfrastructureCommand::ConfigureSoftware { identity, .. } => identity.correlation_id,
            InfrastructureCommand::ApplyPolicy { identity, .. } => identity.correlation_id,
            InfrastructureCommand::UpdateResource { identity, .. } => identity.correlation_id,
            InfrastructureCommand::RemoveResource { identity, .. } => identity.correlation_id,
        }
    }

    /// Get command type as string
    pub fn command_type(&self) -> &'static str {
        match self {
            InfrastructureCommand::RegisterComputeResource { .. } => "RegisterComputeResource",
            InfrastructureCommand::AddInterface { .. } => "AddInterface",
            InfrastructureCommand::DefineNetwork { .. } => "DefineNetwork",
            InfrastructureCommand::DefineNetworkTopology { .. } => "DefineNetworkTopology",
            InfrastructureCommand::ConnectResources { .. } => "ConnectResources",
            InfrastructureCommand::ConfigureSoftware { .. } => "ConfigureSoftware",
            InfrastructureCommand::ApplyPolicy { .. } => "ApplyPolicy",
            InfrastructureCommand::UpdateResource { .. } => "UpdateResource",
            InfrastructureCommand::RemoveResource { .. } => "RemoveResource",
        }
    }

    /// Validate the command
    pub fn validate(&self) -> Result<()> {
        match self {
            InfrastructureCommand::RegisterComputeResource { spec, .. } => {
                // Validate hostname
                if spec.hostname.as_str().is_empty() {
                    return Err(InfrastructureError::ValidationError(
                        "Hostname cannot be empty".into(),
                    ));
                }
                Ok(())
            }
            InfrastructureCommand::AddInterface { spec, .. } => {
                // Validate interface
                if spec.id.as_str().is_empty() {
                    return Err(InfrastructureError::ValidationError(
                        "Interface ID cannot be empty".into(),
                    ));
                }
                Ok(())
            }
            InfrastructureCommand::DefineNetwork { spec, .. } => {
                // Validate network
                if spec.name.is_empty() {
                    return Err(InfrastructureError::ValidationError(
                        "Network name cannot be empty".into(),
                    ));
                }
                if spec.cidr_v4.is_none() && spec.cidr_v6.is_none() {
                    return Err(InfrastructureError::ValidationError(
                        "Network must have at least one CIDR block".into(),
                    ));
                }
                Ok(())
            }
            InfrastructureCommand::DefineNetworkTopology { spec, .. } => {
                // Validate topology
                if spec.networks.is_empty() {
                    return Err(InfrastructureError::ValidationError(
                        "Topology must define at least one network".into(),
                    ));
                }
                Ok(())
            }
            InfrastructureCommand::ConnectResources { spec, .. } => {
                // Validate connection
                if spec.from_resource == spec.to_resource {
                    return Err(InfrastructureError::ValidationError(
                        "Cannot connect resource to itself".into(),
                    ));
                }
                Ok(())
            }
            InfrastructureCommand::ConfigureSoftware { spec, .. } => {
                // Validate software configuration
                if spec.software.name.is_empty() {
                    return Err(InfrastructureError::ValidationError(
                        "Software name cannot be empty".into(),
                    ));
                }
                Ok(())
            }
            InfrastructureCommand::ApplyPolicy { spec, .. } => {
                // Validate policy
                if spec.name.is_empty() {
                    return Err(InfrastructureError::ValidationError(
                        "Policy name cannot be empty".into(),
                    ));
                }
                if spec.rules.is_empty() {
                    return Err(InfrastructureError::ValidationError(
                        "Policy must have at least one rule".into(),
                    ));
                }
                Ok(())
            }
            InfrastructureCommand::UpdateResource { updates, .. } => {
                // Validate updates
                if updates.hostname.is_none()
                    && updates.capabilities.is_none()
                    && updates.metadata.is_none()
                {
                    return Err(InfrastructureError::ValidationError(
                        "At least one field must be updated".into(),
                    ));
                }
                Ok(())
            }
            InfrastructureCommand::RemoveResource { reason, .. } => {
                // Validate removal
                if reason.is_empty() {
                    return Err(InfrastructureError::ValidationError(
                        "Removal reason cannot be empty".into(),
                    ));
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_identity_root() {
        let identity = MessageIdentity::new_root();
        assert_eq!(identity.command_id, identity.correlation_id);
        assert!(identity.causation_id.is_none());
    }

    #[test]
    fn test_message_identity_caused_by() {
        let correlation_id = Uuid::now_v7();
        let event_id = Uuid::now_v7();

        let identity = MessageIdentity::caused_by(correlation_id, event_id);

        assert_eq!(identity.correlation_id, correlation_id);
        assert_eq!(identity.causation_id, Some(event_id));
        assert_ne!(identity.command_id, correlation_id);
    }

    #[test]
    fn test_register_compute_resource_command() {
        let identity = MessageIdentity::new_root();
        let spec = ComputeResourceSpec {
            id: ResourceId::new("server01").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("server01.example.com").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::new(),
        };

        let cmd = InfrastructureCommand::RegisterComputeResource {
            identity: identity.clone(),
            spec,
        };

        assert_eq!(cmd.command_id(), identity.command_id);
        assert_eq!(cmd.command_type(), "RegisterComputeResource");
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_define_network_validation() {
        let identity = MessageIdentity::new_root();

        // Valid network
        let spec = NetworkSpec {
            id: NetworkId::new("lan").unwrap(),
            name: "LAN".into(),
            cidr_v4: Some("10.0.1.0/24".parse().unwrap()),
            cidr_v6: None,
        };

        let cmd = InfrastructureCommand::DefineNetwork {
            identity: identity.clone(),
            spec,
        };

        assert!(cmd.validate().is_ok());

        // Invalid network (no CIDR)
        let invalid_spec = NetworkSpec {
            id: NetworkId::new("lan").unwrap(),
            name: "LAN".into(),
            cidr_v4: None,
            cidr_v6: None,
        };

        let invalid_cmd = InfrastructureCommand::DefineNetwork {
            identity,
            spec: invalid_spec,
        };

        assert!(invalid_cmd.validate().is_err());
    }

    #[test]
    fn test_connect_resources_validation() {
        let identity = MessageIdentity::new_root();

        // Valid connection
        let spec = ConnectionSpec {
            from_resource: ResourceId::new("server01").unwrap(),
            from_interface: InterfaceId::new("eth0").unwrap(),
            to_resource: ResourceId::new("switch01").unwrap(),
            to_interface: InterfaceId::new("port1").unwrap(),
        };

        let cmd = InfrastructureCommand::ConnectResources {
            identity: identity.clone(),
            spec,
        };

        assert!(cmd.validate().is_ok());

        // Invalid connection (same resource)
        let invalid_spec = ConnectionSpec {
            from_resource: ResourceId::new("server01").unwrap(),
            from_interface: InterfaceId::new("eth0").unwrap(),
            to_resource: ResourceId::new("server01").unwrap(),
            to_interface: InterfaceId::new("eth1").unwrap(),
        };

        let invalid_cmd = InfrastructureCommand::ConnectResources {
            identity,
            spec: invalid_spec,
        };

        assert!(invalid_cmd.validate().is_err());
    }

    #[test]
    fn test_command_serialization() {
        let cmd = InfrastructureCommand::RegisterComputeResource {
            identity: MessageIdentity::new_root(),
            spec: ComputeResourceSpec {
                id: ResourceId::new("server01").unwrap(),
                resource_type: ComputeType::Physical,
                hostname: Hostname::new("server01.local").unwrap(),
                system: SystemArchitecture::x86_64_linux(),
                capabilities: ResourceCapabilities::new(),
            },
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: InfrastructureCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(cmd.command_id(), deserialized.command_id());
        assert_eq!(cmd.command_type(), deserialized.command_type());
    }
}
