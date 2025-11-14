// Copyright 2025 Cowboy AI, LLC.

//! Infrastructure Aggregate
//!
//! The Infrastructure aggregate is the root entity that maintains consistency
//! for all infrastructure resources, networks, configurations, and policies.
//! It handles commands and emits events following event sourcing principles.

use super::commands::*;
use super::events::*;
use super::value_objects::*;
use std::collections::HashMap;

/// Infrastructure aggregate - the domain model root
#[derive(Debug, Clone)]
pub struct InfrastructureAggregate {
    /// Aggregate ID
    pub id: InfrastructureId,

    /// Current version (event count)
    pub version: u64,

    /// Compute resources indexed by ID
    pub resources: HashMap<ResourceId, ComputeResource>,

    /// Network interfaces indexed by ID
    pub interfaces: HashMap<InterfaceId, NetworkInterface>,

    /// Networks indexed by ID
    pub networks: HashMap<NetworkId, Network>,

    /// Physical connections
    pub connections: Vec<PhysicalConnection>,

    /// Software configurations indexed by ID
    pub configurations: HashMap<ConfigurationId, SoftwareConfiguration>,

    /// Applied policies indexed by ID
    pub policies: HashMap<PolicyId, PolicyRule>,

    /// Uncommitted events (to be published)
    pub uncommitted_events: Vec<InfrastructureEvent>,
}

impl InfrastructureAggregate {
    /// Create a new Infrastructure aggregate
    pub fn new(id: InfrastructureId) -> Self {
        Self {
            id,
            version: 0,
            resources: HashMap::new(),
            interfaces: HashMap::new(),
            networks: HashMap::new(),
            connections: Vec::new(),
            configurations: HashMap::new(),
            policies: HashMap::new(),
            uncommitted_events: Vec::new(),
        }
    }

    /// Load aggregate from event history
    pub fn from_events(id: InfrastructureId, events: Vec<InfrastructureEvent>) -> Self {
        let mut aggregate = Self::new(id);

        for event in events {
            aggregate.apply_event(&event);
        }

        aggregate
    }

    /// Get uncommitted events and clear the list
    pub fn take_uncommitted_events(&mut self) -> Vec<InfrastructureEvent> {
        std::mem::take(&mut self.uncommitted_events)
    }

    // ========================================================================
    // Command Handlers
    // ========================================================================

