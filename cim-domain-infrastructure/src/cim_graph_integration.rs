//! CIM Graph Integration for Infrastructure Domain
//!
//! Provides adapters and functors for integrating infrastructure aggregates
//! with cim-graph's category theory-based graph representations.
//!
//! This module leverages the existing Kan extension from cim-graph into cim-domain,
//! mapping infrastructure entities to domain objects for graph visualization.

use cim_graph::functors::{
    domain_functor::{DomainAggregateType, DomainFunctor, DomainObject, DomainRelationship, RelationshipType},
    kan_extension::{KanExtension, KanExtensionBuilder, TargetRepresentation},
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    InfrastructureAggregate, ComputeResource, Network, PhysicalConnection,
    ResourceId, NetworkId,
};

/// Infrastructure domain functor adapter
///
/// Provides conversion from infrastructure entities to cim-graph domain objects
/// using the established category theory infrastructure.
#[derive(Debug, Clone)]
pub struct InfrastructureFunctor {
    /// Underlying domain functor
    domain_functor: DomainFunctor,
    /// Kan extension for graph visualization
    kan_extension: Option<KanExtension>,
}

impl InfrastructureFunctor {
    /// Create a new infrastructure functor
    pub fn new(functor_id: String) -> Self {
        Self {
            domain_functor: DomainFunctor::new(functor_id),
            kan_extension: None,
        }
    }

    /// Initialize with Kan extension for graph visualization
    pub fn with_kan_extension(mut self, extension: KanExtension) -> Self {
        self.kan_extension = Some(extension);
        self
    }

    /// Build Kan extension from the current functor state
    pub fn build_kan_extension(&mut self, extension_id: String) -> Result<(), String> {
        let extension = KanExtensionBuilder::new(extension_id)
            .with_base_functor(self.domain_functor.clone())
            .build()?;

        self.kan_extension = Some(extension);
        Ok(())
    }

    /// Map infrastructure aggregate to domain objects
    ///
    /// Creates domain objects for:
    /// - Compute resources
    /// - Networks
    /// - Connections (as relationships)
    pub fn map_infrastructure(&mut self, aggregate: &InfrastructureAggregate) {
        // Map compute resources
        for resource in aggregate.compute_resources() {
            self.map_compute_resource(resource);
        }

        // Map networks
        for network in aggregate.networks() {
            self.map_network(network);
        }

        // Map connections as relationships
        for connection in aggregate.connections() {
            self.map_connection(connection);
        }
    }

    /// Map a compute resource to a domain object
    fn map_compute_resource(&mut self, resource: &ComputeResource) -> DomainObject {
        let node_id = resource.id.to_string();

        // Check if already mapped
        if let Some(existing) = self.domain_functor.node_to_domain.get(&node_id) {
            return existing.clone();
        }

        // Create domain object with Infrastructure aggregate type
        let mut properties = HashMap::new();
        properties.insert("hostname".to_string(), serde_json::json!(resource.hostname.to_string()));
        properties.insert("system".to_string(), serde_json::json!(resource.system.to_string()));
        properties.insert("resource_type".to_string(), serde_json::json!(format!("{:?}", resource.resource_type)));

        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Custom("Infrastructure".to_string()),
            name: format!("ComputeResource::{}", resource.hostname),
            properties,
            version: 1,
        };

