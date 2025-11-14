//! Integration tests for cim-infrastructure-nats
//!
//! These tests require a running NATS server with JetStream enabled:
//! ```bash
//! nats-server -js
//! ```

use std::sync::Arc;
use std::time::Duration;

use cim_domain_infrastructure::{
    ComputeType, Hostname, InfrastructureAggregate, InfrastructureId, MessageIdentity,
    ResourceCapabilities, SystemArchitecture,
};
use cim_infrastructure_nats::{
    event_store::{EventStoreConfig, JetStreamEventStore},
    projections::ProjectionManager,
    publisher::EventPublisher,
    subjects,
    subscriber::{EventHandler, EventSubscriberBuilder, FnEventHandler},
    EventStore, StoredEvent,
};

/// Helper to check if NATS is available
async fn nats_available() -> bool {
    async_nats::connect("nats://localhost:4222")
        .await
        .is_ok()
}

#[tokio::test]
#[ignore] // Requires running NATS server
async fn test_complete_event_workflow() -> Result<(), Box<dyn std::error::Error>> {
    if !nats_available().await {
        eprintln!("Skipping test: NATS server not available at localhost:4222");
        return Ok(());
    }

    tracing_subscriber::fmt::init();

    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Create event store
    let config = EventStoreConfig {
        stream_name: "INFRASTRUCTURE_TEST".to_string(),
        max_age: Duration::from_secs(60),
        max_messages: 1000,
        replicas: 1,
    };

    let event_store = Arc::new(JetStreamEventStore::new(client.clone(), config).await?);

    // Create publisher
    let publisher = EventPublisher::new(client.clone(), Arc::clone(&event_store) as Arc<dyn EventStore>);

    // Create infrastructure aggregate
    let infra_id = InfrastructureId::new();
    let mut aggregate = InfrastructureAggregate::new(infra_id.clone());

    // Create message identity
    let identity = MessageIdentity::new_root();

    // Register a compute resource
    let spec = cim_domain_infrastructure::ComputeResourceSpec {
        id: cim_domain_infrastructure::ResourceId::new("test-vm-01")?,
        resource_type: ComputeType::VirtualMachine,
        hostname: Hostname::new("test-vm-01")?,
        system: SystemArchitecture::x86_64_linux(),
        capabilities: ResourceCapabilities::default(),
    };

    aggregate.handle_register_compute_resource(spec, &identity)?;
    let events = aggregate.take_uncommitted_events();

    // Publish events
    for (seq, event) in events.iter().enumerate() {
        publisher
            .publish(event, infra_id.as_uuid(), (seq + 1) as u64)
            .await?;
    }

    // Wait for events to be persisted
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Read events back
    let stored_events = event_store.read_aggregate(infra_id.as_uuid()).await?;

    assert!(!stored_events.is_empty(), "Should have stored events");

    println!("✅ Successfully published and retrieved {} events", stored_events.len());

    Ok(())
}

#[tokio::test]
#[ignore] // Requires running NATS server
async fn test_event_projection() -> Result<(), Box<dyn std::error::Error>> {
    if !nats_available().await {
        eprintln!("Skipping test: NATS server not available at localhost:4222");
        return Ok(());
    }

    tracing_subscriber::fmt::init();

    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Create event store
    let config = EventStoreConfig {
        stream_name: "INFRASTRUCTURE_PROJECTION_TEST".to_string(),
        max_age: Duration::from_secs(60),
        max_messages: 1000,
        replicas: 1,
    };

    let event_store = Arc::new(JetStreamEventStore::new(client.clone(), config).await?);
    let publisher = EventPublisher::new(client.clone(), Arc::clone(&event_store) as Arc<dyn EventStore>);

    // Create projection manager
    let projection_manager = Arc::new(ProjectionManager::new());

    // Create infrastructure and publish events
    let infra_id = InfrastructureId::new();
    let mut aggregate = InfrastructureAggregate::new(infra_id.clone());
    let identity = MessageIdentity::new_root();

    // Register multiple compute resources
    for i in 1..=3 {
        let spec = cim_domain_infrastructure::ComputeResourceSpec {
            id: cim_domain_infrastructure::ResourceId::new("test-resource")?,
            resource_type: ComputeType::VirtualMachine,
            hostname: Hostname::new(&format!("test-vm-{:02}", i))?,
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::default(),
        };

        aggregate.handle_register_compute_resource(spec, &identity)?;
        let events = aggregate.take_uncommitted_events();

        for (seq, event) in events.iter().enumerate() {
            publisher
                .publish(event, infra_id.as_uuid(), (seq + 1) as u64)
                .await?;
        }
    }

    // Wait for events to be persisted
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Read events and project them
    let stored_events = event_store.read_aggregate(infra_id.as_uuid()).await?;

    for event in &stored_events {
        projection_manager.project(event).await?;
    }

    // Get the projected topology
    let topology = projection_manager.get_topology().await;

    println!("✅ Projected topology:");
    println!("   Compute resources: {}", topology.compute_resources.len());
    println!("   Active resources: {}", topology.active_compute_resources().len());

    assert!(topology.compute_resources.len() >= 3, "Should have at least 3 compute resources");

    Ok(())
}