    /// Handle RegisterComputeResource command
    pub fn handle_register_compute_resource(&mut self, cmd: ComputeResourceSpec, identity: &MessageIdentity) -> Result<()> {
        // Business rules
        if self.resources.contains_key(&cmd.id) {
            return Err(InfrastructureError::ValidationError(
                format!("Resource {} already exists", cmd.id),
            ));
        }

        // Create event
        let event = InfrastructureEvent::compute_resource_registered(
            identity.correlation_id,
            identity.causation_id,
            ComputeResource {
                id: cmd.id,
                resource_type: cmd.resource_type,
                hostname: cmd.hostname,
                system: cmd.system,
                capabilities: cmd.capabilities,
                interfaces: vec![],
                services: vec![],
                guests: vec![],
            },
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Handle AddInterface command
    pub fn handle_add_interface(&mut self, cmd: InterfaceSpec, identity: &MessageIdentity) -> Result<()> {
        // Business rules
        if !self.resources.contains_key(&cmd.resource_id) {
            return Err(InfrastructureError::ValidationError(
                format!("Resource {} does not exist", cmd.resource_id),
            ));
        }

        if self.interfaces.contains_key(&cmd.id) {
            return Err(InfrastructureError::ValidationError(
                format!("Interface {} already exists", cmd.id),
            ));
        }

        // Validate network exists if specified
        if let Some(ref network_id) = cmd.network_id {
            if !self.networks.contains_key(network_id) {
                return Err(InfrastructureError::ValidationError(
                    format!("Network {} does not exist", network_id),
                ));
            }
        }

        // Create event
        let event = InfrastructureEvent::interface_added(
            identity.correlation_id,
            identity.causation_id,
            NetworkInterface {
                id: cmd.id.clone(),
                resource_id: cmd.resource_id.clone(),
                network_id: cmd.network_id,
                addresses: cmd.addresses,
            },
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Handle DefineNetwork command
    pub fn handle_define_network(&mut self, cmd: NetworkSpec, identity: &MessageIdentity) -> Result<()> {
        // Business rules
        if self.networks.contains_key(&cmd.id) {
            return Err(InfrastructureError::ValidationError(
                format!("Network {} already exists", cmd.id),
            ));
        }

        // Create event
        let event = InfrastructureEvent::network_defined(
            identity.correlation_id,
            identity.causation_id,
            Network {
                id: cmd.id,
                name: cmd.name,
                cidr_v4: cmd.cidr_v4,
                cidr_v6: cmd.cidr_v6,
            },
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Handle DefineNetworkTopology command
    pub fn handle_define_network_topology(&mut self, cmd: NetworkTopologySpec, identity: &MessageIdentity) -> Result<()> {
        // Validate all resources exist
        for conn in &cmd.connections {
            if !self.resources.contains_key(&conn.from_resource) {
                return Err(InfrastructureError::ValidationError(
                    format!("Resource {} does not exist", conn.from_resource),
                ));
            }
            if !self.resources.contains_key(&conn.to_resource) {
                return Err(InfrastructureError::ValidationError(
                    format!("Resource {} does not exist", conn.to_resource),
                ));
            }
        }

        // Convert specs to entities
        let networks: Vec<Network> = cmd
            .networks
            .into_iter()
            .map(|spec| Network {
                id: spec.id,
                name: spec.name,
                cidr_v4: spec.cidr_v4,
                cidr_v6: spec.cidr_v6,
            })
            .collect();

        let connections: Vec<PhysicalConnection> = cmd
            .connections
            .into_iter()
            .map(|spec| PhysicalConnection {
                from_resource: spec.from_resource,
                from_interface: spec.from_interface,
                to_resource: spec.to_resource,
                to_interface: spec.to_interface,
            })
            .collect();

        // Create event
        let event = InfrastructureEvent::network_topology_defined(
            identity.correlation_id,
            identity.causation_id,
            cmd.topology_id,
            networks,
            connections,
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Handle ConnectResources command
    pub fn handle_connect_resources(&mut self, cmd: ConnectionSpec, identity: &MessageIdentity) -> Result<()> {
        // Business rules
        if !self.resources.contains_key(&cmd.from_resource) {
            return Err(InfrastructureError::ValidationError(
                format!("Resource {} does not exist", cmd.from_resource),
            ));
        }

        if !self.resources.contains_key(&cmd.to_resource) {
            return Err(InfrastructureError::ValidationError(
                format!("Resource {} does not exist", cmd.to_resource),
            ));
        }

        // Create event
        let event = InfrastructureEvent::resources_connected(
            identity.correlation_id,
            identity.causation_id,
            PhysicalConnection {
                from_resource: cmd.from_resource,
                from_interface: cmd.from_interface,
                to_resource: cmd.to_resource,
                to_interface: cmd.to_interface,
            },
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Handle ConfigureSoftware command
    pub fn handle_configure_software(&mut self, cmd: SoftwareConfigurationSpec, identity: &MessageIdentity) -> Result<()> {
        // Business rules
        if !self.resources.contains_key(&cmd.resource_id) {
            return Err(InfrastructureError::ValidationError(
                format!("Resource {} does not exist", cmd.resource_id),
            ));
        }

        let configuration_id = ConfigurationId::new();

        // Create event
        let event = InfrastructureEvent::software_configured(
            identity.correlation_id,
            identity.causation_id,
            SoftwareConfiguration {
                id: configuration_id,
                resource_id: cmd.resource_id.clone(),
                software: SoftwareArtifact {
                    id: cmd.software.id.clone(),
                    name: cmd.software.name,
                    version: cmd.software.version,
                    derivation_path: cmd.software.derivation_path,
                },
                configuration_data: cmd.configuration_data,
                dependencies: cmd.dependencies,
            },
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Handle ApplyPolicy command
    pub fn handle_apply_policy(&mut self, cmd: PolicyRuleSpec, identity: &MessageIdentity) -> Result<()> {
        // Validate scope
        match &cmd.scope {
            PolicyScope::Resource(id) => {
                if !self.resources.contains_key(id) {
                    return Err(InfrastructureError::ValidationError(
                        format!("Resource {} does not exist", id),
                    ));
                }
            }
            PolicyScope::Network(id) => {
                if !self.networks.contains_key(id) {
                    return Err(InfrastructureError::ValidationError(
                        format!("Network {} does not exist", id),
                    ));
                }
            }
            PolicyScope::Global => {}
        }

        let policy_id = PolicyId::new();

        // Create event
        let event = InfrastructureEvent::policy_applied(
            identity.correlation_id,
            identity.causation_id,
            PolicyRule {
                id: policy_id,
                name: cmd.name,
                scope: cmd.scope,
                rules: cmd.rules,
            },
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Handle UpdateResource command
    pub fn handle_update_resource(&mut self, resource_id: ResourceId, updates: ResourceUpdates, identity: &MessageIdentity) -> Result<()> {
        // Business rules
        if !self.resources.contains_key(&resource_id) {
            return Err(InfrastructureError::ValidationError(
                format!("Resource {} does not exist", resource_id),
            ));
        }

        // Create event
        let event = InfrastructureEvent::resource_updated(
            identity.correlation_id,
            identity.causation_id,
            resource_id,
            updates,
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Handle RemoveResource command
    pub fn handle_remove_resource(&mut self, resource_id: ResourceId, reason: String, identity: &MessageIdentity) -> Result<()> {
        // Business rules
        if !self.resources.contains_key(&resource_id) {
            return Err(InfrastructureError::ValidationError(
                format!("Resource {} does not exist", resource_id),
            ));
        }

        // Create event
        let event = InfrastructureEvent::resource_removed(
            identity.correlation_id,
            identity.causation_id,
            resource_id,
            reason,
        );

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    // ========================================================================
    // Event Application (State Changes)
    // ========================================================================

    /// Apply an event to update aggregate state
    pub fn apply_event(&mut self, event: &InfrastructureEvent) {
        match event {
            InfrastructureEvent::ComputeResourceRegistered { resource, .. } => {
                self.resources.insert(resource.id.clone(), resource.clone());
            }

            InfrastructureEvent::InterfaceAdded { interface, .. } => {
                self.interfaces.insert(interface.id.clone(), interface.clone());

                // Add interface to resource
                if let Some(resource) = self.resources.get_mut(&interface.resource_id) {
                    resource.interfaces.push(interface.id.clone());
                }
            }

            InfrastructureEvent::NetworkDefined { network, .. } => {
                self.networks.insert(network.id.clone(), network.clone());
            }

            InfrastructureEvent::NetworkTopologyDefined {
                networks,
                connections,
                ..
            } => {
                for network in networks {
                    self.networks.insert(network.id.clone(), network.clone());
                }
                for connection in connections {
                    self.connections.push(connection.clone());
                }
            }

            InfrastructureEvent::ResourcesConnected { connection, .. } => {
                self.connections.push(connection.clone());
            }

            InfrastructureEvent::SoftwareConfigured { configuration, .. } => {
                self.configurations
                    .insert(configuration.id, configuration.clone());

                // Add software to resource services
                if let Some(resource) = self.resources.get_mut(&configuration.resource_id) {
                    if !resource.services.contains(&configuration.software.id) {
                        resource.services.push(configuration.software.id.clone());
                    }
                }
            }

            InfrastructureEvent::PolicyApplied { policy, .. } => {
                self.policies.insert(policy.id, policy.clone());
            }

            InfrastructureEvent::ResourceUpdated {
                resource_id,
                updates,
                ..
            } => {
                if let Some(resource) = self.resources.get_mut(resource_id) {
                    if let Some(hostname) = &updates.hostname {
                        resource.hostname = hostname.clone();
                    }
                    if let Some(capabilities) = &updates.capabilities {
                        resource.capabilities = capabilities.clone();
                    }
                    if let Some(metadata) = &updates.metadata {
                        resource.capabilities.metadata = metadata.clone();
                    }
                }
            }

            InfrastructureEvent::ResourceRemoved { resource_id, .. } => {
                self.resources.remove(resource_id);

                // Remove interfaces
                self.interfaces.retain(|_, iface| &iface.resource_id != resource_id);

                // Remove connections
                self.connections.retain(|conn| {
                    &conn.from_resource != resource_id && &conn.to_resource != resource_id
                });

                // Remove configurations
                self.configurations
                    .retain(|_, config| &config.resource_id != resource_id);
            }
        }

        self.version += 1;
    }

    // ========================================================================
    // Query Methods
    // ========================================================================

    /// Get a resource by ID
    pub fn get_resource(&self, id: &ResourceId) -> Option<&ComputeResource> {
        self.resources.get(id)
    }

    /// Get all resources
    pub fn get_all_resources(&self) -> Vec<&ComputeResource> {
        self.resources.values().collect()
    }

    /// Get a network by ID
    pub fn get_network(&self, id: &NetworkId) -> Option<&Network> {
        self.networks.get(id)
    }

    /// Get all networks
    pub fn get_all_networks(&self) -> Vec<&Network> {
        self.networks.values().collect()
    }

    /// Get all compute resources (alias for graph module)
    pub fn compute_resources(&self) -> Vec<&ComputeResource> {
        self.get_all_resources()
    }

    /// Get all networks (alias for graph module)
    pub fn networks(&self) -> Vec<&Network> {
        self.get_all_networks()
    }

    /// Get all connections
    pub fn connections(&self) -> &[PhysicalConnection] {
        &self.connections
    }

    /// Get interfaces for a resource
    pub fn get_resource_interfaces(&self, resource_id: &ResourceId) -> Vec<&NetworkInterface> {
        self.interfaces
            .values()
            .filter(|iface| &iface.resource_id == resource_id)
            .collect()
    }

    /// Get connections for a resource
    pub fn get_resource_connections(&self, resource_id: &ResourceId) -> Vec<&PhysicalConnection> {
        self.connections
            .iter()
            .filter(|conn| &conn.from_resource == resource_id || &conn.to_resource == resource_id)
            .collect()
    }

    /// Get configurations for a resource
    pub fn get_resource_configurations(&self, resource_id: &ResourceId) -> Vec<&SoftwareConfiguration> {
        self.configurations
            .values()
            .filter(|config| &config.resource_id == resource_id)
            .collect()
    }

    /// Get policies for a scope
    pub fn get_policies_for_scope(&self, scope: &PolicyScope) -> Vec<&PolicyRule> {
        self.policies
            .values()
            .filter(|policy| &policy.scope == scope)
            .collect()
    }

    /// Get count of resources by type
    pub fn count_resources_by_type(&self, resource_type: ComputeType) -> usize {
        self.resources
            .values()
            .filter(|r| r.resource_type == resource_type)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_aggregate_creation() {
        let id = InfrastructureId::new();
        let aggregate = InfrastructureAggregate::new(id);

        assert_eq!(aggregate.id, id);
        assert_eq!(aggregate.version, 0);
        assert_eq!(aggregate.resources.len(), 0);
    }

    #[test]
    fn test_register_compute_resource() {
        let mut aggregate = InfrastructureAggregate::new(InfrastructureId::new());
        let identity = MessageIdentity::new_root();

        let spec = ComputeResourceSpec {
            id: ResourceId::new("server01").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("server01.local").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::new(),
        };

        let result = aggregate.handle_register_compute_resource(spec.clone(), &identity);
        assert!(result.is_ok());

        assert_eq!(aggregate.version, 1);
        assert_eq!(aggregate.resources.len(), 1);
        assert_eq!(aggregate.uncommitted_events.len(), 1);

        let resource = aggregate.get_resource(&spec.id).unwrap();
        assert_eq!(resource.hostname.as_str(), "server01.local");
    }

    #[test]
    fn test_duplicate_resource_fails() {
        let mut aggregate = InfrastructureAggregate::new(InfrastructureId::new());
        let identity = MessageIdentity::new_root();

        let spec = ComputeResourceSpec {
            id: ResourceId::new("server01").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("server01.local").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::new(),
        };

        aggregate.handle_register_compute_resource(spec.clone(), &identity).unwrap();

        // Try to register again
        let result = aggregate.handle_register_compute_resource(spec, &identity);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_interface() {
        let mut aggregate = InfrastructureAggregate::new(InfrastructureId::new());
        let identity = MessageIdentity::new_root();

        // First register a resource
        let resource_spec = ComputeResourceSpec {
            id: ResourceId::new("server01").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("server01.local").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::new(),
        };

        aggregate.handle_register_compute_resource(resource_spec, &identity).unwrap();

        // Then add an interface
        let interface_spec = InterfaceSpec {
            id: InterfaceId::new("eth0").unwrap(),
            resource_id: ResourceId::new("server01").unwrap(),
            network_id: None,
            addresses: vec![],
        };

        let result = aggregate.handle_add_interface(interface_spec.clone(), &identity);
        assert!(result.is_ok());

        assert_eq!(aggregate.interfaces.len(), 1);

        let resource = aggregate.get_resource(&ResourceId::new("server01").unwrap()).unwrap();
        assert_eq!(resource.interfaces.len(), 1);
    }

    #[test]
    fn test_define_network() {
        let mut aggregate = InfrastructureAggregate::new(InfrastructureId::new());
        let identity = MessageIdentity::new_root();

        let spec = NetworkSpec {
            id: NetworkId::new("lan").unwrap(),
            name: "Production LAN".into(),
            cidr_v4: Some("10.0.1.0/24".parse().unwrap()),
            cidr_v6: None,
        };

        let result = aggregate.handle_define_network(spec.clone(), &identity);
        assert!(result.is_ok());

        assert_eq!(aggregate.networks.len(), 1);

        let network = aggregate.get_network(&spec.id).unwrap();
        assert_eq!(network.name, "Production LAN");
    }

    #[test]
    fn test_event_sourcing() {
        let id = InfrastructureId::new();

        // Create some events
        let events = vec![
            InfrastructureEvent::compute_resource_registered(
                Uuid::now_v7(),
                None,
                ComputeResource {
                    id: ResourceId::new("server01").unwrap(),
                    resource_type: ComputeType::Physical,
                    hostname: Hostname::new("server01.local").unwrap(),
                    system: SystemArchitecture::x86_64_linux(),
                    capabilities: ResourceCapabilities::new(),
                    interfaces: vec![],
                    services: vec![],
                    guests: vec![],
                },
            ),
            InfrastructureEvent::network_defined(
                Uuid::now_v7(),
                None,
                Network {
                    id: NetworkId::new("lan").unwrap(),
                    name: "LAN".into(),
                    cidr_v4: Some("10.0.1.0/24".parse().unwrap()),
                    cidr_v6: None,
                },
            ),
        ];

        // Rebuild aggregate from events
        let aggregate = InfrastructureAggregate::from_events(id, events);

        assert_eq!(aggregate.version, 2);
        assert_eq!(aggregate.resources.len(), 1);
        assert_eq!(aggregate.networks.len(), 1);
    }

    #[test]
    fn test_remove_resource() {
        let mut aggregate = InfrastructureAggregate::new(InfrastructureId::new());
        let identity = MessageIdentity::new_root();

        // Register resource
        let spec = ComputeResourceSpec {
            id: ResourceId::new("server01").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("server01.local").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::new(),
        };

        aggregate.handle_register_compute_resource(spec.clone(), &identity).unwrap();
        assert_eq!(aggregate.resources.len(), 1);

        // Remove resource
        let result = aggregate.handle_remove_resource(
            spec.id.clone(),
            "Decommissioned".into(),
            &identity,
        );

        assert!(result.is_ok());
        assert_eq!(aggregate.resources.len(), 0);
        assert_eq!(aggregate.version, 2);
    }
}
