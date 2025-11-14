// Copyright 2025 Cowboy AI, LLC.

//! Neo4j projection builder for infrastructure events

use cim_domain_infrastructure::{
    ComputeResource, InfrastructureEvent, Network, NetworkInterface, PhysicalConnection,
};
use neo4rs::{Graph, Query};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::config::Neo4jConfig;
use crate::error::{Neo4jError, Result};
use crate::subscriber::InfrastructureEventSubscriber;

/// Neo4j projection for infrastructure events
pub struct Neo4jProjection {
    /// Neo4j graph connection
    graph: Arc<Graph>,
    /// Event receiver channel
    event_rx: Option<mpsc::Receiver<InfrastructureEvent>>,
    /// Event sender channel (for subscriber)
    event_tx: mpsc::Sender<InfrastructureEvent>,
}

impl Neo4jProjection {
    /// Connect to Neo4j and create projection
    pub async fn connect(config: Neo4jConfig) -> Result<Self> {
        info!("Connecting to Neo4j at {}", config.uri);

        let graph = Graph::new(&config.uri, &config.user, &config.password)
            .await
            .map_err(|e| Neo4jError::Connection(e.to_string()))?;

        let (event_tx, event_rx) = mpsc::channel(1000);

        Ok(Self {
            graph: Arc::new(graph),
            event_rx: Some(event_rx),
            event_tx,
        })
    }

    /// Initialize Neo4j schema (constraints and indexes)
    pub async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing Neo4j schema");

        // Create uniqueness constraints
        let constraints = vec![
            "CREATE CONSTRAINT IF NOT EXISTS FOR (r:ComputeResource) REQUIRE r.id IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (n:Network) REQUIRE n.id IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (i:Interface) REQUIRE i.id IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (s:Software) REQUIRE s.id IS UNIQUE",
            "CREATE CONSTRAINT IF NOT EXISTS FOR (p:Policy) REQUIRE p.id IS UNIQUE",
        ];

        for constraint in constraints {
            self.graph
                .run(Query::new(constraint.to_string()))
                .await?;
        }

