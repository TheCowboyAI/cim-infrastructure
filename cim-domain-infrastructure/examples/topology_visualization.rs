//! Infrastructure Topology Visualization Example
//!
//! This example demonstrates:
//! - Building an infrastructure topology
//! - Using cim-graph's Kan extension for graph representations
//! - Creating Mermaid diagrams through category theory functors
//! - Producing topology reports
//!
//! Run with:
//! ```bash
//! cargo run --example topology_visualization
//! ```

use cim_domain_infrastructure::{
    InfrastructureFunctor,
    ComputeResourceSpec, ComputeType, ConnectionSpec, Hostname, InfrastructureAggregate,
    InfrastructureId, InterfaceId, InterfaceSpec, Ipv4Network, MessageIdentity, NetworkId,
    NetworkSpec, NetworkTopologySpec, ResourceCapabilities, ResourceId, SystemArchitecture,
    TopologyId,
};
use std::net::Ipv4Addr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("===========================================");
    println!("  Infrastructure Topology Visualization");
    println!("  (Using cim-graph Integration)");
    println!("===========================================\n");

    // Create infrastructure aggregate
    let mut infrastructure = InfrastructureAggregate::new(InfrastructureId::new());
    let identity = MessageIdentity::new_root();

    // Step 1: Register compute resources
    println!("📦 Registering compute resources...\n");

    // Web server
    let web_server_id = ResourceId::new("web-server-01")?;
    let web_server = ComputeResourceSpec {
        id: web_server_id.clone(),
        resource_type: ComputeType::VirtualMachine,
        hostname: Hostname::new("web-server-01")?,
        system: SystemArchitecture::x86_64_linux(),
        capabilities: ResourceCapabilities::default(),
    };
    infrastructure.handle_register_compute_resource(web_server.clone(), &identity)?;
    println!("  ✓ Registered: web-server-01 (VM)");

    // Database server
    let db_server_id = ResourceId::new("db-server-01")?;
    let db_server = ComputeResourceSpec {
        id: db_server_id.clone(),
        resource_type: ComputeType::Physical,
        hostname: Hostname::new("db-server-01")?,
        system: SystemArchitecture::x86_64_linux(),
        capabilities: ResourceCapabilities::default(),
    };
    infrastructure.handle_register_compute_resource(db_server.clone(), &identity)?;
    println!("  ✓ Registered: db-server-01 (Physical)");

    // Application server
    let app_server_id = ResourceId::new("app-container-01")?;
    let app_server = ComputeResourceSpec {
        id: app_server_id.clone(),
        resource_type: ComputeType::Container,
        hostname: Hostname::new("app-container-01")?,
        system: SystemArchitecture::aarch64_linux(),
        capabilities: ResourceCapabilities::default(),
    };
    infrastructure.handle_register_compute_resource(app_server.clone(), &identity)?;
    println!("  ✓ Registered: app-container-01 (Container)\n");

    // Step 2: Define networks
    println!("🌐 Defining networks...\n");

    let frontend_network = NetworkId::new("frontend-net")?;
    let backend_network = NetworkId::new("backend-net")?;

    let frontend_spec = NetworkSpec {
        id: frontend_network.clone(),
        name: "Frontend Network".to_string(),
        cidr_v4: Some(Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24)?),
        cidr_v6: None,
    };

    let backend_spec = NetworkSpec {
        id: backend_network.clone(),
        name: "Backend Network".to_string(),
        cidr_v4: Some(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 24)?),
        cidr_v6: None,
    };

    infrastructure.handle_define_network(frontend_spec, &identity)?;
    println!("  ✓ Defined: Frontend Network (192.168.1.0/24)");

    infrastructure.handle_define_network(backend_spec, &identity)?;
    println!("  ✓ Defined: Backend Network (10.0.0.0/24)\n");

    // Step 3: Add network interfaces
    println!("🔌 Adding network interfaces...\n");

    // Web server interface
    let web_interface_id = InterfaceId::new("web-eth0")?;
    let web_interface = InterfaceSpec {
        id: web_interface_id.clone(),
        resource_id: web_server.id.clone(),
        network_id: Some(frontend_network.clone()),
        addresses: vec!["192.168.1.10".parse()?],
    };
    infrastructure.handle_add_interface(web_interface.clone(), &identity)?;
    println!("  ✓ Added: web-eth0 on web-server-01");

    // Database server interface
    let db_interface_id = InterfaceId::new("db-eth0")?;
    let db_interface = InterfaceSpec {
        id: db_interface_id.clone(),
        resource_id: db_server.id.clone(),
        network_id: Some(backend_network.clone()),
        addresses: vec!["10.0.0.10".parse()?],
    };
    infrastructure.handle_add_interface(db_interface.clone(), &identity)?;
    println!("  ✓ Added: db-eth0 on db-server-01");

    // App server interface
    let app_interface_id = InterfaceId::new("app-eth0")?;
    let app_interface = InterfaceSpec {
        id: app_interface_id.clone(),
        resource_id: app_server.id.clone(),
        network_id: Some(backend_network.clone()),
        addresses: vec!["10.0.0.20".parse()?],
    };
    infrastructure.handle_add_interface(app_interface.clone(), &identity)?;
    println!("  ✓ Added: app-eth0 on app-container-01\n");

    // Step 4: Define topology with connections
    println!("🔗 Establishing connections...\n");

    let topology_spec = NetworkTopologySpec {
        topology_id: TopologyId::new(),
        networks: vec![],
        connections: vec![
            // Web server to App server
            ConnectionSpec {
                from_resource: web_server.id.clone(),
                from_interface: web_interface_id.clone(),
                to_resource: app_server.id.clone(),
                to_interface: app_interface_id.clone(),
            },
            // App server to Database
            ConnectionSpec {
                from_resource: app_server.id.clone(),
                from_interface: app_interface_id.clone(),
                to_resource: db_server.id.clone(),
                to_interface: db_interface_id.clone(),
            },
        ],
    };

    infrastructure.handle_define_network_topology(topology_spec, &identity)?;
    println!("  ✓ Connected: web-server-01 → app-container-01");
    println!("  ✓ Connected: app-container-01 → db-server-01\n");

    // Step 5: Map infrastructure to cim-graph domain objects using functor
    println!("📊 Mapping infrastructure to domain objects via functor...\n");

    let mut functor = InfrastructureFunctor::new("infrastructure_topology".to_string());
    functor.map_infrastructure(&infrastructure);

    println!("  Domain Object Statistics:");
    println!("  - Domain Objects: {}", functor.domain_objects().count());
    println!("  - Relationships: {}\n", functor.relationships().count());

    // Step 6: Generate Mermaid diagram using cim-graph rendering
    println!("🎨 Mermaid Diagram (via cim-graph):");
    println!("{}", "=".repeat(60));
    println!("{}", functor.to_mermaid());
    println!("{}\n", "=".repeat(60));

    // Step 7: Generate topology report
    println!("📋 Topology Report:");
    println!("{}", "=".repeat(60));
    println!("{}", functor.topology_report());
    println!("{}", "=".repeat(60));

    // Step 8: Serialize to IPLD-compatible JSON
    println!("\n💾 IPLD-compatible JSON representation:");
    println!("{}", "=".repeat(60));
    let json = functor.to_ipld_json()?;
    println!("{}", json);
    println!("{}\n", "=".repeat(60));

    // Step 9: Demonstrate Kan extension (optional)
    println!("🔬 Category Theory: Building Kan Extension...\n");

    if let Err(e) = functor.build_kan_extension("infra_kan_ext".to_string()) {
        println!("  ⚠ Kan extension build note: {}", e);
    } else {
        println!("  ✓ Kan extension constructed successfully");

        if let Err(e) = functor.extend_to_concepts() {
            println!("  ⚠ Concept extension note: {}", e);
        } else {
            println!("  ✓ Infrastructure extended to concept space via Lan_F(G)");

            if let Some(extension) = functor.kan_extension() {
                println!("  ✓ Universal property satisfied");
                println!("  - Extension ID: {}", extension.extension_id);
                println!("  - Extended mappings: {}", extension.extended_mappings.len());
            }
        }
    }

    println!("\n✅ Topology visualization complete!");
    println!("\n🎯 Key Architectural Points:");
    println!("  1. Infrastructure aggregates mapped to cim-graph domain objects");
    println!("  2. Category theory functors preserve composition");
    println!("  3. Kan extension provides universal construction for graph→domain");
    println!("  4. No custom graph implementation - pure composition!");
    println!("\nTo visualize the Mermaid diagram:");
    println!("  1. Copy the Mermaid code above");
    println!("  2. Visit https://mermaid.live");
    println!("  3. Paste and view your topology!");

    Ok(())
}
