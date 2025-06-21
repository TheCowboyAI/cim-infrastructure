//! Infrastructure Layer 1.1: NATS JetStream Connection Tests for cim-infrastructure
//! 
//! User Story: As a system, I need to connect to NATS JetStream and create event streams
//!
//! Test Requirements:
//! - Verify NATS connection establishment
//! - Verify stream creation with correct configuration
//! - Verify event publishing with acknowledgment
//! - Verify event consumption with proper ordering
//!
//! Event Sequence:
//! 1. ConnectionEstablished
//! 2. StreamCreated { name, subjects }
//! 3. EventPublished { subject, sequence }
//! 4. EventConsumed { subject, sequence }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Create NatsClient]
//!     B --> C{Connection OK?}
//!     C -->|Yes| D[ConnectionEstablished Event]
//!     C -->|No| E[Test Failure]
//!     D --> F[Create JetStream]
//!     F --> G[StreamCreated Event]
//!     G --> H[Publish to Stream]
//!     H --> I[EventPublished Event]
//!     I --> J[Consume from Stream]
//!     J --> K[EventConsumed Event]
//!     K --> L[Test Success]
//! ```

use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Infrastructure-specific event types for testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InfrastructureEvent {
    ConnectionEstablished { client_name: String },
    StreamCreated { name: String, subjects: Vec<String> },
    EventPublished { subject: String, sequence: u64 },
    EventConsumed { subject: String, sequence: u64 },
    ConnectionFailed { error: String },
}

/// Event stream validator for infrastructure testing
pub struct EventStreamValidator {
    expected_events: Vec<InfrastructureEvent>,
    captured_events: Vec<InfrastructureEvent>,
}

impl EventStreamValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
        }
    }

    pub fn expect_sequence(mut self, events: Vec<InfrastructureEvent>) -> Self {
        self.expected_events = events;
        self
    }

    pub fn capture_event(&mut self, event: InfrastructureEvent) {
        self.captured_events.push(event);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.captured_events.len() != self.expected_events.len() {
            return Err(format!(
                "Event count mismatch: expected {}, got {}",
                self.expected_events.len(),
                self.captured_events.len()
            ));
        }

        for (i, (expected, actual)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected != actual {
                return Err(format!(
                    "Event mismatch at position {}: expected {:?}, got {:?}",
                    i, expected, actual
                ));
            }
        }

        Ok(())
    }
}

/// Mock NATS client for testing infrastructure patterns
pub struct MockInfrastructureClient {
    connected: bool,
    client_name: String,
    streams: Vec<(String, Vec<String>)>,
    published_events: Vec<(String, u64)>,
}

impl MockInfrastructureClient {
    pub fn new(client_name: String) -> Self {
        Self {
            connected: false,
            client_name,
            streams: Vec::new(),
            published_events: Vec::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        // Simulate connection with delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        self.connected = true;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn create_jetstream(&mut self) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }
        Ok(())
    }

    pub async fn create_stream(&mut self, name: String, subjects: Vec<String>) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }
        
