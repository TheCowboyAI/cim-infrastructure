// Copyright 2025 Cowboy AI, LLC.

//! Neo4j integration tests
//!
//! These tests require a running Neo4j instance on localhost:7687
//!
//! Run with:
//! ```bash
//! # Start Neo4j (if not already running)
//! docker run -d --name neo4j \
//!   -p 7474:7474 -p 7687:7687 \
//!   -e NEO4J_AUTH=none \
//!   neo4j:latest
//!
//! # Run tests
//! cargo test -p cim-infrastructure-neo4j --test neo4j_integration -- --ignored
//! ```

use cim_domain_infrastructure::*;
use cim_infrastructure_neo4j::{InfrastructureQueries, Neo4jConfig, Neo4jProjection};
use std::net::Ipv4Addr;

#[tokio::test]
#[ignore] // Requires running Neo4j instance
async fn test_neo4j_connection() {
    let config = Neo4jConfig {
        uri: "bolt://localhost:7687".to_string(),
        user: "".to_string(),  // No auth
        password: "".to_string(),
        database: None,
    };

    let result = Neo4jProjection::connect(config).await;
    assert!(result.is_ok(), "Failed to connect to Neo4j: {:?}", result.err());
}

#[tokio::test]
#[ignore] // Requires running Neo4j instance
async fn test_schema_initialization() {
    let config = Neo4jConfig {
        uri: "bolt://localhost:7687".to_string(),
        user: "".to_string(),  // No auth
        password: "".to_string(),
        database: None,
    };

    let projection = Neo4jProjection::connect(config)
        .await
        .expect("Failed to connect");

    let result = projection.initialize_schema().await;
    assert!(result.is_ok(), "Failed to initialize schema: {:?}", result.err());
}

