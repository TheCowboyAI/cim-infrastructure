// Copyright (c) 2025 - Cowboy AI, Inc.
//! Event Store Integration Tests

use cim_infrastructure::event_store::{EventStore, nats::NatsEventStore};
use cim_infrastructure::events::infrastructure::InfrastructureEvent;
use cim_infrastructure::events::compute_resource::{ComputeResourceEvent, ResourceRegistered};
use cim_infrastructure::domain::hostname::Hostname;
use cim_infrastructure::domain::resource_type::ResourceType;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_event_store_connect() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing NatsEventStore::connect...");

    let store = NatsEventStore::connect("nats://10.0.20.3:4222").await?;
    println!("✅ NatsEventStore connected successfully");

    Ok(())
}

#[tokio::test]
async fn test_event_store_append_and_read() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing event store append and read...");

    let store = NatsEventStore::connect("nats://10.0.20.3:4222").await?;
    let aggregate_id = Uuid::now_v7();

    // Create test event
    let event = InfrastructureEvent::ComputeResource(
        ComputeResourceEvent::ResourceRegistered(ResourceRegistered {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id,
            timestamp: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            hostname: Hostname::new("test-server01")?,
            resource_type: ResourceType::PhysicalServer,
        }),
    );

    println!("Appending event for aggregate: {}", aggregate_id);

    // Append event
    let version = store.append(aggregate_id, vec![event], None).await?;
    println!("✅ Event appended, version: {}", version);

    // Read events back
    println!("Reading events back...");
    let events = store.read_events(aggregate_id).await?;
    println!("✅ Read {} event(s)", events.len());

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sequence, 1);

    Ok(())
}

#[tokio::test]
async fn test_event_store_multiple_events() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing multiple events...");

    let store = NatsEventStore::connect("nats://10.0.20.3:4222").await?;
    let aggregate_id = Uuid::now_v7();

    // Append multiple events
    let events: Vec<InfrastructureEvent> = (1..=3)
        .map(|i| {
            InfrastructureEvent::ComputeResource(
                ComputeResourceEvent::ResourceRegistered(ResourceRegistered {
                    event_version: 1,
                    event_id: Uuid::now_v7(),
                    aggregate_id,
                    timestamp: Utc::now(),
                    correlation_id: Uuid::now_v7(),
                    causation_id: None,
                    hostname: Hostname::new(&format!("test-server{:02}", i)).unwrap(),
                    resource_type: ResourceType::PhysicalServer,
                }),
            )
        })
        .collect();

    println!("Appending {} events...", events.len());
    let version = store.append(aggregate_id, events, None).await?;
    println!("✅ Events appended, final version: {}", version);

    // Read events back
    let read_events = store.read_events(aggregate_id).await?;
    println!("✅ Read {} event(s)", read_events.len());

    assert_eq!(read_events.len(), 3);
    assert_eq!(read_events[0].sequence, 1);
    assert_eq!(read_events[1].sequence, 2);
    assert_eq!(read_events[2].sequence, 3);

    Ok(())
}