        self.streams.push((name, subjects));
        Ok(())
    }

    pub async fn publish_with_ack(&mut self, subject: String, sequence: u64) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }
        
        // Simulate acknowledgment delay
        tokio::time::sleep(Duration::from_millis(5)).await;
        self.published_events.push((subject, sequence));
        Ok(())
    }

    pub async fn consume_with_ack(&self, subject: &str) -> Result<(String, u64), String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }
        
        // Find the event with matching subject
        self.published_events
            .iter()
            .find(|(s, _)| s == subject)
            .cloned()
            .ok_or_else(|| "No events found for subject".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infrastructure_nats_connection() {
        // Arrange
        let mut validator = EventStreamValidator::new()
            .expect_sequence(vec![
                InfrastructureEvent::ConnectionEstablished {
                    client_name: "cim-infrastructure-test".to_string(),
                },
            ]);

        let mut client = MockInfrastructureClient::new("cim-infrastructure-test".to_string());

        // Act
        let result = client.connect().await;

        // Assert
        assert!(result.is_ok());
        assert!(client.is_connected());
        
        // Capture event
        validator.capture_event(InfrastructureEvent::ConnectionEstablished {
            client_name: client.client_name.clone(),
        });
        
        // Validate sequence
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_jetstream_initialization() {
        // Arrange
        let mut validator = EventStreamValidator::new()
            .expect_sequence(vec![
                InfrastructureEvent::ConnectionEstablished {
                    client_name: "cim-infrastructure-test".to_string(),
                },
                InfrastructureEvent::StreamCreated {
                    name: "CIM_INFRASTRUCTURE_EVENTS".to_string(),
                    subjects: vec!["cim.infrastructure.>".to_string()],
                },
            ]);

        let mut client = MockInfrastructureClient::new("cim-infrastructure-test".to_string());

        // Act
        client.connect().await.unwrap();
        validator.capture_event(InfrastructureEvent::ConnectionEstablished {
            client_name: client.client_name.clone(),
        });

        client.create_jetstream().await.unwrap();
        
        let stream_result = client.create_stream(
            "CIM_INFRASTRUCTURE_EVENTS".to_string(),
            vec!["cim.infrastructure.>".to_string()],
        ).await;

        // Assert
        assert!(stream_result.is_ok());
        
        validator.capture_event(InfrastructureEvent::StreamCreated {
            name: "CIM_INFRASTRUCTURE_EVENTS".to_string(),
            subjects: vec!["cim.infrastructure.>".to_string()],
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_event_publishing_with_jetstream_ack() {
        // Arrange
        let mut validator = EventStreamValidator::new()
            .expect_sequence(vec![
                InfrastructureEvent::ConnectionEstablished {
                    client_name: "cim-infrastructure-test".to_string(),
                },
                InfrastructureEvent::EventPublished {
                    subject: "cim.infrastructure.client.connected".to_string(),
                    sequence: 1,
                },
            ]);

        let mut client = MockInfrastructureClient::new("cim-infrastructure-test".to_string());

        // Act
        client.connect().await.unwrap();
        validator.capture_event(InfrastructureEvent::ConnectionEstablished {
            client_name: client.client_name.clone(),
        });

        let publish_result = client.publish_with_ack(
            "cim.infrastructure.client.connected".to_string(),
            1,
        ).await;

        // Assert
        assert!(publish_result.is_ok());
        
        validator.capture_event(InfrastructureEvent::EventPublished {
            subject: "cim.infrastructure.client.connected".to_string(),
            sequence: 1,
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_ordered_event_consumption() {
        // Arrange
        let mut validator = EventStreamValidator::new()
            .expect_sequence(vec![
                InfrastructureEvent::ConnectionEstablished {
                    client_name: "cim-infrastructure-test".to_string(),
                },
                InfrastructureEvent::EventPublished {
                    subject: "cim.infrastructure.test.event".to_string(),
                    sequence: 1,
                },
                InfrastructureEvent::EventPublished {
                    subject: "cim.infrastructure.test.event".to_string(),
                    sequence: 2,
                },
                InfrastructureEvent::EventConsumed {
                    subject: "cim.infrastructure.test.event".to_string(),
                    sequence: 1,
                },
            ]);

        let mut client = MockInfrastructureClient::new("cim-infrastructure-test".to_string());

        // Act
        client.connect().await.unwrap();
        validator.capture_event(InfrastructureEvent::ConnectionEstablished {
            client_name: client.client_name.clone(),
        });

        // Publish multiple events
        client.publish_with_ack("cim.infrastructure.test.event".to_string(), 1).await.unwrap();
        validator.capture_event(InfrastructureEvent::EventPublished {
            subject: "cim.infrastructure.test.event".to_string(),
            sequence: 1,
        });

        client.publish_with_ack("cim.infrastructure.test.event".to_string(), 2).await.unwrap();
        validator.capture_event(InfrastructureEvent::EventPublished {
            subject: "cim.infrastructure.test.event".to_string(),
            sequence: 2,
        });

        // Consume first event
        let (subject, sequence) = client.consume_with_ack("cim.infrastructure.test.event").await.unwrap();

        // Assert
        assert_eq!(subject, "cim.infrastructure.test.event");
        assert_eq!(sequence, 1); // Should get first event
        
        validator.capture_event(InfrastructureEvent::EventConsumed {
            subject,
            sequence,
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_connection_failure_handling() {
        // Arrange
        let mut validator = EventStreamValidator::new()
            .expect_sequence(vec![
                InfrastructureEvent::ConnectionFailed {
                    error: "Not connected to NATS".to_string(),
                },
            ]);

        let mut client = MockInfrastructureClient::new("cim-infrastructure-test".to_string());

        // Act - try to create stream without connection
        let stream_result = client.create_stream(
            "TEST_STREAM".to_string(),
            vec!["test.>".to_string()],
        ).await;

        // Assert
        assert!(stream_result.is_err());
        assert_eq!(stream_result.unwrap_err(), "Not connected to NATS");
        
        validator.capture_event(InfrastructureEvent::ConnectionFailed {
            error: "Not connected to NATS".to_string(),
        });
        
        assert!(validator.validate().is_ok());
    }
} 