        // Create indexes for common queries
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS FOR (r:ComputeResource) ON (r.hostname)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Network) ON (n.name)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Network) ON (n.cidr)",
        ];

        for index in indexes {
            self.graph
                .run(Query::new(index.to_string()))
                .await?;
        }

        info!("Schema initialization complete");
        Ok(())
    }

    /// Start event subscriber for NATS infrastructure events
    pub async fn start_event_subscriber(&mut self, nats_url: &str) -> Result<()> {
        let subscriber = InfrastructureEventSubscriber::new(nats_url, self.event_tx.clone()).await?;
        subscriber.start().await?;

        // Start projection processor
        if let Some(event_rx) = self.event_rx.take() {
            self.start_projection_processor(event_rx);
        }

        Ok(())
    }

    /// Start projection processor task
    fn start_projection_processor(&self, mut event_rx: mpsc::Receiver<InfrastructureEvent>) {
        let graph = Arc::clone(&self.graph);

        tokio::spawn(async move {
            info!("Starting projection processor");

            while let Some(event) = event_rx.recv().await {
                if let Err(e) = Self::process_event(&graph, event).await {
                    error!("Failed to process event: {}", e);
                }
            }

            info!("Projection processor stopped");
        });
    }

    /// Process a single infrastructure event
    async fn process_event(graph: &Graph, event: InfrastructureEvent) -> Result<()> {
        debug!("Processing event: {:?}", event);

        match event {
            InfrastructureEvent::ComputeResourceRegistered { resource, .. } => {
                Self::project_compute_resource(graph, &resource).await?;
            }
            InfrastructureEvent::ResourceRemoved { resource_id, .. } => {
                Self::remove_compute_resource(graph, &resource_id.to_string()).await?;
            }
            InfrastructureEvent::NetworkDefined { network, .. } => {
                Self::project_network(graph, &network).await?;
            }
            InfrastructureEvent::InterfaceAdded { interface, .. } => {
                Self::project_interface(graph, &interface).await?;
            }
            InfrastructureEvent::ResourcesConnected { connection, .. } => {
                Self::project_connection(graph, &connection).await?;
            }
            InfrastructureEvent::ResourceUpdated { .. } => {
                debug!("Resource updates not yet projected");
            }
            InfrastructureEvent::SoftwareConfigured { .. } => {
                debug!("Software configuration projection not yet implemented");
            }
            InfrastructureEvent::PolicyApplied { .. } => {
                debug!("Policy rule projection not yet implemented");
            }
            InfrastructureEvent::NetworkTopologyDefined { .. } => {
                debug!("Topology definition projection not yet implemented");
            }
        }

        Ok(())
    }

    /// Project a compute resource to Neo4j
    async fn project_compute_resource(graph: &Graph, resource: &ComputeResource) -> Result<()> {
        debug!("Projecting compute resource: {}", resource.id);

        let cpu_cores = resource.capabilities.cpu_cores.unwrap_or(0) as i64;
        let memory_mb = resource.capabilities.memory_mb.unwrap_or(0) as i64;

        let query = Query::new(
            r#"
            MERGE (r:ComputeResource {id: $id})
            SET r.resource_type = $resource_type,
                r.hostname = $hostname,
                r.system = $system,
                r.cpu_cores = $cpu_cores,
                r.memory_mb = $memory_mb,
                r.updated_at = timestamp()
            RETURN r
            "#
                .to_string(),
        )
        .param("id", resource.id.to_string())
        .param("resource_type", format!("{:?}", resource.resource_type))
        .param("hostname", resource.hostname.to_string())
        .param("system", format!("{:?}", resource.system))
        .param("cpu_cores", cpu_cores)
        .param("memory_mb", memory_mb);

        graph.run(query).await?;
        debug!("Compute resource projected successfully");

        Ok(())
    }

    /// Remove a compute resource from Neo4j
    async fn remove_compute_resource(graph: &Graph, resource_id: &str) -> Result<()> {
        debug!("Removing compute resource: {}", resource_id);

        let query = Query::new(
            r#"
            MATCH (r:ComputeResource {id: $id})
            DETACH DELETE r
            "#
            .to_string(),
        )
        .param("id", resource_id.to_string());

        graph.run(query).await?;
        Ok(())
    }

    /// Project a network to Neo4j
    async fn project_network(graph: &Graph, network: &Network) -> Result<()> {
        debug!("Projecting network: {}", network.id);

        let cidr = network
            .cidr_v4
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "".to_string());

        let query = Query::new(
            r#"
            MERGE (n:Network {id: $id})
            SET n.name = $name,
                n.cidr = $cidr,
                n.updated_at = timestamp()
            RETURN n
            "#
            .to_string(),
        )
        .param("id", network.id.to_string())
        .param("name", network.name.clone())
        .param("cidr", cidr);

        graph.run(query).await?;
        debug!("Network projected successfully");

        Ok(())
    }

    /// Remove a network from Neo4j
    async fn remove_network(graph: &Graph, network_id: &str) -> Result<()> {
        debug!("Removing network: {}", network_id);

        let query = Query::new(
            r#"
            MATCH (n:Network {id: $id})
            DETACH DELETE n
            "#
            .to_string(),
        )
        .param("id", network_id.to_string());

        graph.run(query).await?;
        Ok(())
    }

    /// Project a network interface to Neo4j
    async fn project_interface(graph: &Graph, interface: &NetworkInterface) -> Result<()> {
        debug!("Projecting interface: {}", interface.id);

        let addresses: Vec<String> = interface.addresses.iter().map(|a| a.to_string()).collect();
        let addresses_json = serde_json::to_string(&addresses)?;

        // Create interface node
        let query = Query::new(
            r#"
            MERGE (i:Interface {id: $id})
            SET i.resource_id = $resource_id,
                i.addresses = $addresses,
                i.updated_at = timestamp()
            RETURN i
            "#
            .to_string(),
        )
        .param("id", interface.id.to_string())
        .param("resource_id", interface.resource_id.to_string())
        .param("addresses", addresses_json);

        graph.run(query).await?;

        // Create HAS_INTERFACE relationship
        let rel_query = Query::new(
            r#"
            MATCH (r:ComputeResource {id: $resource_id})
            MATCH (i:Interface {id: $interface_id})
            MERGE (r)-[rel:HAS_INTERFACE]->(i)
            SET rel.updated_at = timestamp()
            "#
            .to_string(),
        )
        .param("resource_id", interface.resource_id.to_string())
        .param("interface_id", interface.id.to_string());

        graph.run(rel_query).await?;

        // If interface is connected to a network, create CONNECTED_TO relationship
        if let Some(network_id) = &interface.network_id {
            let net_query = Query::new(
                r#"
                MATCH (i:Interface {id: $interface_id})
                MATCH (n:Network {id: $network_id})
                MERGE (i)-[rel:CONNECTED_TO]->(n)
                SET rel.updated_at = timestamp()
                "#
                .to_string(),
            )
            .param("interface_id", interface.id.to_string())
            .param("network_id", network_id.to_string());

            graph.run(net_query).await?;
        }

        debug!("Interface projected successfully");
        Ok(())
    }

    /// Remove an interface from Neo4j
    async fn remove_interface(graph: &Graph, interface_id: &str) -> Result<()> {
        debug!("Removing interface: {}", interface_id);

        let query = Query::new(
            r#"
            MATCH (i:Interface {id: $id})
            DETACH DELETE i
            "#
            .to_string(),
        )
        .param("id", interface_id.to_string());

        graph.run(query).await?;
        Ok(())
    }

    /// Project a physical connection to Neo4j
    async fn project_connection(graph: &Graph, connection: &PhysicalConnection) -> Result<()> {
        debug!(
            "Projecting connection: {} -> {}",
            connection.from_interface, connection.to_interface
        );

        // Create ROUTES_TO relationship between interfaces
        let query = Query::new(
            r#"
            MATCH (from:Interface {id: $from_interface})
            MATCH (to:Interface {id: $to_interface})
            MERGE (from)-[rel:ROUTES_TO]->(to)
            SET rel.from_resource = $from_resource,
                rel.to_resource = $to_resource,
                rel.updated_at = timestamp()
            "#
            .to_string(),
        )
        .param("from_interface", connection.from_interface.to_string())
        .param("to_interface", connection.to_interface.to_string())
        .param("from_resource", connection.from_resource.to_string())
        .param("to_resource", connection.to_resource.to_string());

        graph.run(query).await?;
        debug!("Connection projected successfully");

        Ok(())
    }

    /// Get the underlying Neo4j graph connection
    pub fn graph(&self) -> &Arc<Graph> {
        &self.graph
    }
}
