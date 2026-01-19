// Copyright (c) 2025 - Cowboy AI, Inc.
//! NetBox Device Types Example
//!
//! Demonstrates projecting various infrastructure device types to NetBox:
//! - Routers
//! - Switches
//! - Servers
//! - Appliances
//! - Others
//!
//! Run with: cargo run --example netbox_device_types --features netbox
//!
//! Prerequisites:
//! 1. NATS server running: docker run -p 4222:4222 nats:latest -js
//! 2. NetBox running at http://10.0.224.131
//! 3. Source secrets: source ~/.secrets/cim-env.sh

use anyhow::{Context, Result};
use async_nats::jetstream;
use cim_infrastructure::adapters::InfrastructureEvent;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 NetBox Device Types Example");
    println!("================================\n");

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
                    max_age: Duration::from_secs(7 * 24 * 60 * 60),
                    ..Default::default()
                })
                .await
                .context("Failed to create stream")?;
            println!("✅ Stream created\n");
        }
    }

    // Test 1: Core Router
    println!("📤 Example 1: Publishing Router event");
    publish_event(
        &jetstream,
        "router",
        "core-router-01.example.com",
        "Cisco",
        "ASR 1001-X",
        "Core network router for datacenter backbone"
    ).await?;

    // Test 2: Distribution Switch
    println!("📤 Example 2: Publishing Switch event");
    publish_event(
        &jetstream,
        "switch",
        "dist-switch-01.example.com",
        "Cisco",
        "Catalyst 3850",
        "Distribution layer switch"
    ).await?;

    // Test 3: Layer 3 Switch
    println!("📤 Example 3: Publishing Layer 3 Switch event");
    publish_event(
        &jetstream,
        "layer3_switch",
        "l3-switch-01.example.com",
        "Arista",
        "7050X3",
        "Layer 3 switch with routing capabilities"
    ).await?;

    // Test 4: Physical Server
    println!("📤 Example 4: Publishing Physical Server event");
    publish_event(
        &jetstream,
        "physical_server",
        "compute-01.example.com",
        "Dell",
        "PowerEdge R750",
        "Compute host for virtualization"
    ).await?;

    // Test 5: Firewall Appliance
    println!("📤 Example 5: Publishing Firewall event");
    publish_event(
        &jetstream,
        "firewall",
        "fw-01.example.com",
        "Palo Alto",
        "PA-5250",
        "Perimeter firewall"
    ).await?;

    // Test 6: Load Balancer
    println!("📤 Example 6: Publishing Load Balancer event");
    publish_event(
        &jetstream,
        "load_balancer",
        "lb-01.example.com",
        "F5",
        "BIG-IP 4000s",
        "Application load balancer"
    ).await?;

    // Test 7: Storage Array
    println!("📤 Example 7: Publishing Storage Array event");
    publish_event(
        &jetstream,
        "storage_array",
        "storage-01.example.com",
        "NetApp",
        "FAS8700",
        "Primary storage array"
    ).await?;

    // Test 8: Access Point
    println!("📤 Example 8: Publishing Access Point event");
    publish_event(
        &jetstream,
        "access_point",
        "ap-floor3-01.example.com",
        "Ubiquiti",
        "UniFi AP AC Pro",
        "WiFi access point - Floor 3"
    ).await?;

    // Test 9: PDU
    println!("📤 Example 9: Publishing PDU event");
    publish_event(
        &jetstream,
        "pdu",
        "pdu-rack01-a.example.com",
        "APC",
        "AP8941",
        "Power distribution unit - Rack 01"
    ).await?;

    // Test 10: UPS
    println!("📤 Example 10: Publishing UPS event");
    publish_event(
        &jetstream,
        "ups",
        "ups-row-a.example.com",
        "APC",
        "Symmetra PX 100kW",
        "UPS for row A"
    ).await?;

    // Wait for projection
    println!("\n⏳ Waiting 5 seconds for projector to process events...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verification
    println!("\n📊 Verification");
    println!("==============");
    println!("Check NetBox UI at: http://10.0.224.131/dcim/devices/");
    println!("\nYou should see devices organized by role with color coding:");
    println!("  🟢 Compute (Green): Physical Server");
    println!("  🔵 Network (Blue): Router, Switch, L3 Switch, Access Point, Load Balancer");
    println!("  🔴 Security (Red): Firewall");
    println!("  🟠 Storage (Orange): Storage Array");
    println!("  🟡 Power (Yellow): PDU, UPS");
    println!("\nNote: Make sure the NetBox projector is running:");
    println!("  cargo run --bin netbox-projector --features netbox\n");

    Ok(())
}

async fn publish_event(
    jetstream: &async_nats::jetstream::Context,
    resource_type: &str,
    hostname: &str,
    manufacturer: &str,
    model: &str,
    description: &str,
) -> Result<()> {
    let event = InfrastructureEvent {
        event_id: Uuid::now_v7(),
        aggregate_id: Uuid::now_v7(),
        event_type: "ComputeRegistered".to_string(),
        data: json!({
            "id": format!("{}-{}", resource_type, hostname),
            "hostname": hostname,
            "resource_type": resource_type,
            "manufacturer": manufacturer,
            "model": model,
            "description": description
        }),
    };

    let payload = serde_json::to_vec(&event)?;
    jetstream
        .publish("infrastructure.compute.registered", payload.into())
        .await
        .context("Failed to publish event")?
        .await
        .context("Failed to await ack")?;

    println!("✅ Published: {} - {} {} ({})",
             hostname, manufacturer, model, resource_type);

    Ok(())
}
