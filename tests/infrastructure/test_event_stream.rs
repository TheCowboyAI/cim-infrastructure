//! Infrastructure Layer 1.2: Event Store Tests for cim-infrastructure
//! 
//! User Story: As a domain, I need to persist events with CID chains for integrity
//!
//! Test Requirements:
//! - Verify event persistence with CID calculation
//! - Verify CID chain integrity
//! - Verify event replay from store
//! - Verify snapshot creation and restoration
//!
//! Event Sequence:
//! 1. EventStoreInitialized
//! 2. EventPersisted { event_id, cid, previous_cid }
//! 3. CIDChainValidated { start_cid, end_cid, length }
//! 4. EventsReplayed { count, aggregate_id }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Initialize Event Store]
//!     B --> C[EventStoreInitialized]
//!     C --> D[Persist Infrastructure Event]
//!     D --> E[Calculate CID]
//!     E --> F[EventPersisted]
//!     F --> G[Validate Chain]
//!     G --> H[CIDChainValidated]
//!     H --> I[Replay Events]
//!     I --> J[EventsReplayed]
//!     J --> K[Test Success]
//! ```

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Mock CID type for infrastructure testing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InfrastructureCid(String);

impl InfrastructureCid {
    pub fn from_data(data: &[u8], previous: Option<&InfrastructureCid>) -> Self {
        // Infrastructure-specific CID calculation
        let mut hash_data = data.to_vec();
        if let Some(prev) = previous {
            hash_data.extend_from_slice(prev.0.as_bytes());
        }
        
        let hash = hash_data.iter().fold(0u64, |acc, &b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        
        InfrastructureCid(format!("infra_cid_{:016x}", hash))
    }
}

/// Infrastructure event for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureEventData {
    pub event_id: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub sequence: u64,
    pub payload: Vec<u8>,
}

/// Infrastructure event store for testing
pub struct InfrastructureEventStore {
    events: Vec<(InfrastructureEventData, InfrastructureCid, Option<InfrastructureCid>)>,
    snapshots: HashMap<String, Vec<u8>>,
    streams: HashMap<String, Vec<usize>>, // aggregate_id -> event indices
}

