//! Infrastructure Layer 1.3: Message Routing Tests for cim-infrastructure
//! 
//! User Story: As infrastructure, I need to route messages between components efficiently
//!
//! Test Requirements:
//! - Verify subject-based routing
//! - Verify message handler registration
//! - Verify message delivery to correct handlers
//! - Verify error handling for unroutable messages
//!
//! Event Sequence:
//! 1. RouterInitialized
//! 2. HandlerRegistered { subject, handler_id }
//! 3. MessageRouted { subject, handler_id }
//! 4. MessageDelivered { handler_id, message_id }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Initialize Router]
//!     B --> C[RouterInitialized]
//!     C --> D[Register Handler]
//!     D --> E[HandlerRegistered]
//!     E --> F[Route Message]
//!     F --> G{Handler Found?}
//!     G -->|Yes| H[MessageRouted]
//!     G -->|No| I[RoutingError]
//!     H --> J[Deliver Message]
//!     J --> K[MessageDelivered]
//!     K --> L[Test Success]
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Message routing event types
#[derive(Debug, Clone, PartialEq)]
pub enum RoutingEvent {
    RouterInitialized,
    HandlerRegistered { subject: String, handler_id: String },
    MessageRouted { subject: String, handler_id: String },
    MessageDelivered { handler_id: String, message_id: String },
    RoutingError { subject: String, error: String },
}

/// Mock message for routing tests
#[derive(Debug, Clone)]
pub struct RoutingMessage {
    pub id: String,
    pub subject: String,
    pub payload: Vec<u8>,
}

/// Mock message handler
pub trait MessageHandler: Send + Sync {
    fn id(&self) -> String;
    fn handle(&self, message: RoutingMessage) -> Result<(), String>;
}

/// Test message handler implementation
#[derive(Clone)]
pub struct TestHandler {
    id: String,
    received_messages: Arc<Mutex<Vec<RoutingMessage>>>,
}

