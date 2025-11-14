// Copyright 2025 Cowboy AI, LLC.

//! NATS event subscriber for infrastructure events

use async_nats;
use cim_domain_infrastructure::InfrastructureEvent;
use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::Result;

/// Event subscriber for infrastructure events from NATS
pub struct InfrastructureEventSubscriber {
    /// NATS client
    client: async_nats::Client,
    /// Channel for sending events to projection
    event_tx: mpsc::Sender<InfrastructureEvent>,
}

impl InfrastructureEventSubscriber {
    /// Create a new event subscriber
    pub async fn new(
        nats_url: &str,
        event_tx: mpsc::Sender<InfrastructureEvent>,
    ) -> Result<Self> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| crate::error::Neo4jError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            event_tx,
        })
    }

    /// Start subscribing to infrastructure events
    pub async fn start(&self) -> Result<()> {
        info!("Starting infrastructure event subscriber");

        // Subscribe to all infrastructure events using wildcard
        let subject = "infrastructure.>";
        debug!("Subscribing to subject: {}", subject);

        let mut subscriber = self
            .client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| crate::error::Neo4jError::Connection(e.to_string()))?;

        let event_tx = self.event_tx.clone();

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            loop {
                match subscriber.next().await {
                    Some(msg) => {
                        debug!("Received message from subject: {}", msg.subject);

                        // Deserialize event
                        match serde_json::from_slice::<InfrastructureEvent>(&msg.payload) {
                            Ok(event) => {
                                debug!("Deserialized event: {:?}", event);

                                // Send to projection
                                if let Err(e) = event_tx.send(event).await {
                                    error!("Failed to send event to projection: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to deserialize event: {}", e);
                            }
                        }
                    }
                    None => {
                        info!("Subscriber stream ended");
                        break;
                    }
                }
            }
        });

        info!("Infrastructure event subscriber started");
        Ok(())
    }
}
