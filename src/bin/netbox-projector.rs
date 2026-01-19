// Copyright (c) 2025 - Cowboy AI, Inc.
//! NetBox Projector Service
//!
//! Listens to NATS JetStream infrastructure events and projects them to NetBox.
//!
//! This service implements the CQRS read-side projection pattern:
//! - Events → JetStream → Consumer → NetBox Projection Adapter → NetBox API
//!
//! Run with: cargo run --bin netbox-projector --features netbox
//!
//! Prerequisites:
//! 1. NATS server running (default: localhost:4222)
//! 2. NetBox API accessible (via NETBOX_URL environment variable)
//! 3. NetBox API token set (via NETBOX_API_TOKEN environment variable)

use anyhow::{Context, Result};
use async_nats::jetstream;
use cim_infrastructure::{
    adapters::{InfrastructureEvent, NetBoxConfig, NetBoxProjectionAdapter},
    projection::ProjectionAdapter,
};
use futures::StreamExt;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Configuration for the NetBox projector service
#[derive(Debug, Clone)]
struct ProjectorConfig {
    /// NATS server URL
    nats_url: String,
    /// JetStream stream name for infrastructure events
    stream_name: String,
    /// Consumer name for this projector
    consumer_name: String,
    /// NetBox configuration
    netbox: NetBoxConfig,
}

impl ProjectorConfig {
    /// Load configuration from environment variables
    fn from_env() -> Result<Self> {
        let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "localhost:4222".to_string());

        let stream_name =
            std::env::var("NATS_STREAM").unwrap_or_else(|_| "INFRASTRUCTURE".to_string());

        let consumer_name = std::env::var("NATS_CONSUMER")
            .unwrap_or_else(|_| "netbox-projector".to_string());

        let netbox = NetBoxConfig {
            base_url: std::env::var("NETBOX_URL")
                .unwrap_or_else(|_| "http://10.0.224.131".to_string()),
            api_token: std::env::var("NETBOX_API_TOKEN")
                .context("NETBOX_API_TOKEN not set. Run: source ~/.secrets/cim-env.sh")?,
            default_site_id: std::env::var("NETBOX_DEFAULT_SITE")
                .ok()
                .and_then(|s| s.parse().ok()),
            timeout_secs: 30,
        };

        Ok(Self {
            nats_url,
            stream_name,
            consumer_name,
            netbox,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("🚀 Starting NetBox Projector Service");

    // Load configuration
    let config = ProjectorConfig::from_env()?;
    info!("📋 Configuration loaded:");
    info!("  - NATS URL: {}", config.nats_url);
    info!("  - Stream: {}", config.stream_name);
    info!("  - Consumer: {}", config.consumer_name);
    info!("  - NetBox URL: {}", config.netbox.base_url);

    // Connect to NATS
    info!("🔌 Connecting to NATS at {}", config.nats_url);
    let client = async_nats::connect(&config.nats_url)
        .await
        .context("Failed to connect to NATS")?;
    info!("✅ Connected to NATS");

    // Get JetStream context
    let jetstream = jetstream::new(client);

    // Create or get stream
    info!("📡 Setting up JetStream stream: {}", config.stream_name);
    let stream = match jetstream.get_stream(&config.stream_name).await {
        Ok(stream) => {
            info!("✅ Found existing stream: {}", config.stream_name);
            stream
        }
        Err(_) => {
            info!(
                "📝 Stream '{}' not found, creating...",
                config.stream_name
            );
            jetstream
                .create_stream(jetstream::stream::Config {
                    name: config.stream_name.clone(),
                    subjects: vec!["infrastructure.>".to_string()],
                    max_age: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
                    ..Default::default()
                })
                .await
                .context("Failed to create stream")?;
            info!("✅ Created stream: {}", config.stream_name);
            jetstream.get_stream(&config.stream_name).await?
        }
    };

    // Create or get consumer
    info!("👂 Setting up consumer: {}", config.consumer_name);
    let consumer = match stream.get_consumer(&config.consumer_name).await {
        Ok(consumer) => {
            info!("✅ Found existing consumer: {}", config.consumer_name);
            consumer
        }
        Err(_) => {
            info!(
                "📝 Consumer '{}' not found, creating...",
                config.consumer_name
            );
            stream
                .create_consumer(jetstream::consumer::pull::Config {
                    durable_name: Some(config.consumer_name.clone()),
                    ack_policy: jetstream::consumer::AckPolicy::Explicit,
                    ..Default::default()
                })
                .await
                .context("Failed to create consumer")?;
            info!("✅ Created consumer: {}", config.consumer_name);
            stream
                .get_consumer(&config.consumer_name)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get consumer: {}", e))?
        }
    };

    // Initialize NetBox projection adapter
    info!("🔧 Initializing NetBox projection adapter");
    let mut adapter = NetBoxProjectionAdapter::new(config.netbox.clone())
        .await
        .context("Failed to create NetBox adapter")?;

    adapter
        .initialize()
        .await
        .context("Failed to initialize NetBox adapter")?;
    info!("✅ NetBox projection adapter initialized");

    // Start consuming messages
    info!("🎧 Starting event consumption...");
    let messages = consumer
        .stream()
        .max_messages_per_batch(10)
        .messages()
        .await
        .context("Failed to start consuming messages")?;

    tokio::pin!(messages);

    let mut event_count = 0u64;
    let mut error_count = 0u64;

    while let Some(message) = messages.next().await {
        match message {
            Ok(msg) => {
                debug!(
                    "📨 Received message from subject: {}",
                    msg.subject
                );

                // Parse event
                match serde_json::from_slice::<InfrastructureEvent>(&msg.payload) {
                    Ok(event) => {
                        info!(
                            "🔄 Processing event: {} ({})",
                            event.event_type, event.event_id
                        );

                        // Project to NetBox
                        match adapter.project(event.clone()).await {
                            Ok(_) => {
                                event_count += 1;
                                info!(
                                    "✅ Successfully projected event {} (total: {})",
                                    event.event_id, event_count
                                );

                                // Acknowledge the message
                                if let Err(e) = msg.ack().await {
                                    error!("⚠️ Failed to acknowledge message: {}", e);
                                }
                            }
                            Err(e) => {
                                error_count += 1;
                                error!(
                                    "❌ Failed to project event {}: {} (total errors: {})",
                                    event.event_id, e, error_count
                                );

                                // Negative acknowledge - allows retry with backoff
                                if let Err(e) = msg.ack_with(jetstream::AckKind::Nak(None)).await
                                {
                                    error!("⚠️ Failed to NAK message: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        error!("❌ Failed to parse event: {} (total errors: {})", e, error_count);

                        // Terminate - bad message format, retry won't help
                        if let Err(e) = msg.ack_with(jetstream::AckKind::Term).await {
                            error!("⚠️ Failed to terminate message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                error!("❌ Error receiving message: {} (total errors: {})", e, error_count);

                // Brief backoff on errors
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        // Log statistics periodically
        if (event_count + error_count) % 100 == 0 {
            info!(
                "📊 Statistics: {} events processed, {} errors",
                event_count, error_count
            );
        }
    }

    warn!("⚠️ Message stream ended unexpectedly");
    Ok(())
}