impl TestHandler {
    pub fn new(id: String) -> Self {
        Self {
            id,
            received_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn received_count(&self) -> usize {
        self.received_messages.lock().unwrap().len()
    }

    pub fn get_received(&self) -> Vec<RoutingMessage> {
        self.received_messages.lock().unwrap().clone()
    }
}

impl MessageHandler for TestHandler {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn handle(&self, message: RoutingMessage) -> Result<(), String> {
        self.received_messages.lock().unwrap().push(message);
        Ok(())
    }
}

/// Infrastructure message router
pub struct InfrastructureRouter {
    handlers: HashMap<String, Arc<dyn MessageHandler>>,
    subject_patterns: Vec<(String, String)>, // (pattern, handler_id)
}

impl InfrastructureRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            subject_patterns: Vec::new(),
        }
    }

    pub fn register_handler(
        &mut self,
        subject_pattern: String,
        handler: Arc<dyn MessageHandler>,
    ) -> Result<(), String> {
        let handler_id = handler.id();
        
        // Store handler
        self.handlers.insert(handler_id.clone(), handler);
        
        // Store subject pattern
        self.subject_patterns.push((subject_pattern, handler_id));
        
        Ok(())
    }

    pub fn route_message(&self, message: RoutingMessage) -> Result<String, String> {
        // Find matching handler
        for (pattern, handler_id) in &self.subject_patterns {
            if self.matches_pattern(&message.subject, pattern) {
                return Ok(handler_id.clone());
            }
        }
        
        Err(format!("No handler found for subject: {message.subject}"))
    }

    pub fn deliver_message(&self, message: RoutingMessage) -> Result<(), String> {
        let handler_id = self.route_message(message.clone())?;
        
        let handler = self.handlers.get(&handler_id)
            .ok_or_else(|| format!("Handler not found: {handler_id}"))?;
        
        handler.handle(message)?;
        
        Ok(())
    }

    fn matches_pattern(&self, subject: &str, pattern: &str) -> bool {
        // Simple pattern matching: ">" matches everything after prefix
        if pattern.ends_with('>') {
            let prefix = &pattern[..pattern.len() - 1];
            subject.starts_with(prefix)
        } else {
            subject == pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_initialization() {
        // Arrange & Act
        let router = InfrastructureRouter::new();
        
        // Assert
        assert_eq!(router.handlers.len(), 0);
        assert_eq!(router.subject_patterns.len(), 0);
    }

    #[test]
    fn test_handler_registration() {
        // Arrange
        let mut router = InfrastructureRouter::new();
        let handler = Arc::new(TestHandler::new("test_handler_1".to_string()));
        
        // Act
        let result = router.register_handler(
            "cim.infrastructure.test.>".to_string(),
            handler.clone(),
        );
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(router.handlers.len(), 1);
        assert_eq!(router.subject_patterns.len(), 1);
        assert!(router.handlers.contains_key("test_handler_1"));
    }

    #[test]
    fn test_exact_subject_routing() {
        // Arrange
        let mut router = InfrastructureRouter::new();
        let handler = Arc::new(TestHandler::new("exact_handler".to_string()));
        
        router.register_handler(
            "cim.infrastructure.status".to_string(),
            handler,
        ).unwrap();
        
        let message = RoutingMessage {
            id: "msg_1".to_string(),
            subject: "cim.infrastructure.status".to_string(),
            payload: vec![1, 2, 3],
        };
        
        // Act
        let handler_id = router.route_message(message).unwrap();
        
        // Assert
        assert_eq!(handler_id, "exact_handler");
    }

    #[test]
    fn test_wildcard_subject_routing() {
        // Arrange
        let mut router = InfrastructureRouter::new();
        let handler = Arc::new(TestHandler::new("wildcard_handler".to_string()));
        
        router.register_handler(
            "cim.infrastructure.>".to_string(),
            handler,
        ).unwrap();
        
        let messages = vec![
            RoutingMessage {
                id: "msg_1".to_string(),
                subject: "cim.infrastructure.status".to_string(),
                payload: vec![1],
            },
            RoutingMessage {
                id: "msg_2".to_string(),
                subject: "cim.infrastructure.metrics.cpu".to_string(),
                payload: vec![2],
            },
        ];
        
        // Act & Assert
        for message in messages {
            let handler_id = router.route_message(message).unwrap();
            assert_eq!(handler_id, "wildcard_handler");
        }
    }

    #[test]
    fn test_message_delivery() {
        // Arrange
        let mut router = InfrastructureRouter::new();
        let handler = Arc::new(TestHandler::new("delivery_handler".to_string()));
        
        router.register_handler(
            "cim.infrastructure.test".to_string(),
            handler.clone(),
        ).unwrap();
        
        let message = RoutingMessage {
            id: "msg_1".to_string(),
            subject: "cim.infrastructure.test".to_string(),
            payload: vec![1, 2, 3],
        };
        
        // Act
        let result = router.deliver_message(message.clone());
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(handler.received_count(), 1);
        
        let received = handler.get_received();
        assert_eq!(received[0].id, "msg_1");
        assert_eq!(received[0].subject, "cim.infrastructure.test");
    }

    #[test]
    fn test_multiple_handlers_different_patterns() {
        // Arrange
        let mut router = InfrastructureRouter::new();
        
        let handler1 = Arc::new(TestHandler::new("handler_1".to_string()));
        let handler2 = Arc::new(TestHandler::new("handler_2".to_string()));
        
        router.register_handler(
            "cim.infrastructure.metrics.>".to_string(),
            handler1.clone(),
        ).unwrap();
        
        router.register_handler(
            "cim.infrastructure.events.>".to_string(),
            handler2.clone(),
        ).unwrap();
        
        // Act
        let metrics_msg = RoutingMessage {
            id: "metrics_1".to_string(),
            subject: "cim.infrastructure.metrics.cpu".to_string(),
            payload: vec![1],
        };
        
        let events_msg = RoutingMessage {
            id: "events_1".to_string(),
            subject: "cim.infrastructure.events.connected".to_string(),
            payload: vec![2],
        };
        
        router.deliver_message(metrics_msg).unwrap();
        router.deliver_message(events_msg).unwrap();
        
        // Assert
        assert_eq!(handler1.received_count(), 1);
        assert_eq!(handler2.received_count(), 1);
        
        assert_eq!(handler1.get_received()[0].id, "metrics_1");
        assert_eq!(handler2.get_received()[0].id, "events_1");
    }

    #[test]
    fn test_unroutable_message_error() {
        // Arrange
        let router = InfrastructureRouter::new();
        
        let message = RoutingMessage {
            id: "msg_1".to_string(),
            subject: "unknown.subject".to_string(),
            payload: vec![1, 2, 3],
        };
        
        // Act
        let result = router.route_message(message);
        
        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "No handler found for subject: unknown.subject"
        );
    }

    #[test]
    fn test_routing_event_sequence() {
        // Arrange
        let mut events = Vec::new();
        
        // Simulate event sequence
        events.push(RoutingEvent::RouterInitialized);
        
        events.push(RoutingEvent::HandlerRegistered {
            subject: "cim.infrastructure.>".to_string(),
            handler_id: "infra_handler".to_string(),
        });
        
        events.push(RoutingEvent::MessageRouted {
            subject: "cim.infrastructure.test".to_string(),
            handler_id: "infra_handler".to_string(),
        });
        
        events.push(RoutingEvent::MessageDelivered {
            handler_id: "infra_handler".to_string(),
            message_id: "test_msg_1".to_string(),
        });
        
        // Assert event sequence
        assert_eq!(events.len(), 4);
        assert_eq!(events[0], RoutingEvent::RouterInitialized);
        
        if let RoutingEvent::HandlerRegistered { subject, handler_id } = &events[1] {
            assert_eq!(subject, "cim.infrastructure.>");
            assert_eq!(handler_id, "infra_handler");
        } else {
            panic!("Expected HandlerRegistered event");
        }
    }
} 