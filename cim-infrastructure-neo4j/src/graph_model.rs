// Copyright 2025 Cowboy AI, LLC.

//! Graph model definitions for Neo4j projection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node types in the infrastructure graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// Compute resource (server, VM, container)
    ComputeResource,
    /// Network segment
    Network,
    /// Network interface
    Interface,
    /// Software artifact or configuration
    Software,
    /// Security or compliance policy
    Policy,
}

impl NodeType {
    /// Get the Neo4j label for this node type
    pub fn label(&self) -> &'static str {
        match self {
            NodeType::ComputeResource => "ComputeResource",
            NodeType::Network => "Network",
            NodeType::Interface => "Interface",
            NodeType::Software => "Software",
            NodeType::Policy => "Policy",
        }
    }
}

/// Relationship types in the infrastructure graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Compute resource has a network interface
    HasInterface,
    /// Interface is connected to a network
    ConnectedTo,
    /// Interface routes traffic to another interface
    RoutesTo,
    /// Compute resource runs software
    Runs,
    /// Resource enforces a policy
    Enforces,
    /// Network applies a policy
    Applies,
}

impl RelationshipType {
    /// Get the Neo4j relationship type name
    pub fn type_name(&self) -> &'static str {
        match self {
            RelationshipType::HasInterface => "HAS_INTERFACE",
            RelationshipType::ConnectedTo => "CONNECTED_TO",
            RelationshipType::RoutesTo => "ROUTES_TO",
            RelationshipType::Runs => "RUNS",
            RelationshipType::Enforces => "ENFORCES",
            RelationshipType::Applies => "APPLIES",
        }
    }
}

/// Graph model for infrastructure visualization
pub struct GraphModel;

impl GraphModel {
    /// Generate Cypher query to create a compute resource node
    pub fn create_compute_resource_query(
        _id: &str,
        _resource_type: &str,
        _hostname: &str,
        _properties: HashMap<String, serde_json::Value>,
    ) -> String {

        format!(
            r#"
            MERGE (r:ComputeResource {{id: $id}})
            SET r.resource_type = $resource_type,
                r.hostname = $hostname,
                r.properties = $properties,
                r.updated_at = timestamp()
            RETURN r
            "#
        )
    }

    /// Generate Cypher query to create a network node
    pub fn create_network_query(
        _id: &str,
        _name: &str,
        _cidr: Option<&str>,
        _properties: HashMap<String, serde_json::Value>,
    ) -> String {
        format!(
            r#"
            MERGE (n:Network {{id: $id}})
            SET n.name = $name,
                n.cidr = $cidr,
                n.properties = $properties,
                n.updated_at = timestamp()
            RETURN n
            "#
        )
    }

    /// Generate Cypher query to create an interface node
    pub fn create_interface_query(
        _id: &str,
        _resource_id: &str,
        _addresses: Vec<String>,
        _properties: HashMap<String, serde_json::Value>,
    ) -> String {
        format!(
            r#"
            MERGE (i:Interface {{id: $id}})
            SET i.resource_id = $resource_id,
                i.addresses = $addresses,
                i.properties = $properties,
                i.updated_at = timestamp()
            RETURN i
            "#
        )
    }

    /// Generate Cypher query to create HAS_INTERFACE relationship
    pub fn create_has_interface_relationship_query(
        _resource_id: &str,
        _interface_id: &str,
    ) -> String {
        format!(
            r#"
            MATCH (r:ComputeResource {{id: $resource_id}})
            MATCH (i:Interface {{id: $interface_id}})
            MERGE (r)-[rel:HAS_INTERFACE]->(i)
            SET rel.updated_at = timestamp()
            RETURN rel
            "#
        )
    }

    /// Generate Cypher query to create CONNECTED_TO relationship
    pub fn create_connected_to_relationship_query(
        _interface_id: &str,
        _network_id: &str,
    ) -> String {
        format!(
            r#"
            MATCH (i:Interface {{id: $interface_id}})
            MATCH (n:Network {{id: $network_id}})
            MERGE (i)-[rel:CONNECTED_TO]->(n)
            SET rel.updated_at = timestamp()
            RETURN rel
            "#
        )
    }

    /// Generate Cypher query to create ROUTES_TO relationship (for connections)
    pub fn create_routes_to_relationship_query(
        _from_interface_id: &str,
        _to_interface_id: &str,
    ) -> String {
        format!(
            r#"
            MATCH (from:Interface {{id: $from_interface_id}})
            MATCH (to:Interface {{id: $to_interface_id}})
            MERGE (from)-[rel:ROUTES_TO]->(to)
            SET rel.updated_at = timestamp()
            RETURN rel
            "#
        )
    }

    /// Generate Cypher query to remove a node
    pub fn remove_node_query(node_type: NodeType, _id: &str) -> String {
        format!(
            r#"
            MATCH (n:{} {{id: $id}})
            DETACH DELETE n
            "#,
            node_type.label()
        )
    }

    /// Generate Cypher query to find routing path between two resources
    pub fn find_routing_path_query(
        _from_resource_id: &str,
        _to_resource_id: &str,
    ) -> String {
        format!(
            r#"
            MATCH path = shortestPath(
                (from:ComputeResource {{id: $from_resource_id}})
                -[*]->
                (to:ComputeResource {{id: $to_resource_id}})
            )
            RETURN path
            "#
        )
    }

    /// Generate Cypher query to find all resources in a network
    pub fn find_resources_in_network_query(_network_id: &str) -> String {
        format!(
            r#"
            MATCH (n:Network {{id: $network_id}})<-[:CONNECTED_TO]-(i:Interface)<-[:HAS_INTERFACE]-(r:ComputeResource)
            RETURN r, i
            "#
        )
    }

    /// Generate Cypher query to find all policies affecting a resource
    pub fn find_resource_policies_query(_resource_id: &str) -> String {
        format!(
            r#"
            MATCH (r:ComputeResource {{id: $resource_id}})-[:ENFORCES]->(p:Policy)
            RETURN p
            UNION
            MATCH (r:ComputeResource {{id: $resource_id}})-[:HAS_INTERFACE]->(:Interface)-[:CONNECTED_TO]->(n:Network)-[:APPLIES]->(p:Policy)
            RETURN p
            "#
        )
    }
}
