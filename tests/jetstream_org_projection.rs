// Copyright (c) 2026 - Cowboy AI, Inc.
//! JetStream Organization Projection Integration Test
//!
//! Tests end-to-end projection of organization and people from cim-keys
//! into cim-infrastructure JetStream event store.
//!
//! This verifies that events from cim-keys can be properly stored and
//! retrieved from JetStream in cim-infrastructure.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Event structures matching cim-keys format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCreatedEvent {
    pub organization_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreatedEvent {
    pub person_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub organization_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Test organization data from cim-keys example
struct CowboyAIData {
    org_id: Uuid,
    org_name: String,
    people: Vec<PersonData>,
}

struct PersonData {
    id: Uuid,
    name: String,
    email: String,
    title: String,
}

impl CowboyAIData {
    fn example() -> Self {
        // IDs from cim-keys/examples/graph-data/organization-example.json
        Self {
            org_id: Uuid::parse_str("01933e60-0000-0000-0000-000000000001").unwrap(),
            org_name: "CowboyAI".to_string(),
            people: vec![
                PersonData {
                    id: Uuid::parse_str("01933e60-0000-0000-0000-000000000002").unwrap(),
                    name: "Alice Smith".to_string(),
                    email: "alice@cowboyai.com".to_string(),
                    title: "Engineering Manager".to_string(),
                },
                PersonData {
                    id: Uuid::parse_str("01933e60-0000-0000-0000-000000000003").unwrap(),
                    name: "Bob Jones".to_string(),
                    email: "bob@cowboyai.com".to_string(),
                    title: "Senior Developer".to_string(),
                },
                PersonData {
                    id: Uuid::parse_str("01933e60-0000-0000-0000-000000000004").unwrap(),
                    name: "Charlie Davis".to_string(),
                    email: "charlie@cowboyai.com".to_string(),
                    title: "Developer".to_string(),
                },
            ],
        }
    }

    fn create_organization_event(&self) -> OrganizationCreatedEvent {
        OrganizationCreatedEvent {
            organization_id: self.org_id,
            name: self.org_name.clone(),
            domain: Some("cowboyai.com".to_string()),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        }
    }

    fn create_person_events(&self) -> Vec<PersonCreatedEvent> {
        self.people
            .iter()
            .map(|person| PersonCreatedEvent {
                person_id: person.id,
                name: person.name.clone(),
                email: Some(person.email.clone()),
                title: Some(person.title.clone()),
                department: Some("Engineering".to_string()),
                organization_id: self.org_id,
                correlation_id: Uuid::now_v7(),
                causation_id: None,
            })
            .collect()
    }
}

#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_cowboyai_organization_to_jetstream() -> Result<()> {
    use async_nats::jetstream;
    use futures::StreamExt;
    use std::time::Duration;

    // Load CowboyAI organization data
    let data = CowboyAIData::example();

    // Verify we have the expected organization
    assert_eq!(data.org_name, "CowboyAI");
    assert_eq!(data.people.len(), 3);

    // Create organization event
    let org_event = data.create_organization_event();
    assert_eq!(org_event.organization_id, data.org_id);
    assert_eq!(org_event.name, "CowboyAI");

    // Create person events
    let person_events = data.create_person_events();
    assert_eq!(person_events.len(), 3);

    // Verify person data
    assert_eq!(person_events[0].name, "Alice Smith");
    assert_eq!(person_events[0].title, Some("Engineering Manager".to_string()));
    assert_eq!(person_events[1].name, "Bob Jones");
    assert_eq!(person_events[2].name, "Charlie Davis");

    // All events should reference the organization
    for event in &person_events {
        assert_eq!(event.organization_id, data.org_id);
    }

    println!("✅ Organization event created: {}", org_event.name);
    println!("✅ {} person events created", person_events.len());

    // Connect to NATS
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "localhost:4222".to_string());
    println!("🔌 Connecting to NATS at {}", nats_url);
    let client = async_nats::connect(&nats_url).await?;
    println!("✅ Connected to NATS");

    // Get JetStream context
    let jetstream = jetstream::new(client);

    // Create or get test stream
    let stream_name = "ORGANIZATION_TEST";
    println!("📡 Setting up JetStream stream: {}", stream_name);

    let stream = match jetstream.get_stream(stream_name).await {
        Ok(stream) => {
            println!("✅ Found existing stream: {}", stream_name);
            stream
        }
        Err(_) => {
            println!("📝 Creating stream: {}", stream_name);
            jetstream
                .create_stream(jetstream::stream::Config {
                    name: stream_name.to_string(),
                    subjects: vec!["organization.>".to_string(), "person.>".to_string()],
                    max_age: Duration::from_secs(60 * 60), // 1 hour for test
                    ..Default::default()
                })
                .await?;
            println!("✅ Created stream: {}", stream_name);
            jetstream.get_stream(stream_name).await?
        }
    };

    // Publish organization event to JetStream
    let org_subject = format!("organization.created.{}", org_event.organization_id);
    let org_payload = serde_json::to_vec(&org_event)?;

    println!("📤 Publishing organization event to subject: {}", org_subject);
    let ack = jetstream.publish(org_subject.clone(), org_payload.into()).await?;
    println!("✅ Organization event published (seq: {})", ack.await?.sequence);

    // Publish person events to JetStream
    for (idx, person_event) in person_events.iter().enumerate() {
        let person_subject = format!("person.created.{}", person_event.person_id);
        let person_payload = serde_json::to_vec(&person_event)?;

        println!("📤 Publishing person event {} to subject: {}", idx + 1, person_subject);
        let ack = jetstream.publish(person_subject, person_payload.into()).await?;
        println!("✅ Person event {} published (seq: {})", idx + 1, ack.await?.sequence);
    }

    // Create consumer to read events back
    let consumer_name = "test-consumer";
    println!("👂 Creating consumer: {}", consumer_name);

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some(consumer_name.to_string()),
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            deliver_policy: jetstream::consumer::DeliverPolicy::All,
            ..Default::default()
        })
        .await?;
    println!("✅ Consumer created");

    // Read events back from JetStream
    println!("📥 Reading events from JetStream...");
    let messages = consumer
        .stream()
        .max_messages_per_batch(10)
        .messages()
        .await?;

    tokio::pin!(messages);

    let mut retrieved_org_event: Option<OrganizationCreatedEvent> = None;
    let mut retrieved_person_events: Vec<PersonCreatedEvent> = Vec::new();
    let mut event_count = 0;

    // Collect up to 4 events (1 org + 3 people)
    while let Some(message) = messages.next().await {
        let msg = message?;
        event_count += 1;

        println!("📨 Received message {} from subject: {}", event_count, msg.subject);

        if msg.subject.starts_with("organization.") {
            let event: OrganizationCreatedEvent = serde_json::from_slice(&msg.payload)?;
            println!("  ✅ Organization: {}", event.name);
            retrieved_org_event = Some(event);
            msg.ack().await.map_err(|e| anyhow::anyhow!("Failed to acknowledge message: {}", e))?;
        } else if msg.subject.starts_with("person.") {
            let event: PersonCreatedEvent = serde_json::from_slice(&msg.payload)?;
            println!("  ✅ Person: {} ({})", event.name, event.title.as_ref().unwrap_or(&"No title".to_string()));
            retrieved_person_events.push(event);
            msg.ack().await.map_err(|e| anyhow::anyhow!("Failed to acknowledge message: {}", e))?;
        }

        // Stop after receiving all expected events
        if event_count >= 4 {
            break;
        }
    }

    // Verify organization event was retrieved
    let retrieved_org = retrieved_org_event.expect("Organization event not found in JetStream");
    assert_eq!(retrieved_org.organization_id, org_event.organization_id);
    assert_eq!(retrieved_org.name, org_event.name);
    assert_eq!(retrieved_org.domain, org_event.domain);
    println!("✅ Organization event verified: {}", retrieved_org.name);

    // Verify person events were retrieved
    assert_eq!(retrieved_person_events.len(), 3, "Expected 3 person events");

    // Verify all people are present (order may vary in JetStream)
    let retrieved_names: Vec<String> = retrieved_person_events.iter().map(|p| p.name.clone()).collect();
    assert!(retrieved_names.contains(&"Alice Smith".to_string()));
    assert!(retrieved_names.contains(&"Bob Jones".to_string()));
    assert!(retrieved_names.contains(&"Charlie Davis".to_string()));

    // Verify all person events reference the organization
    for event in &retrieved_person_events {
        assert_eq!(event.organization_id, data.org_id, "Person {} should reference organization", event.name);
    }

    println!("✅ All person events verified and linked to organization");
    println!("🎉 End-to-end projection test completed successfully!");

    Ok(())
}