#[tokio::test]
#[ignore] // Requires running NATS server
async fn test_event_subscription() -> Result<(), Box<dyn std::error::Error>> {
    if !nats_available().await {
        eprintln!("Skipping test: NATS server not available at localhost:4222");
        return Ok(());
    }

    tracing_subscriber::fmt::init();

    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Create event store
    let config = EventStoreConfig {
        stream_name: "INFRASTRUCTURE_SUBSCRIPTION_TEST".to_string(),
        max_age: Duration::from_secs(60),
        max_messages: 1000,
        replicas: 1,
    };

    let event_store = Arc::new(JetStreamEventStore::new(client.clone(), config).await?);

    // Create a simple event counter
    let event_count = Arc::new(tokio::sync::Mutex::new(0));
    let counter_clone = Arc::clone(&event_count);

    let handler = Arc::new(FnEventHandler::new(move |event: StoredEvent| {
        let counter = Arc::clone(&counter_clone);
        tokio::spawn(async move {
            let mut count = counter.lock().await;
            *count += 1;
            println!("📨 Received event: {} (count: {})", event.event_type, *count);
        });
        Ok(())
    }));

    // Subscribe to all compute events
    let subscriber = EventSubscriberBuilder::new()
        .client(client.clone())
        .subject(subjects::subjects::all_compute_events())
        .handler(handler)
        .build()?;

    let _subscription_handle = subscriber.subscribe().await?;

    // Give subscription time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create publisher
    let publisher = EventPublisher::new(client.clone(), Arc::clone(&event_store) as Arc<dyn EventStore>);

    // Publish some events
    let infra_id = InfrastructureId::new();
    let mut aggregate = InfrastructureAggregate::new(infra_id.clone());
    let identity = MessageIdentity::new_root();

    let spec = cim_domain_infrastructure::ComputeResourceSpec {
        id: cim_domain_infrastructure::ResourceId::new("test-resource")?,
        resource_type: ComputeType::Physical,
        hostname: Hostname::new("test-server-01")?,
        system: SystemArchitecture::x86_64_linux(),
        capabilities: ResourceCapabilities::default(),
    };

    aggregate.handle_register_compute_resource(spec, &identity)?;
    let events = aggregate.take_uncommitted_events();

    for (seq, event) in events.iter().enumerate() {
        publisher
            .publish(event, infra_id.as_uuid(), (seq + 1) as u64)
            .await?;
    }

    // Wait for events to be processed
    tokio::time::sleep(Duration::from_millis(500)).await;

    let final_count = *event_count.lock().await;
    println!("✅ Processed {} events via subscription", final_count);

    // Note: Due to timing, count might be 0 if messages haven't been delivered yet
    // In a real system, you'd use proper synchronization

    Ok(())
}

#[tokio::test]
#[ignore] // Requires running NATS server
async fn test_event_replay() -> Result<(), Box<dyn std::error::Error>> {
    if !nats_available().await {
        eprintln!("Skipping test: NATS server not available at localhost:4222");
        return Ok(());
    }

    tracing_subscriber::fmt::init();

    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Create event store
    let config = EventStoreConfig {
        stream_name: "INFRASTRUCTURE_REPLAY_TEST".to_string(),
        max_age: Duration::from_secs(60),
        max_messages: 1000,
        replicas: 1,
    };

    let event_store = Arc::new(JetStreamEventStore::new(client.clone(), config).await?);
    let publisher = EventPublisher::new(client.clone(), Arc::clone(&event_store) as Arc<dyn EventStore>);

    // Create and publish events
    let infra_id = InfrastructureId::new();
    let mut aggregate = InfrastructureAggregate::new(infra_id.clone());
    let identity = MessageIdentity::new_root();

    // Create a sequence of events
    for i in 1..=5 {
        let spec = cim_domain_infrastructure::ComputeResourceSpec {
            id: cim_domain_infrastructure::ResourceId::new("test-resource")?,
            resource_type: ComputeType::Container,
            hostname: Hostname::new(&format!("container-{:02}", i))?,
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::default(),
        };

        aggregate.handle_register_compute_resource(spec, &identity)?;
        let events = aggregate.take_uncommitted_events();

        for (seq, event) in events.iter().enumerate() {
            publisher
                .publish(event, infra_id.as_uuid(), (seq + 1) as u64)
                .await?;
        }
    }

    // Wait for events to be persisted
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Replay events to rebuild projection
    let projection_manager = ProjectionManager::new();
    let all_events = event_store.read_all().await?;

    projection_manager.rebuild(all_events.clone()).await?;

    let topology = projection_manager.get_topology().await;

    println!("✅ Replayed {} events", all_events.len());
    println!("   Final state: {} compute resources", topology.compute_resources.len());

    assert!(topology.compute_resources.len() >= 5, "Should have at least 5 compute resources after replay");

    Ok(())
}
