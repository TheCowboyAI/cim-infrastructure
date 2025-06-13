//! Tests for NATS infrastructure aligned with user stories

use cim_infrastructure::{NatsClient, NatsConfig, MessageHandler, InfrastructureError, InfrastructureResult};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// User Story: F7 - NATS Message Bus Integration
///
/// As a system architect
/// I want reliable NATS messaging infrastructure
/// So that bounded contexts can communicate through events
///
/// ```mermaid
/// graph LR
///     Context1[Bounded Context 1]
///     Context2[Bounded Context 2]
///     NATS[NATS Message Bus]
///
///     Context1 -->|publish| NATS
///     NATS -->|subscribe| Context2
///
///     NATS -->|guarantees| Delivery[At-least-once delivery]
///     NATS -->|provides| Ordering[Message ordering]
/// ```
///
/// Acceptance Criteria:
/// - Can connect to NATS with configurable timeouts
/// - Can publish messages to subjects
/// - Can subscribe to subjects and receive messages
/// - Handles connection failures gracefully
#[tokio::test]
async fn test_nats_config_creation() {
    // Given NATS configuration parameters
    let config = NatsConfig {
        servers: vec!["nats://localhost:4222".to_string()],
        name: "test-client".to_string(),
        connect_timeout: Duration::from_secs(5),
        request_timeout: Duration::from_secs(30),
    };

    // Then configuration is properly set
    assert_eq!(config.servers, vec!["nats://localhost:4222"]);
    assert_eq!(config.name, "test-client");
    assert_eq!(config.connect_timeout, Duration::from_secs(5));
    assert_eq!(config.request_timeout, Duration::from_secs(30));
}

#[tokio::test]
async fn test_nats_config_default() {
    // Given default configuration
    let config = NatsConfig::default();

    // Then defaults are properly set
    assert_eq!(config.servers, vec!["nats://localhost:4222"]);
    assert_eq!(config.name, "cim-client");
    assert_eq!(config.connect_timeout, Duration::from_secs(10));
    assert_eq!(config.request_timeout, Duration::from_secs(5));
}

/// User Story: F8 - Message Handler Pattern
///
/// As a developer
/// I want a standard message handler interface
/// So that I can process messages consistently across contexts
///
/// ```mermaid
/// sequenceDiagram
///     participant NATS
///     participant Processor
///     participant Handler
///
///     NATS->>Processor: Message arrives
///     Processor->>Handler: handle_message()
///     Handler->>Handler: Process business logic
///     Handler-->>Processor: Result
///     Processor-->>NATS: Acknowledge
/// ```
///
/// Acceptance Criteria:
/// - Handlers implement async trait for message processing
/// - Errors are properly propagated
/// - Messages are acknowledged after successful processing
#[tokio::test]
async fn test_message_handler_implementation() {
    // Given a test message handler
    #[derive(Clone)]
    struct TestHandler {
        messages: Arc<Mutex<Vec<String>>>,
        subject: String,
    }

    #[async_trait]
    impl MessageHandler for TestHandler {
        type Message = Value;

        async fn handle(&self, message: Self::Message) -> InfrastructureResult<()> {
            let msg_str = serde_json::to_string(&message).unwrap();
            self.messages.lock().unwrap().push(msg_str);
            Ok(())
        }

        fn subject(&self) -> &str {
            &self.subject
        }
    }

    // When I create a handler
    let handler = TestHandler {
        messages: Arc::new(Mutex::new(Vec::new())),
        subject: "test.events".to_string(),
    };

    // And process a message
    let test_message = serde_json::json!({
        "type": "TestEvent",
        "data": "test payload"
    });

    handler.handle(test_message).await.unwrap();

    // Then the message is recorded
    let messages = handler.messages.lock().unwrap();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("TestEvent"));
}

/// User Story: F10 - Error Handling
///
/// As a developer
/// I want clear infrastructure errors
/// So that I can handle failures appropriately
///
/// ```mermaid
/// graph TD
///     Op[Infrastructure Operation]
///     Success[Success]
///     ConnErr[Connection Error]
///     PubErr[Publish Error]
///     SubErr[Subscribe Error]
///     SerErr[Serialization Error]
///
///     Op -->|network issue| ConnErr
///     Op -->|publish fails| PubErr
///     Op -->|subscribe fails| SubErr
///     Op -->|bad data| SerErr
/// ```
///
/// Acceptance Criteria:
/// - Different error types for different failures
/// - Errors include context
/// - Errors can be converted to strings for logging
#[test]
fn test_infrastructure_error_types() {
    // Given various infrastructure errors
    let conn_err = InfrastructureError::NatsConnection("connection refused".to_string());
    let pub_err = InfrastructureError::NatsPublish("publish timeout".to_string());
    let sub_err = InfrastructureError::NatsSubscribe("invalid subject".to_string());
    let ser_err = InfrastructureError::Serialization("invalid JSON".to_string());

    // When I format them as strings
    let conn_msg = conn_err.to_string();
    let pub_msg = pub_err.to_string();
    let sub_msg = sub_err.to_string();
    let ser_msg = ser_err.to_string();

    // Then they provide clear error messages
    assert_eq!(conn_msg, "NATS connection error: connection refused");
    assert_eq!(pub_msg, "NATS publish error: publish timeout");
    assert_eq!(sub_msg, "NATS subscribe error: invalid subject");
    assert_eq!(ser_msg, "Serialization error: invalid JSON");
}

// Integration tests that require NATS server running
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestMessage {
    id: String,
    content: String,
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_nats_connection() {
    // Given NATS configuration
    let config = NatsConfig::default();

    // When I connect to NATS
    let result = NatsClient::new(config).await;

    // Then connection is established
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_publish_subscribe_flow() {
    // Given a connected NATS client
    let config = NatsConfig::default();
    let client = NatsClient::new(config).await.unwrap();

    // And a test message
    let subject = "test.integration";
    let message = TestMessage {
        id: "123".to_string(),
        content: "integration test".to_string(),
    };

    // When I subscribe to a subject
    let mut subscriber = client.subscribe(subject).await.unwrap();

    // And publish a message
    client.publish(subject, &message).await.unwrap();

    // Then I receive the message
    if let Some(msg) = futures::StreamExt::next(&mut subscriber).await {
        let received: TestMessage = serde_json::from_slice(&msg.payload).unwrap();
        assert_eq!(received, message);
    } else {
        panic!("No message received");
    }
}
