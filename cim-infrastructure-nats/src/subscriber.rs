//! Event subscriber for infrastructure events
//!
//! Subscribes to infrastructure domain events with:
//! - Subject-based filtering
//! - Event deserialization
//! - Async event handlers
//! - Durable subscriptions

use async_nats::{Client, Subscriber};
use tokio_stream::StreamExt;
use std::sync::Arc;
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::event_store::StoredEvent;

/// Error types for event subscription
#[derive(Debug, Error)]
pub enum SubscribeError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Subscribe error: {0}")]
    Subscribe(#[from] async_nats::SubscribeError),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    #[error("Handler error: {0}")]
    Handler(String),
}

/// Result type for subscribe operations
pub type Result<T> = std::result::Result<T, SubscribeError>;

/// Event handler trait
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a received event
    async fn handle(&self, event: StoredEvent) -> Result<()>;

    /// Handle error during event processing
    async fn handle_error(&self, error: SubscribeError) {
        error!(error = %error, "Event handler error");
    }
}

/// Function-based event handler
pub struct FnEventHandler<F>
where
    F: Fn(StoredEvent) -> Result<()> + Send + Sync,
{
    handler: F,
}

impl<F> FnEventHandler<F>
where
    F: Fn(StoredEvent) -> Result<()> + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait::async_trait]
impl<F> EventHandler for FnEventHandler<F>
where
    F: Fn(StoredEvent) -> Result<()> + Send + Sync,
{
    async fn handle(&self, event: StoredEvent) -> Result<()> {
        (self.handler)(event)
    }
}

/// Event subscriber
pub struct EventSubscriber {
    client: Client,
    subject: String,
    handler: Arc<dyn EventHandler>,
}

impl EventSubscriber {
    /// Create a new event subscriber
    pub fn new(client: Client, subject: String, handler: Arc<dyn EventHandler>) -> Self {
        Self {
            client,
            subject,
            handler,
        }
    }

    /// Start subscribing to events
    pub async fn subscribe(self) -> Result<JoinHandle<()>> {
        let subscriber = self.client.subscribe(self.subject.clone()).await?;

        info!(subject = %self.subject, "Started event subscription");

        let handle = tokio::spawn(async move {
            self.process_messages(subscriber).await;
        });

        Ok(handle)
    }

    /// Process messages from subscription
    async fn process_messages(self, mut subscriber: Subscriber) {
        while let Some(message) = subscriber.next().await {
            debug!(
                subject = %message.subject,
                payload_size = message.payload.len(),
                "Received event"
            );

            match self.process_message(&message.payload).await {
                Ok(()) => {
                    debug!("Event processed successfully");
                }
                Err(e) => {
                    self.handler.handle_error(e).await;
                }
            }
        }

        warn!(subject = %self.subject, "Subscription ended");
    }

    /// Process a single message
    async fn process_message(&self, payload: &[u8]) -> Result<()> {
        let stored_event: StoredEvent = serde_json::from_slice(payload)?;

        debug!(
            event_id = %stored_event.event_id,
            event_type = %stored_event.event_type,
            aggregate_id = %stored_event.aggregate_id,
            sequence = stored_event.sequence,
            "Processing stored event"
        );

        self.handler.handle(stored_event).await
    }
}

/// Subscriber builder
pub struct EventSubscriberBuilder {
    client: Option<Client>,
    subject: Option<String>,
    handler: Option<Arc<dyn EventHandler>>,
}

impl EventSubscriberBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            client: None,
            subject: None,
            handler: None,
        }
    }

    /// Set the NATS client
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Set the subject to subscribe to
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set the event handler
    pub fn handler(mut self, handler: Arc<dyn EventHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Build the subscriber
    pub fn build(self) -> Result<EventSubscriber> {
        let client = self
            .client
            .ok_or_else(|| SubscribeError::InvalidEvent("client not set".to_string()))?;

        let subject = self
            .subject
            .ok_or_else(|| SubscribeError::InvalidEvent("subject not set".to_string()))?;

        let handler = self
            .handler
            .ok_or_else(|| SubscribeError::InvalidEvent("handler not set".to_string()))?;

        Ok(EventSubscriber::new(client, subject, handler))
    }
}

impl Default for EventSubscriberBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-subject subscriber for handling multiple event types
pub struct MultiSubjectSubscriber {
    client: Client,
    subjects: Vec<String>,
    handler: Arc<dyn EventHandler>,
}

impl MultiSubjectSubscriber {
    /// Create a new multi-subject subscriber
    pub fn new(client: Client, subjects: Vec<String>, handler: Arc<dyn EventHandler>) -> Self {
        Self {
            client,
            subjects,
            handler,
        }
    }

    /// Start subscribing to all subjects
    pub async fn subscribe(self) -> Result<Vec<JoinHandle<()>>> {
        let mut handles = Vec::new();

        for subject in self.subjects {
            let subscriber = EventSubscriber::new(
                self.client.clone(),
                subject.clone(),
                Arc::clone(&self.handler),
            );

            let handle = subscriber.subscribe().await?;
            handles.push(handle);
        }

        info!(subject_count = handles.len(), "Started multi-subject subscription");
        Ok(handles)
    }
}

/// Event projection trait for building read models
#[async_trait::async_trait]
pub trait EventProjection: Send + Sync {
    /// Project an event to update the read model
    async fn project(&mut self, event: &StoredEvent) -> Result<()>;

    /// Get the current state of the read model
    async fn get_state(&self) -> serde_json::Value;
}

/// Simple in-memory projection
pub struct InMemoryProjection {
    state: serde_json::Value,
}

impl InMemoryProjection {
    pub fn new() -> Self {
        Self {
            state: serde_json::json!({}),
        }
    }
}

impl Default for InMemoryProjection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl EventProjection for InMemoryProjection {
    async fn project(&mut self, event: &StoredEvent) -> Result<()> {
        // Simple projection: just track event count by type
        let counts = self.state.as_object_mut().unwrap();
        let event_type = &event.event_type;

        let count = counts
            .get(event_type)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        counts.insert(event_type.clone(), serde_json::json!(count + 1));

        Ok(())
    }

    async fn get_state(&self) -> serde_json::Value {
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_in_memory_projection() {
        let mut projection = InMemoryProjection::new();

        let event = StoredEvent {
            event_id: Uuid::now_v7(),
            aggregate_id: Uuid::now_v7(),
            sequence: 1,
            timestamp: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            event_type: "ComputeResourceRegistered".to_string(),
            data: serde_json::json!({}),
            metadata: None,
        };

        projection.project(&event).await.unwrap();

        let state = projection.get_state().await;
        assert_eq!(state["ComputeResourceRegistered"], 1);
    }
}