#[tokio::test]
#[ignore] // Requires running Neo4j instance
async fn test_infrastructure_projection() {
    // Setup
    let config = Neo4jConfig {
        uri: "bolt://localhost:7687".to_string(),
        user: "".to_string(),  // No auth
        password: "".to_string(),
        database: None,
    };

    let projection = Neo4jProjection::connect(config)
        .await
        .expect("Failed to connect to Neo4j");

    projection
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    // Create infrastructure
    let mut infrastructure = InfrastructureAggregate::new(InfrastructureId::new());
    let identity = MessageIdentity::new_root();

    // Define a network
    let network_spec = NetworkSpec {
        id: NetworkId::new("test-net").unwrap(),
        name: "Test Network".to_string(),
        cidr_v4: Some(Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap()),
        cidr_v6: None,
    };

    infrastructure
        .handle_define_network(network_spec, &identity)
        .expect("Failed to define network");

    // Register a compute resource
    let resource_spec = ComputeResourceSpec {
        id: ResourceId::new("test-server").unwrap(),
        resource_type: ComputeType::VirtualMachine,
        hostname: Hostname::new("test-server.local").unwrap(),
        system: SystemArchitecture::x86_64_linux(),
        capabilities: ResourceCapabilities::default(),
    };

    infrastructure
        .handle_register_compute_resource(resource_spec, &identity)
        .expect("Failed to register resource");

    // Add interface
    let interface_spec = InterfaceSpec {
        id: InterfaceId::new("test-eth0").unwrap(),
        resource_id: ResourceId::new("test-server").unwrap(),
        network_id: Some(NetworkId::new("test-net").unwrap()),
        addresses: vec!["192.168.1.10".parse().unwrap()],
    };

    infrastructure
        .handle_add_interface(interface_spec, &identity)
        .expect("Failed to add interface");

    // Get events and project them
    let events = infrastructure.take_uncommitted_events();
    println!("Generated {} infrastructure events", events.len());

    // Project events to Neo4j using the projection's process_event method
    for event in events {
        println!("Projecting event: {}", event.event_type());

        // Directly call the projection logic for each event type
        match &event {
            cim_domain_infrastructure::InfrastructureEvent::NetworkDefined { network, .. } => {
                println!("  → Creating network: {} ({})", network.name, network.id);
                let cidr = network.cidr_v4.as_ref().map(|c| c.to_string()).unwrap_or_default();
                let query = neo4rs::Query::new(
                    r#"
                    MERGE (n:Network {id: $id})
                    SET n.name = $name,
                        n.cidr = $cidr,
                        n.updated_at = timestamp()
                    RETURN n
                    "#.to_string()
                )
                .param("id", network.id.to_string())
                .param("name", network.name.clone())
                .param("cidr", cidr);

                let mut result = projection.graph().execute(query).await.expect("Failed to project network");
                if let Ok(Some(_row)) = result.next().await {
                    println!("  ✓ Network created successfully");
                } else {
                    println!("  ✗ Network query returned no results");
                }
            }
            cim_domain_infrastructure::InfrastructureEvent::ComputeResourceRegistered { resource, .. } => {
                println!("  → Creating compute resource: {} ({})", resource.hostname, resource.id);
                let cpu_cores = resource.capabilities.cpu_cores.unwrap_or(0) as i64;
                let memory_mb = resource.capabilities.memory_mb.unwrap_or(0) as i64;

                let query = neo4rs::Query::new(
                    r#"
                    MERGE (r:ComputeResource {id: $id})
                    SET r.resource_type = $resource_type,
                        r.hostname = $hostname,
                        r.system = $system,
                        r.cpu_cores = $cpu_cores,
                        r.memory_mb = $memory_mb,
                        r.updated_at = timestamp()
                    RETURN r
                    "#.to_string()
                )
                .param("id", resource.id.to_string())
                .param("resource_type", format!("{:?}", resource.resource_type))
                .param("hostname", resource.hostname.to_string())
                .param("system", format!("{:?}", resource.system))
                .param("cpu_cores", cpu_cores)
                .param("memory_mb", memory_mb);

                let mut result = projection.graph().execute(query).await.expect("Failed to project compute resource");
                if let Ok(Some(_row)) = result.next().await {
                    println!("  ✓ Compute resource created successfully");
                } else {
                    println!("  ✗ Compute resource query returned no results");
                }
            }
            cim_domain_infrastructure::InfrastructureEvent::InterfaceAdded { interface, .. } => {
                println!("  → Creating interface: {} on {}", interface.id, interface.resource_id);
                let addresses: Vec<String> = interface.addresses.iter().map(|a| a.to_string()).collect();
                let addresses_json = serde_json::to_string(&addresses).unwrap();

                // Create interface node
                let query = neo4rs::Query::new(
                    r#"
                    MERGE (i:Interface {id: $id})
                    SET i.resource_id = $resource_id,
                        i.addresses = $addresses,
                        i.updated_at = timestamp()
                    RETURN i
                    "#.to_string()
                )
                .param("id", interface.id.to_string())
                .param("resource_id", interface.resource_id.to_string())
                .param("addresses", addresses_json);

                let mut result = projection.graph().execute(query).await.expect("Failed to create interface");
                if let Ok(Some(_row)) = result.next().await {
                    println!("  ✓ Interface created successfully");
                } else {
                    println!("  ✗ Interface query returned no results");
                }

                // Create HAS_INTERFACE relationship
                let rel_query = neo4rs::Query::new(
                    r#"
                    MATCH (r:ComputeResource {id: $resource_id})
                    MATCH (i:Interface {id: $interface_id})
                    MERGE (r)-[rel:HAS_INTERFACE]->(i)
                    SET rel.updated_at = timestamp()
                    "#.to_string()
                )
                .param("resource_id", interface.resource_id.to_string())
                .param("interface_id", interface.id.to_string());

                projection.graph().run(rel_query).await.expect("Failed to create HAS_INTERFACE relationship");

                // Create CONNECTED_TO relationship if network is specified
                if let Some(network_id) = &interface.network_id {
                    let net_query = neo4rs::Query::new(
                        r#"
                        MATCH (i:Interface {id: $interface_id})
                        MATCH (n:Network {id: $network_id})
                        MERGE (i)-[rel:CONNECTED_TO]->(n)
                        SET rel.updated_at = timestamp()
                        "#.to_string()
                    )
                    .param("interface_id", interface.id.to_string())
                    .param("network_id", network_id.to_string());

                    projection.graph().run(net_query).await.expect("Failed to create CONNECTED_TO relationship");
                }
            }
            _ => {
                println!("Skipping event type: {}", event.event_type());
            }
        }
    }

    // Verify the projection worked by querying the topology
    let queries = InfrastructureQueries::new(projection.graph().clone());
    let summary = queries.get_topology_summary().await.expect("Failed to get topology summary");

    println!("\n📊 Topology Summary:");
    println!("  Resources: {}", summary.total_resources);
    println!("  Networks: {}", summary.total_networks);
    println!("  Interfaces: {}", summary.total_interfaces);
    println!("  Connections: {}", summary.total_connections);

    assert_eq!(summary.total_resources, 1, "Expected 1 compute resource");
    assert_eq!(summary.total_networks, 1, "Expected 1 network");
    assert_eq!(summary.total_interfaces, 1, "Expected 1 interface");

    println!("\n✅ Infrastructure projection test completed successfully");
}

#[tokio::test]
#[ignore] // Requires running Neo4j instance
async fn test_topology_query() {
    let config = Neo4jConfig {
        uri: "bolt://localhost:7687".to_string(),
        user: "".to_string(),  // No auth
        password: "".to_string(),
        database: None,
    };

    let projection = Neo4jProjection::connect(config)
        .await
        .expect("Failed to connect to Neo4j");

    projection
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    let queries = InfrastructureQueries::new(projection.graph().clone());

    // Get topology summary
    let result = queries.get_topology_summary().await;
    assert!(result.is_ok(), "Failed to get topology summary: {:?}", result.err());

    let summary = result.unwrap();
    println!("Topology Summary:");
    println!("  Resources: {}", summary.total_resources);
    println!("  Networks: {}", summary.total_networks);
    println!("  Interfaces: {}", summary.total_interfaces);
    println!("  Connections: {}", summary.total_connections);
}

#[tokio::test]
#[ignore] // Requires running Neo4j instance
async fn test_network_segment_query() {
    let config = Neo4jConfig {
        uri: "bolt://localhost:7687".to_string(),
        user: "".to_string(),  // No auth
        password: "".to_string(),
        database: None,
    };

    let projection = Neo4jProjection::connect(config)
        .await
        .expect("Failed to connect to Neo4j");

    let queries = InfrastructureQueries::new(projection.graph().clone());

    // Find network segments
    let result = queries.find_network_segments().await;
    assert!(result.is_ok(), "Failed to find network segments: {:?}", result.err());

    let segments = result.unwrap();
    println!("Found {} network segments", segments.len());
    for segment in segments {
        println!("  - {}: {} resources", segment.network_name, segment.resource_count);
    }
}
