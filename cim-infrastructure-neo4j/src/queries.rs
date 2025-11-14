// Copyright 2025 Cowboy AI, LLC.

//! Specialized queries for infrastructure graph analysis

use neo4rs::{Graph, Query};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

use crate::error::Result;

/// Query interface for infrastructure graph
pub struct InfrastructureQueries {
    graph: Arc<Graph>,
}

impl InfrastructureQueries {
    /// Create new query interface
    pub fn new(graph: Arc<Graph>) -> Self {
        Self { graph }
    }

    /// Find shortest routing path between two compute resources
    pub async fn find_routing_path(
        &self,
        from_resource_id: &str,
        to_resource_id: &str,
    ) -> Result<Vec<PathNode>> {
        debug!(
            "Finding routing path from {} to {}",
            from_resource_id, to_resource_id
        );

        let query = Query::new(
            r#"
            MATCH path = shortestPath(
                (from:ComputeResource {id: $from_id})
                -[:HAS_INTERFACE|ROUTES_TO|CONNECTED_TO*]->
                (to:ComputeResource {id: $to_id})
            )
            UNWIND nodes(path) as node
            RETURN node.id as id, labels(node)[0] as label, node.hostname as hostname
            "#
            .to_string(),
        )
        .param("from_id", from_resource_id)
        .param("to_id", to_resource_id);

        let mut result = self.graph.execute(query).await?;
        let mut path_nodes = Vec::new();

        while let Some(row) = result.next().await? {
            let id: String = row.get("id").unwrap_or_default();
            let label: String = row.get("label").unwrap_or_default();
            let hostname: Option<String> = row.get("hostname").ok();

            path_nodes.push(PathNode {
                id,
                node_type: label,
                hostname,
            });
        }

        Ok(path_nodes)
    }

    /// Find all compute resources in a specific network
    pub async fn find_resources_in_network(&self, network_id: &str) -> Result<Vec<ResourceInfo>> {
        debug!("Finding resources in network: {}", network_id);

        let query = Query::new(
            r#"
            MATCH (n:Network {id: $network_id})<-[:CONNECTED_TO]-(i:Interface)<-[:HAS_INTERFACE]-(r:ComputeResource)
            RETURN r.id as id, r.hostname as hostname, r.resource_type as resource_type,
                   collect(i.addresses) as interface_addresses
            "#
            .to_string(),
        )
        .param("network_id", network_id);

        let mut result = self.graph.execute(query).await?;
        let mut resources = Vec::new();

        while let Some(row) = result.next().await? {
            let id: String = row.get("id").unwrap_or_default();
            let hostname: String = row.get("hostname").unwrap_or_default();
            let resource_type: String = row.get("resource_type").unwrap_or_default();

            resources.push(ResourceInfo {
                id,
                hostname,
                resource_type,
            });
        }

        Ok(resources)
    }

    /// Find all networks connected to a compute resource
    pub async fn find_resource_networks(&self, resource_id: &str) -> Result<Vec<NetworkInfo>> {
        debug!("Finding networks for resource: {}", resource_id);

        let query = Query::new(
            r#"
            MATCH (r:ComputeResource {id: $resource_id})-[:HAS_INTERFACE]->(i:Interface)-[:CONNECTED_TO]->(n:Network)
            RETURN DISTINCT n.id as id, n.name as name, n.cidr as cidr
            "#
            .to_string(),
        )
        .param("resource_id", resource_id);

        let mut result = self.graph.execute(query).await?;
        let mut networks = Vec::new();

        while let Some(row) = result.next().await? {
            let id: String = row.get("id").unwrap_or_default();
            let name: String = row.get("name").unwrap_or_default();
            let cidr: Option<String> = row.get("cidr").ok();

            networks.push(NetworkInfo { id, name, cidr });
        }

        Ok(networks)
    }

    /// Find all policies affecting a resource (directly or through networks)
    pub async fn find_resource_policies(&self, resource_id: &str) -> Result<Vec<PolicyInfo>> {
        debug!("Finding policies for resource: {}", resource_id);

        let query = Query::new(
            r#"
            MATCH (r:ComputeResource {id: $resource_id})-[:ENFORCES]->(p:Policy)
            RETURN p.id as id, p.name as name, 'direct' as source
            UNION
            MATCH (r:ComputeResource {id: $resource_id})-[:HAS_INTERFACE]->(:Interface)-[:CONNECTED_TO]->(n:Network)-[:APPLIES]->(p:Policy)
            RETURN p.id as id, p.name as name, n.name as source
            "#
            .to_string(),
        )
        .param("resource_id", resource_id);

        let mut result = self.graph.execute(query).await?;
        let mut policies = Vec::new();

        while let Some(row) = result.next().await? {
            let id: String = row.get("id").unwrap_or_default();
            let name: String = row.get("name").unwrap_or_default();
            let source: String = row.get("source").unwrap_or_default();

            policies.push(PolicyInfo { id, name, source });
        }

        Ok(policies)
    }