impl InfrastructureEventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            snapshots: HashMap::new(),
            streams: HashMap::new(),
        }
    }

    pub fn persist_event(
        &mut self,
        event: InfrastructureEventData,
        previous_cid: Option<InfrastructureCid>,
    ) -> Result<InfrastructureCid, String> {
        // Serialize event for CID calculation
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        let cid = InfrastructureCid::from_data(&event_bytes, previous_cid.as_ref());
        
        let event_index = self.events.len();
        self.events.push((event.clone(), cid.clone(), previous_cid));
        
        // Update stream index
        self.streams
            .entry(event.aggregate_id.clone())
            .or_insert_with(Vec::new)
            .push(event_index);
        
        Ok(cid)
    }

    pub fn validate_chain(&self, aggregate_id: &str) -> Result<(InfrastructureCid, InfrastructureCid, usize), String> {
        let indices = self.streams.get(aggregate_id)
            .ok_or_else(|| format!("No events for aggregate {}", aggregate_id))?;
        
        if indices.is_empty() {
            return Err("No events in stream".to_string());
        }

        // Validate chain for this aggregate
        for i in 1..indices.len() {
            let (_, _, prev_cid) = &self.events[indices[i]];
            let (_, expected_prev_cid, _) = &self.events[indices[i - 1]];
            
            if prev_cid.as_ref() != Some(expected_prev_cid) {
                return Err(format!("Chain broken at position {} for aggregate {}", i, aggregate_id));
            }
        }

        let start_cid = self.events[indices[0]].1.clone();
        let end_cid = self.events[indices[indices.len() - 1]].1.clone();
        
        Ok((start_cid, end_cid, indices.len()))
    }

    pub fn replay_events(&self, aggregate_id: &str) -> Vec<InfrastructureEventData> {
        self.streams.get(aggregate_id)
            .map(|indices| {
                indices.iter()
                    .map(|&i| self.events[i].0.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn create_snapshot(&mut self, aggregate_id: &str, data: Vec<u8>) -> Result<(), String> {
        self.snapshots.insert(aggregate_id.to_string(), data);
        Ok(())
    }

    pub fn restore_snapshot(&self, aggregate_id: &str) -> Option<Vec<u8>> {
        self.snapshots.get(aggregate_id).cloned()
    }

    pub fn get_latest_cid(&self, aggregate_id: &str) -> Option<InfrastructureCid> {
        self.streams.get(aggregate_id)
            .and_then(|indices| indices.last())
            .map(|&i| self.events[i].1.clone())
    }
}

/// Event types for event store testing
#[derive(Debug, Clone, PartialEq)]
pub enum EventStoreEvent {
    EventStoreInitialized,
    EventPersisted { 
        event_id: String, 
        cid: InfrastructureCid, 
        previous_cid: Option<InfrastructureCid> 
    },
    CIDChainValidated { 
        start_cid: InfrastructureCid, 
        end_cid: InfrastructureCid, 
        length: usize 
    },
    EventsReplayed { 
        count: usize, 
        aggregate_id: String 
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infrastructure_event_store_initialization() {
        // Arrange & Act
        let store = InfrastructureEventStore::new();
        
        // Assert
        assert_eq!(store.events.len(), 0);
        assert_eq!(store.snapshots.len(), 0);
        assert_eq!(store.streams.len(), 0);
    }

    #[test]
    fn test_infrastructure_event_persistence() {
        // Arrange
        let mut store = InfrastructureEventStore::new();
        let event = InfrastructureEventData {
            event_id: "infra_evt_1".to_string(),
            aggregate_id: "infra_agg_1".to_string(),
            event_type: "ClientConnected".to_string(),
            sequence: 1,
            payload: vec![1, 2, 3, 4],
        };

        // Act
        let cid = store.persist_event(event.clone(), None).unwrap();

        // Assert
        assert!(cid.0.starts_with("infra_cid_"));
        assert_eq!(store.events.len(), 1);
        assert_eq!(store.streams.get("infra_agg_1").unwrap().len(), 1);
        
        let (stored_event, stored_cid, prev_cid) = &store.events[0];
        assert_eq!(stored_event.event_id, "infra_evt_1");
        assert_eq!(stored_cid, &cid);
        assert_eq!(prev_cid, &None);
    }

    #[test]
    fn test_infrastructure_cid_chain_integrity() {
        // Arrange
        let mut store = InfrastructureEventStore::new();
        let aggregate_id = "infra_agg_1";
        
        // Create a chain of infrastructure events
        let events = vec![
            InfrastructureEventData {
                event_id: "evt_1".to_string(),
                aggregate_id: aggregate_id.to_string(),
                event_type: "StreamCreated".to_string(),
                sequence: 1,
                payload: vec![1, 2, 3],
            },
            InfrastructureEventData {
                event_id: "evt_2".to_string(),
                aggregate_id: aggregate_id.to_string(),
                event_type: "EventPublished".to_string(),
                sequence: 2,
                payload: vec![4, 5, 6],
            },
            InfrastructureEventData {
                event_id: "evt_3".to_string(),
                aggregate_id: aggregate_id.to_string(),
                event_type: "EventConsumed".to_string(),
                sequence: 3,
                payload: vec![7, 8, 9],
            },
        ];

        // Act
        let mut previous_cid = None;
        let mut cids = Vec::new();
        
        for event in events {
            let cid = store.persist_event(event, previous_cid.clone()).unwrap();
            cids.push(cid.clone());
            previous_cid = Some(cid);
        }

        // Validate chain
        let (start_cid, end_cid, length) = store.validate_chain(aggregate_id).unwrap();

        // Assert
        assert_eq!(start_cid, cids[0]);
        assert_eq!(end_cid, cids[2]);
        assert_eq!(length, 3);
    }

    #[test]
    fn test_infrastructure_event_replay() {
        // Arrange
        let mut store = InfrastructureEventStore::new();
        let aggregate_id = "infra_agg_1";
        
        // Add events for infrastructure aggregate
        for i in 1..=3 {
            let event = InfrastructureEventData {
                event_id: format!("evt_{}", i),
                aggregate_id: aggregate_id.to_string(),
                event_type: format!("Event{}", i),
                sequence: i as u64,
                payload: vec![i as u8],
            };
            store.persist_event(event, store.get_latest_cid(aggregate_id)).ok();
        }
        
        // Add event for different aggregate
        let other_event = InfrastructureEventData {
            event_id: "evt_other".to_string(),
            aggregate_id: "infra_agg_2".to_string(),
            event_type: "OtherEvent".to_string(),
            sequence: 1,
            payload: vec![99],
        };
        store.persist_event(other_event, None).ok();

        // Act
        let replayed = store.replay_events(aggregate_id);

        // Assert
        assert_eq!(replayed.len(), 3);
        assert_eq!(replayed[0].event_id, "evt_1");
        assert_eq!(replayed[1].event_id, "evt_2");
        assert_eq!(replayed[2].event_id, "evt_3");
        
        // Verify other aggregate not included
        let other_replayed = store.replay_events("infra_agg_2");
        assert_eq!(other_replayed.len(), 1);
    }

    #[test]
    fn test_infrastructure_snapshot_management() {
        // Arrange
        let mut store = InfrastructureEventStore::new();
        let aggregate_id = "infra_agg_1";
        let snapshot_data = vec![100, 200, 150, 75, 50];

        // Act
        store.create_snapshot(aggregate_id, snapshot_data.clone()).unwrap();
        let restored = store.restore_snapshot(aggregate_id);

        // Assert
        assert_eq!(restored, Some(snapshot_data));
        
        // Test non-existent snapshot
        let missing = store.restore_snapshot("non_existent");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_broken_chain_detection_infrastructure() {
        // Arrange
        let mut store = InfrastructureEventStore::new();
        let aggregate_id = "infra_agg_1";
        
        // Create first event properly
        let event1 = InfrastructureEventData {
            event_id: "evt_1".to_string(),
            aggregate_id: aggregate_id.to_string(),
            event_type: "Event1".to_string(),
            sequence: 1,
            payload: vec![1],
        };
        
        let _cid1 = store.persist_event(event1, None).unwrap();
        
        // Manually break the chain by adding event with wrong previous CID
        let event2 = InfrastructureEventData {
            event_id: "evt_2".to_string(),
            aggregate_id: aggregate_id.to_string(),
            event_type: "Event2".to_string(),
            sequence: 2,
            payload: vec![2],
        };
        
        // Create a wrong CID
        let wrong_cid = InfrastructureCid("wrong_infrastructure_cid".to_string());
        
        // Manually insert with wrong previous CID
        let event_bytes = serde_json::to_vec(&event2).unwrap();
        let cid2 = InfrastructureCid::from_data(&event_bytes, Some(&wrong_cid));
        
        store.events.push((event2, cid2, Some(wrong_cid)));
        store.streams.get_mut(aggregate_id).unwrap().push(1);

        // Act
        let result = store.validate_chain(aggregate_id);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain broken at position 1"));
    }
} 