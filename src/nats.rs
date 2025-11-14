//! NATS client abstraction for messaging infrastructure

use async_nats::{Client, ConnectOptions, Subscriber};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::errors::{InfrastructureError, InfrastructureResult};

/// Configuration for NATS connection
#[derive(Debug, Clone)]
pub struct NatsConfig {
    /// NATS server URLs
    pub servers: Vec<String>,
    /// Client name
    pub name: String,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            servers: vec!["nats://localhost:4222".to_string()],
            name: "cim-client".to_string(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(5),
        }
    }
}

/// NATS client wrapper providing domain-specific operations
#[derive(Clone)]
pub struct NatsClient {
    client: Client,
}

impl NatsClient {
    /// Create a new NATS client with the given configuration
    pub async fn new(config: NatsConfig) -> InfrastructureResult<Self> {
        let connect_options = ConnectOptions::new()
            .name(&config.name)
            .connection_timeout(config.connect_timeout)
            .request_timeout(Some(config.request_timeout));

        let client = async_nats::connect_with_options(config.servers.join(","), connect_options)
            .await
            .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;

        info!("Connected to NATS at {:?}", config.servers);

        Ok(Self { client })
    }

    /// Publish a message to a subject
    pub async fn publish<T>(&self, subject: &str, message: &T) -> InfrastructureResult<()>
    where
        T: Serialize,
    {
        let payload = serde_json::to_vec(message)?;

        self.client
            .publish(subject.to_string(), payload.into())
            .await
            .map_err(|e| InfrastructureError::NatsPublish(e.to_string()))?;

        debug!("Published message to subject: {}", subject);
        Ok(())
    }

    /// Subscribe to a subject
    pub async fn subscribe(&self, subject: &str) -> InfrastructureResult<Subscriber> {
        let subscriber = self
            .client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| InfrastructureError::NatsSubscribe(e.to_string()))?;

        info!("Subscribed to subject: {}", subject);
        Ok(subscriber)
    }

    /// Request-reply pattern
    pub async fn request<T, R>(&self, subject: &str, request: &T) -> InfrastructureResult<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let payload = serde_json::to_vec(request)?;

        let response = self
            .client
            .request(subject.to_string(), payload.into())
            .await
            .map_err(|e| InfrastructureError::NatsPublish(e.to_string()))?;

        let result: R = serde_json::from_slice(&response.payload)
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        Ok(result)
    }

    /// Get the underlying NATS client for advanced operations
    pub fn inner(&self) -> &Client {
        &self.client
    }
}

/// Trait for handling messages from NATS
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// The type of message this handler processes
    type Message: for<'de> Deserialize<'de> + Send;

    /// Handle a message
    async fn handle(&self, message: Self::Message) -> InfrastructureResult<()>;

    /// Get the subject this handler subscribes to
    fn subject(&self) -> &str;
}

/// Message processor that runs handlers for subscriptions
pub struct MessageProcessor {
    client: NatsClient,
}

impl MessageProcessor {
    /// Create a new message processor
    pub fn new(client: NatsClient) -> Self {
        Self { client }
    }

    /// Start processing messages for a specific handler
    pub async fn run_handler<H>(&self, handler: Arc<H>) -> InfrastructureResult<()>
    where
        H: MessageHandler<Message = serde_json::Value> + 'static,
    {
        let subject = handler.subject().to_string();
        let mut subscriber = self.client.subscribe(&subject).await?;

        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                match serde_json::from_slice::<serde_json::Value>(&msg.payload) {
                    Ok(payload) => {
                        if let Err(e) = handler.handle(payload).await {
                            error!("Handler error for subject {}: {}", subject, e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize message on {}: {}", subject, e);
                    }
                }
            }
        });

        Ok(())
    }
}
