// Copyright (c) 2025 - Cowboy AI, Inc.
//! NetBox Integration Test Example
//!
//! This example demonstrates the full event sourcing workflow:
//! 1. Publish infrastructure events to NATS JetStream
//! 2. NetBox projector consumes events and projects to NetBox
//! 3. Verify data appears in NetBox via API
//!
//! Run with: cargo run --example netbox_integration_test --features netbox
//!
//! Prerequisites:
//! 1. NATS server running: docker run -p 4222:4222 nats:latest -js
//! 2. NetBox running at http://10.0.224.131
//! 3. Source secrets: source ~/.secrets/cim-env.sh
//! 4. NetBox projector running: cargo run --bin netbox-projector --features netbox

use anyhow::{Context, Result};
use async_nats::jetstream;
use cim_infrastructure::adapters::InfrastructureEvent;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 NetBox Integration Test");
    println!("==========================\n");

    // Configuration
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "localhost:4222".to_string());
    let stream_name = "INFRASTRUCTURE";

    // Connect to NATS
    println!("🔌 Connecting to NATS at {}", nats_url);
    let client = async_nats::connect(&nats_url)
        .await
        .context("Failed to connect to NATS. Is NATS running?")?;
    println!("✅ Connected to NATS\n");

    // Get JetStream context
    let jetstream = jetstream::new(client);

    // Ensure stream exists
    println!("📡 Checking JetStream stream: {}", stream_name);
    match jetstream.get_stream(stream_name).await {
        Ok(_) => println!("✅ Stream exists\n"),
        Err(_) => {
            println!("📝 Creating stream...");
            jetstream
                .create_stream(jetstream::stream::Config {
                    name: stream_name.to_string(),
                    subjects: vec!["infrastructure.>".to_string()],
                    max_age: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
                    ..Default::default()
                })
                .await
                .context("Failed to create stream")?;
            println!("✅ Stream created\n");
        }
    }

    // Test 1: Publish ComputeRegistered event
    println!("📤 Test 1: Publishing ComputeRegistered event");
    let compute_event = InfrastructureEvent {
        event_id: Uuid::now_v7(),
        aggregate_id: Uuid::now_v7(),
        event_type: "ComputeRegistered".to_string(),
        data: json!({
            "id": "test-server-1",
            "hostname": "test-web01.example.com",
            "resource_type": "physical_server",
            "manufacturer": "Dell",
            "model": "PowerEdge R750"
        }),
    };

    let payload = serde_json::to_vec(&compute_event)?;
    jetstream
        .publish("infrastructure.compute.registered", payload.into())
        .await
        .context("Failed to publish event")?
        .await
        .context("Failed to await ack")?;

    println!("✅ Published ComputeRegistered event: {}", compute_event.event_id);
    println!("   Hostname: test-web01.example.com");
    println!("   Type: physical_server\n");

    // Test 2: Publish NetworkDefined event
    println!("📤 Test 2: Publishing NetworkDefined event");
    let network_event = InfrastructureEvent {
        event_id: Uuid::now_v7(),
        aggregate_id: Uuid::now_v7(),
        event_type: "NetworkDefined".to_string(),
        data: json!({
            "id": "test-network-1",
            "name": "Test DMZ Network",
            "cidr": "192.168.100.0/24",
            "vlan_id": 100
        }),
    };

    let payload = serde_json::to_vec(&network_event)?;
    jetstream
        .publish("infrastructure.network.defined", payload.into())
        .await
        .context("Failed to publish event")?
        .await
        .context("Failed to await ack")?;

    println!("✅ Published NetworkDefined event: {}", network_event.event_id);
    println!("   CIDR: 192.168.100.0/24");
    println!("   VLAN: 100\n");

    // Test 3: Publish InterfaceAdded event
    println!("📤 Test 3: Publishing InterfaceAdded event");
    let interface_event = InfrastructureEvent {
        event_id: Uuid::now_v7(),
        aggregate_id: Uuid::now_v7(),
        event_type: "InterfaceAdded".to_string(),
        data: json!({
            "device": "test-web01.example.com",
            "name": "eth0",
            "type": "1000base-t",
            "mac_address": "00:11:22:33:44:55",
            "mtu": 1500,
            "description": "Primary network interface"
        }),
    };

    let payload = serde_json::to_vec(&interface_event)?;
    jetstream
        .publish("infrastructure.interface.added", payload.into())
        .await
        .context("Failed to publish event")?
        .await
        .context("Failed to await ack")?;

    println!("✅ Published InterfaceAdded event: {}", interface_event.event_id);
    println!("   Device: test-web01.example.com");
    println!("   Interface: eth0\n");

    // Test 4: Publish IPAssigned event
    println!("📤 Test 4: Publishing IPAssigned event");
    let ip_event = InfrastructureEvent {
        event_id: Uuid::now_v7(),
        aggregate_id: Uuid::now_v7(),
        event_type: "IPAssigned".to_string(),
        data: json!({
            "address": "192.168.100.10/24",
            "device": "test-web01.example.com",
            "interface": "eth0",
            "status": "active"
        }),
    };

    let payload = serde_json::to_vec(&ip_event)?;
    jetstream
        .publish("infrastructure.ip.assigned", payload.into())
        .await
        .context("Failed to publish event")?
        .await
        .context("Failed to await ack")?;

    println!("✅ Published IPAssigned event: {}", ip_event.event_id);
    println!("   IP: 192.168.100.10/24");
    println!("   Assigned to: test-web01.example.com (eth0)\n");

    // Wait for projection to complete
    println!("⏳ Waiting 5 seconds for projector to process events...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify in NetBox
    println!("\n📊 Verification");
    println!("==============");
    println!("Check NetBox UI at: http://10.0.224.131");
    println!("  - Device: test-web01.example.com");
    println!("  - Network: 192.168.100.0/24");
    println!("  - Interface: eth0 on test-web01.example.com");
    println!("  - IP Address: 192.168.100.10/24\n");

    // Optional: Query NetBox API to verify
    if let Ok(token) = std::env::var("NETBOX_API_TOKEN") {
        println!("🔍 Querying NetBox API...");
        let client = reqwest::Client::new();

        // Check device
        let url = "http://10.0.224.131/api/dcim/devices/?name=test-web01.example.com";
        let response = client
            .get(url)
            .header("Authorization", format!("Token {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(count) = data["count"].as_i64() {
                if count > 0 {
                    println!("✅ Device found in NetBox");
                } else {
                    println!("⚠️ Device not found in NetBox (projector may not be running)");
                }
            }
        }
    }

    println!("\n🎉 Integration test complete!");
    println!("\nNote: Make sure the NetBox projector is running:");
    println!("  cargo run --bin netbox-projector --features netbox");

    Ok(())
}