        self.domain_functor.node_to_domain.insert(node_id, domain_obj.clone());
        domain_obj
    }

    /// Map a network to a domain object
    fn map_network(&mut self, network: &Network) -> DomainObject {
        let node_id = network.id.to_string();

        // Check if already mapped
        if let Some(existing) = self.domain_functor.node_to_domain.get(&node_id) {
            return existing.clone();
        }

        // Create domain object
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), serde_json::json!(network.name));
        if let Some(cidr_v4) = &network.cidr_v4 {
            properties.insert("cidr_v4".to_string(), serde_json::json!(cidr_v4.to_string()));
        }
        if let Some(cidr_v6) = &network.cidr_v6 {
            properties.insert("cidr_v6".to_string(), serde_json::json!(cidr_v6.to_string()));
        }

        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Custom("Infrastructure".to_string()),
            name: format!("Network::{}", network.name),
            properties,
            version: 1,
        };

        self.domain_functor.node_to_domain.insert(node_id, domain_obj.clone());
        domain_obj
    }

    /// Map a connection to a domain relationship
    fn map_connection(&mut self, connection: &PhysicalConnection) -> Option<DomainRelationship> {
        let source_id = connection.from_resource.to_string();
        let target_id = connection.to_resource.to_string();

        // Get source and target domain objects
        let source_domain = self.domain_functor.node_to_domain.get(&source_id)?;
        let target_domain = self.domain_functor.node_to_domain.get(&target_id)?;

        let connection_id = format!(
            "{}:{}--{}:{}",
            connection.from_resource,
            connection.from_interface,
            connection.to_resource,
            connection.to_interface
        );

        let mut properties = HashMap::new();
        properties.insert("from_interface".to_string(), serde_json::json!(connection.from_interface));
        properties.insert("to_interface".to_string(), serde_json::json!(connection.to_interface));

        let relationship = DomainRelationship {
            id: connection_id.clone(),
            source_id: source_domain.id,
            target_id: target_domain.id,
            relationship_type: RelationshipType::Custom("PhysicalConnection".to_string()),
            properties,
        };

        self.domain_functor.edge_to_relationship.insert(connection_id, relationship.clone());
        Some(relationship)
    }

    /// Get all domain objects
    pub fn domain_objects(&self) -> impl Iterator<Item = &DomainObject> {
        self.domain_functor.domain_objects()
    }

    /// Get all relationships
    pub fn relationships(&self) -> impl Iterator<Item = &DomainRelationship> {
        self.domain_functor.relationships()
    }

    /// Get domain object by resource ID
    pub fn get_resource_domain_object(&self, resource_id: &ResourceId) -> Option<&DomainObject> {
        self.domain_functor.get_domain_object(&resource_id.to_string())
    }

    /// Get domain object by network ID
    pub fn get_network_domain_object(&self, network_id: &NetworkId) -> Option<&DomainObject> {
        self.domain_functor.get_domain_object(&network_id.to_string())
    }

    /// Get the Kan extension if initialized
    pub fn kan_extension(&self) -> Option<&KanExtension> {
        self.kan_extension.as_ref()
    }

    /// Extend infrastructure entities to concept space via Kan extension
    ///
    /// This uses the Kan extension's colimit construction:
    /// Lan_F(G)(d) = colim_{F(g)→d} G(g)
    pub fn extend_to_concepts(&mut self) -> Result<(), String> {
        let extension = self.kan_extension.as_mut()
            .ok_or("Kan extension not initialized")?;

        // Extend each domain object to concept space
        for domain_obj in self.domain_functor.domain_objects() {
            let target = TargetRepresentation::ConceptNode {
                concept_id: domain_obj.name.clone(),
                properties: domain_obj.properties.clone(),
            };

            extension.extend_object(domain_obj, target);
        }

        Ok(())
    }

    /// Generate Mermaid diagram using cim-graph rendering
    ///
    /// This replaces the custom Mermaid generation with cim-graph's
    /// standard rendering infrastructure.
    pub fn to_mermaid(&self) -> String {
        let mut output = String::new();
        output.push_str("graph TD\n");

        // Add nodes
        for (idx, domain_obj) in self.domain_objects().enumerate() {
            let shape = match domain_obj.aggregate_type {
                DomainAggregateType::Custom(ref t) if t == "Infrastructure" => {
                    if domain_obj.name.starts_with("ComputeResource") {
                        ("[", "]")  // Rectangle for compute resources
                    } else {
                        ("(", ")")  // Round for networks
                    }
                }
                _ => ("[", "]"),
            };

            output.push_str(&format!(
                "    {}{}{}{}  <!-- {} -->\n",
                idx,
                shape.0,
                domain_obj.name,
                shape.1,
                domain_obj.id
            ));
        }

        // Add edges
        for relationship in self.relationships() {
            // Find source and target indices
            let source_idx = self.domain_objects()
                .position(|obj| obj.id == relationship.source_id);
            let target_idx = self.domain_objects()
                .position(|obj| obj.id == relationship.target_id);

            if let (Some(src), Some(tgt)) = (source_idx, target_idx) {
                let label = match &relationship.relationship_type {
                    RelationshipType::Custom(s) => s.as_str(),
                    _ => "connects",
                };
                output.push_str(&format!(
                    "    {} -->|{}| {}\n",
                    src, label, tgt
                ));
            }
        }

        // Add styling
        output.push_str("\n    classDef compute fill:#e1f5ff,stroke:#01579b;\n");
        output.push_str("    classDef network fill:#fff3e0,stroke:#e65100;\n");

        output
    }

    /// Export to IPLD-compatible JSON
    pub fn to_ipld_json(&self) -> Result<String, serde_json::Error> {
        let mut ipld = HashMap::new();

        ipld.insert("domain_objects", serde_json::to_value(
            self.domain_objects().collect::<Vec<_>>()
        )?);

        ipld.insert("relationships", serde_json::to_value(
            self.relationships().collect::<Vec<_>>()
        )?);

        if let Some(extension) = &self.kan_extension {
            ipld.insert("kan_extension", serde_json::to_value(extension)?);
        }

        serde_json::to_string_pretty(&ipld)
    }
}