    /// Get network topology summary
    pub async fn get_topology_summary(&self) -> Result<TopologySummary> {
        debug!("Getting topology summary");

        let query = Query::new(
            r#"
            OPTIONAL MATCH (r:ComputeResource)
            WITH count(r) as total_resources
            OPTIONAL MATCH (n:Network)
            WITH total_resources, count(n) as total_networks
            OPTIONAL MATCH (i:Interface)
            WITH total_resources, total_networks, count(i) as total_interfaces
            OPTIONAL MATCH ()-[rel:ROUTES_TO]->()
            RETURN total_resources, total_networks, total_interfaces, count(rel) as total_connections
            "#
            .to_string(),
        );

        let mut result = self.graph.execute(query).await?;

        if let Some(row) = result.next().await? {
            let total_resources: i64 = row.get("total_resources").unwrap_or(0);
            let total_networks: i64 = row.get("total_networks").unwrap_or(0);
            let total_interfaces: i64 = row.get("total_interfaces").unwrap_or(0);
            let total_connections: i64 = row.get("total_connections").unwrap_or(0);

            Ok(TopologySummary {
                total_resources: total_resources as usize,
                total_networks: total_networks as usize,
                total_interfaces: total_interfaces as usize,
                total_connections: total_connections as usize,
            })
        } else {
            Ok(TopologySummary::default())
        }
    }

    /// Find isolated resources (not connected to any network)
    pub async fn find_isolated_resources(&self) -> Result<Vec<ResourceInfo>> {
        debug!("Finding isolated resources");

        let query = Query::new(
            r#"
            MATCH (r:ComputeResource)
            WHERE NOT (r)-[:HAS_INTERFACE]->(:Interface)-[:CONNECTED_TO]->(:Network)
            RETURN r.id as id, r.hostname as hostname, r.resource_type as resource_type
            "#
            .to_string(),
        );

        let mut result = self.graph.execute(query).await?;
        let mut resources = Vec::new();

        while let Some(row) = result.next().await? {
            let id: String = row.get("id").unwrap_or_default();
            let hostname: String = row.get("hostname").unwrap_or_default();
            let resource_type: String = row.get("resource_type").unwrap_or_default();

            resources.push(ResourceInfo {
                id,
                hostname,
                resource_type,
            });
        }

        Ok(resources)
    }

    /// Find network segments (groups of interconnected resources)
    pub async fn find_network_segments(&self) -> Result<Vec<NetworkSegment>> {
        debug!("Finding network segments");

        let query = Query::new(
            r#"
            MATCH (n:Network)<-[:CONNECTED_TO]-(:Interface)<-[:HAS_INTERFACE]-(r:ComputeResource)
            WITH n, collect(DISTINCT r.id) as resource_ids, count(DISTINCT r) as resource_count
            RETURN n.id as network_id, n.name as network_name, resource_count, resource_ids
            ORDER BY resource_count DESC
            "#
            .to_string(),
        );

        let mut result = self.graph.execute(query).await?;
        let mut segments = Vec::new();

        while let Some(row) = result.next().await? {
            let network_id: String = row.get("network_id").unwrap_or_default();
            let network_name: String = row.get("network_name").unwrap_or_default();
            let resource_count: i64 = row.get("resource_count").unwrap_or(0);

            segments.push(NetworkSegment {
                network_id,
                network_name,
                resource_count: resource_count as usize,
            });
        }

        Ok(segments)
    }
}

/// Node in a routing path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    pub id: String,
    pub node_type: String,
    pub hostname: Option<String>,
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub id: String,
    pub hostname: String,
    pub resource_type: String,
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub cidr: Option<String>,
}

/// Policy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyInfo {
    pub id: String,
    pub name: String,
    pub source: String,
}

/// Topology summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopologySummary {
    pub total_resources: usize,
    pub total_networks: usize,
    pub total_interfaces: usize,
    pub total_connections: usize,
}

/// Network segment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSegment {
    pub network_id: String,
    pub network_name: String,
    pub resource_count: usize,
}