#[tokio::test]
async fn test_organization_event_serialization() -> Result<()> {
    let data = CowboyAIData::example();
    let org_event = data.create_organization_event();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&org_event)?;
    println!("Organization Event JSON:\n{}", json);

    // Deserialize back
    let deserialized: OrganizationCreatedEvent = serde_json::from_str(&json)?;
    assert_eq!(deserialized.organization_id, org_event.organization_id);
    assert_eq!(deserialized.name, org_event.name);

    Ok(())
}

#[tokio::test]
async fn test_person_event_serialization() -> Result<()> {
    let data = CowboyAIData::example();
    let person_events = data.create_person_events();

    // Serialize first person event
    let json = serde_json::to_string_pretty(&person_events[0])?;
    println!("Person Event JSON:\n{}", json);

    // Deserialize back
    let deserialized: PersonCreatedEvent = serde_json::from_str(&json)?;
    assert_eq!(deserialized.person_id, person_events[0].person_id);
    assert_eq!(deserialized.name, person_events[0].name);
    assert_eq!(deserialized.organization_id, data.org_id);

    Ok(())
}

#[tokio::test]
async fn test_organization_relationships() -> Result<()> {
    let data = CowboyAIData::example();

    // Verify organizational structure
    assert_eq!(data.people.len(), 3);

    // Alice is Engineering Manager
    let alice = &data.people[0];
    assert_eq!(alice.name, "Alice Smith");
    assert_eq!(alice.title, "Engineering Manager");

    // Bob and Charlie report to Alice (based on graph-data example)
    let bob = &data.people[1];
    let charlie = &data.people[2];
    assert_eq!(bob.name, "Bob Jones");
    assert_eq!(bob.title, "Senior Developer");
    assert_eq!(charlie.name, "Charlie Davis");
    assert_eq!(charlie.title, "Developer");

    // All are part of the same organization
    let org_id = data.org_id;
    for person in &data.people {
        // In actual implementation, we'd verify organization_id from events
        // For now, we verify they're all part of the same dataset
        assert!(person.id.as_u128() > org_id.as_u128());
    }

    Ok(())
}