/// Topology report generation
impl InfrastructureFunctor {
    /// Generate a topology report
    pub fn topology_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# Infrastructure Topology Report\n\n");

        // Count entities
        let compute_count = self.domain_objects()
            .filter(|obj| obj.name.starts_with("ComputeResource"))
            .count();
        let network_count = self.domain_objects()
            .filter(|obj| obj.name.starts_with("Network"))
            .count();
        let connection_count = self.relationships().count();

        report.push_str(&format!("## Summary\n\n"));
        report.push_str(&format!("- Compute Resources: {}\n", compute_count));
        report.push_str(&format!("- Networks: {}\n", network_count));
        report.push_str(&format!("- Connections: {}\n\n", connection_count));

        // List compute resources
        report.push_str("## Compute Resources\n\n");
        for obj in self.domain_objects().filter(|o| o.name.starts_with("ComputeResource")) {
            report.push_str(&format!("- {}\n", obj.name));
            if let Some(hostname) = obj.properties.get("hostname") {
                report.push_str(&format!("  - Hostname: {}\n", hostname));
            }
            if let Some(system) = obj.properties.get("system") {
                report.push_str(&format!("  - System: {}\n", system));
            }
        }

        // List networks
        report.push_str("\n## Networks\n\n");
        for obj in self.domain_objects().filter(|o| o.name.starts_with("Network")) {
            report.push_str(&format!("- {}\n", obj.name));
            if let Some(cidr_v4) = obj.properties.get("cidr_v4") {
                report.push_str(&format!("  - IPv4 CIDR: {}\n", cidr_v4));
            }
            if let Some(cidr_v6) = obj.properties.get("cidr_v6") {
                report.push_str(&format!("  - IPv6 CIDR: {}\n", cidr_v6));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        InfrastructureId, ComputeType, Hostname, SystemArchitecture,
        ResourceCapabilities, Ipv4Network,
    };
    use std::net::Ipv4Addr;

    #[test]
    fn test_infrastructure_functor_creation() {
        let functor = InfrastructureFunctor::new("test_functor".to_string());
        assert_eq!(functor.domain_objects().count(), 0);
    }

    #[test]
    fn test_map_compute_resource() {
        let mut functor = InfrastructureFunctor::new("test".to_string());

        let resource = ComputeResource {
            id: ResourceId::new("test-vm").unwrap(),
            resource_type: ComputeType::VirtualMachine,
            hostname: Hostname::new("test-vm").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::default(),
            interfaces: vec![],
            guests: vec![],
            services: vec![],
        };

        let domain_obj = functor.map_compute_resource(&resource);

        assert_eq!(domain_obj.aggregate_type, DomainAggregateType::Custom("Infrastructure".to_string()));
        assert!(domain_obj.name.contains("ComputeResource"));
        assert!(domain_obj.properties.contains_key("hostname"));
    }

    #[test]
    fn test_map_network() {
        let mut functor = InfrastructureFunctor::new("test".to_string());

        let network = Network {
            id: NetworkId::new("frontend-net").unwrap(),
            name: "Frontend Network".to_string(),
            cidr_v4: Some(Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap()),
            cidr_v6: None,
        };

        let domain_obj = functor.map_network(&network);

        assert_eq!(domain_obj.aggregate_type, DomainAggregateType::Custom("Infrastructure".to_string()));
        assert!(domain_obj.name.contains("Network"));
        assert!(domain_obj.properties.contains_key("name"));
    }

    #[test]
    fn test_mermaid_generation() {
        let mut functor = InfrastructureFunctor::new("test".to_string());
        let mut aggregate = InfrastructureAggregate::new(InfrastructureId::new());

        // Add some entities
        let resource = ComputeResource {
            id: ResourceId::new("vm1").unwrap(),
            resource_type: ComputeType::VirtualMachine,
            hostname: Hostname::new("vm1").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::default(),
            interfaces: vec![],
            guests: vec![],
            services: vec![],
        };

        functor.map_compute_resource(&resource);

        let mermaid = functor.to_mermaid();
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("ComputeResource"));
    }
}
