// Copyright (c) 2025 - Cowboy AI, Inc.
//! Event Versioning Tests
//!
//! Task 1.5: Tests for event schema evolution through upcasting
//!
//! Tests verify:
//! - Upcasting from v1 to v2 events
//! - Upcaster chains for multi-version migration
//! - Version validation
//! - JSON transformation correctness
//! - Error handling for unsupported versions

use cim_infrastructure::events::compute_resource::ResourceRegistered;
use cim_infrastructure::events::versioning::*;
use serde_json::json;

// Import test fixtures for deterministic data
use crate::fixtures::*;

/// Example upcaster: ResourceRegistered v1 → v2
///
/// V2 adds a new optional "tags" field for resource categorization
struct ResourceRegisteredV1ToV2;

impl Upcaster<ResourceRegistered> for ResourceRegisteredV1ToV2 {
    fn from_version(&self) -> u32 {
        1
    }

    fn to_version(&self) -> u32 {
        2
    }

    fn upcast(&self, mut value: serde_json::Value) -> Result<serde_json::Value, UpcastError> {
        // V1 events didn't have tags - add empty array as default
        if let Some(obj) = value.as_object_mut() {
            obj.insert("tags".to_string(), json!([]));
            set_event_version(&mut value, 2)?;
            Ok(value)
        } else {
            Err(UpcastError::TransformationFailed(
                "Event is not a JSON object".to_string(),
            ))
        }
    }

    fn validate(&self, value: &serde_json::Value) -> Result<(), UpcastError> {
        // Verify tags field exists
        if value.get("tags").is_none() {
            return Err(UpcastError::MissingField("tags".to_string()));
        }
        Ok(())
    }
}

/// Example upcaster: ResourceRegistered v2 → v3
///
/// V3 renames "hostname" to "fqdn" for clarity
struct ResourceRegisteredV2ToV3;

impl Upcaster<ResourceRegistered> for ResourceRegisteredV2ToV3 {
    fn from_version(&self) -> u32 {
        2
    }

    fn to_version(&self) -> u32 {
        3
    }

    fn upcast(&self, mut value: serde_json::Value) -> Result<serde_json::Value, UpcastError> {
        if let Some(obj) = value.as_object_mut() {
            // Rename hostname to fqdn
            if let Some(hostname) = obj.remove("hostname") {
                obj.insert("fqdn".to_string(), hostname);
            } else {
                return Err(UpcastError::MissingField("hostname".to_string()));
            }

            set_event_version(&mut value, 3)?;
            Ok(value)
        } else {
            Err(UpcastError::TransformationFailed(
                "Event is not a JSON object".to_string(),
            ))
        }
    }

    fn validate(&self, value: &serde_json::Value) -> Result<(), UpcastError> {
        // Verify fqdn field exists and hostname is gone
        if value.get("fqdn").is_none() {
            return Err(UpcastError::MissingField("fqdn".to_string()));
        }
        if value.get("hostname").is_some() {
            return Err(UpcastError::InvalidFieldValue {
                field: "hostname".to_string(),
                reason: "Should have been renamed to fqdn".to_string(),
            });
        }
        Ok(())
    }
}

#[test]
fn test_upcast_v1_to_v2() {
    // Arrange - Create a v1 event JSON
    let v1_json = json!({
        "event_version": 1,
        "event_id": EVENT_ID_1,
        "aggregate_id": AGGREGATE_ID_1,
        "timestamp": FIXED_TIMESTAMP,
        "correlation_id": CORRELATION_ID_1,
        "causation_id": null,
        "hostname": "server01.example.com",
        "resource_type": "physical_server"
    });

    // Act
    let upcaster = ResourceRegisteredV1ToV2;
    let v2_json = upcaster.upcast(v1_json.clone()).expect("Upcast failed");

    // Assert
    assert_eq!(get_event_version(&v2_json).unwrap(), 2);
    assert!(v2_json.get("tags").is_some());
    assert_eq!(v2_json["tags"], json!([]));
    // Original fields preserved
    assert_eq!(v2_json["hostname"], "server01.example.com");
}

#[test]
fn test_upcast_v2_to_v3() {
    // Arrange - Create a v2 event JSON
    let v2_json = json!({
        "event_version": 2,
        "event_id": EVENT_ID_1,
        "aggregate_id": AGGREGATE_ID_1,
        "timestamp": FIXED_TIMESTAMP,
        "correlation_id": CORRELATION_ID_1,
        "causation_id": null,
        "hostname": "server01.example.com",
        "resource_type": "physical_server",
        "tags": ["production", "web-server"]
    });

    // Act
    let upcaster = ResourceRegisteredV2ToV3;
    let v3_json = upcaster.upcast(v2_json.clone()).expect("Upcast failed");

    // Assert
    assert_eq!(get_event_version(&v3_json).unwrap(), 3);
    assert!(v3_json.get("fqdn").is_some());
    assert_eq!(v3_json["fqdn"], "server01.example.com");
    assert!(v3_json.get("hostname").is_none()); // Renamed
    // Tags preserved
    assert_eq!(v3_json["tags"], json!(["production", "web-server"]));
}

#[test]
fn test_upcast_chain_v1_to_v3() {
    // Arrange - Create a v1 event and chain of upcasters
    let v1_json = json!({
        "event_version": 1,
        "event_id": EVENT_ID_1,
        "aggregate_id": AGGREGATE_ID_1,
        "timestamp": FIXED_TIMESTAMP,
        "correlation_id": CORRELATION_ID_1,
        "causation_id": null,
        "hostname": "server01.example.com",
        "resource_type": "physical_server"
    });

    let mut chain: UpcasterChain<ResourceRegistered> = UpcasterChain::new();
    chain.add(ResourceRegisteredV1ToV2);
    chain.add(ResourceRegisteredV2ToV3);

    // Act - Upcast from v1 to latest (v3)
    let v3_json = chain.upcast_to_latest(v1_json, 1).expect("Chain upcast failed");

    // Assert
    assert_eq!(get_event_version(&v3_json).unwrap(), 3);
    assert!(v3_json.get("fqdn").is_some());
    assert_eq!(v3_json["fqdn"], "server01.example.com");
    assert!(v3_json.get("hostname").is_none());
    assert!(v3_json.get("tags").is_some());
    assert_eq!(v3_json["tags"], json!([]));
}

#[test]
fn test_upcast_chain_to_intermediate_version() {
    // Arrange
    let v1_json = json!({
        "event_version": 1,
        "event_id": EVENT_ID_1,
        "hostname": "server01.example.com"
    });

    let mut chain: UpcasterChain<ResourceRegistered> = UpcasterChain::new();
    chain.add(ResourceRegisteredV1ToV2);
    chain.add(ResourceRegisteredV2ToV3);

    // Act - Upcast only to v2, not v3
    let v2_json = chain
        .upcast_to_version(v1_json, 1, 2)
        .expect("Upcast to v2 failed");

    // Assert
    assert_eq!(get_event_version(&v2_json).unwrap(), 2);
    assert!(v2_json.get("tags").is_some());
    assert!(v2_json.get("hostname").is_some()); // Not renamed yet
}

#[test]
fn test_upcast_chain_already_latest_version() {
    // Arrange - Event is already at latest version
    let v3_json = json!({
        "event_version": 3,
        "event_id": EVENT_ID_1,
        "fqdn": "server01.example.com",
        "tags": []
    });

    let mut chain: UpcasterChain<ResourceRegistered> = UpcasterChain::new();
    chain.add(ResourceRegisteredV1ToV2);
    chain.add(ResourceRegisteredV2ToV3);

    // Act
    let result = chain.upcast_to_latest(v3_json.clone(), 3).expect("Should not fail");

    // Assert - No transformation needed
    assert_eq!(result, v3_json);
}

#[test]
fn test_upcast_chain_skip_intermediate() {
    // Arrange - Event is v2, skip to v3
    let v2_json = json!({
        "event_version": 2,
        "event_id": EVENT_ID_1,
        "hostname": "server01.example.com",
        "tags": ["production"]
    });

    let mut chain: UpcasterChain<ResourceRegistered> = UpcasterChain::new();
    chain.add(ResourceRegisteredV1ToV2);
    chain.add(ResourceRegisteredV2ToV3);

    // Act - Starting from v2, should only apply v2→v3 upcaster
    let v3_json = chain.upcast_to_latest(v2_json, 2).expect("Upcast failed");

    // Assert
    assert_eq!(get_event_version(&v3_json).unwrap(), 3);
    assert_eq!(v3_json["fqdn"], "server01.example.com");
    assert_eq!(v3_json["tags"], json!(["production"]));
}

#[test]
fn test_upcaster_validation_failure() {
    // Arrange - Create event without required field
    let invalid_v2_json = json!({
        "event_version": 2,
        "event_id": EVENT_ID_1,
        // Missing hostname field that v2→v3 requires
        "tags": []
    });

    // Act
    let upcaster = ResourceRegisteredV2ToV3;
    let result = upcaster.upcast(invalid_v2_json);

    // Assert - Should fail validation
    assert!(result.is_err());
    match result.unwrap_err() {
        UpcastError::MissingField(field) => {
            assert_eq!(field, "hostname");
        }
        _ => panic!("Expected MissingField error"),
    }
}

#[test]
fn test_get_event_version_helper() {
    // Arrange
    let json = json!({
        "event_version": 5,
        "data": "test"
    });

    // Act & Assert
    assert_eq!(get_event_version(&json).unwrap(), 5);
}

#[test]
fn test_set_event_version_helper() {
    // Arrange
    let mut json = json!({
        "event_version": 1,
        "data": "test"
    });

    // Act
    set_event_version(&mut json, 3).unwrap();

    // Assert
    assert_eq!(get_event_version(&json).unwrap(), 3);
}

#[test]
fn test_event_version_info_builder() {
    // Arrange & Act
    let info = EventVersionInfo::new("ResourceRegistered", 2)
        .introduced_at("2026-01-19")
        .with_change("Added tags field for categorization")
        .with_change("Made description field optional");

    // Assert
    assert_eq!(info.event_type, "ResourceRegistered");
    assert_eq!(info.version, 2);
    assert_eq!(info.changes.len(), 2);
    assert!(!info.deprecated);
}

#[test]
fn test_event_version_info_deprecated() {
    // Arrange & Act
    let info = EventVersionInfo::new("OldEventType", 1)
        .introduced_at("2020-01-01")
        .with_change("Initial version")
        .deprecated();

    // Assert
    assert!(info.deprecated);
}

#[test]
fn test_upcast_error_display() {
    // Test each error variant displays correctly
    let err1 = UpcastError::UnsupportedVersion {
        from: 1,
        to: 2,
        found: 3,
    };
    assert!(err1.to_string().contains("version 1"));

    let err2 = UpcastError::TransformationFailed("test error".to_string());
    assert!(err2.to_string().contains("test error"));

    let err3 = UpcastError::MissingField("field1".to_string());
    assert!(err3.to_string().contains("field1"));

    let err4 = UpcastError::InvalidFieldValue {
        field: "field2".to_string(),
        reason: "bad value".to_string(),
    };
    assert!(err4.to_string().contains("field2"));
    assert!(err4.to_string().contains("bad value"));
}

#[test]
fn test_upcast_chain_latest_version() {
    // Arrange
    let mut chain: UpcasterChain<ResourceRegistered> = UpcasterChain::new();

    // Act & Assert - Empty chain
    assert_eq!(chain.latest_version(), None);

    // Add upcasters
    chain.add(ResourceRegisteredV1ToV2);
    assert_eq!(chain.latest_version(), Some(2));

    chain.add(ResourceRegisteredV2ToV3);
    assert_eq!(chain.latest_version(), Some(3));
}

#[test]
fn test_upcast_to_version_backwards_fails() {
    // Arrange - Try to downgrade from v2 to v1 (not supported)
    let v2_json = json!({
        "event_version": 2,
        "event_id": EVENT_ID_1,
        "hostname": "test",
        "tags": []
    });

    let mut chain: UpcasterChain<ResourceRegistered> = UpcasterChain::new();
    chain.add(ResourceRegisteredV1ToV2);

    // Act
    let result = chain.upcast_to_version(v2_json, 2, 1);

    // Assert - Should fail (can't go backwards)
    assert!(result.is_err());
    match result.unwrap_err() {
        UpcastError::UnsupportedVersion { from, to, found } => {
            assert_eq!(from, 2);
            assert_eq!(to, 1);
            assert_eq!(found, 2);
        }
        _ => panic!("Expected UnsupportedVersion error"),
    }
}

#[test]
fn test_upcast_to_same_version_is_noop() {
    // Arrange
    let v2_json = json!({
        "event_version": 2,
        "event_id": EVENT_ID_1,
        "hostname": "test",
        "tags": []
    });

    let mut chain: UpcasterChain<ResourceRegistered> = UpcasterChain::new();
    chain.add(ResourceRegisteredV1ToV2);

    // Act - Upcast v2 to v2 (no-op)
    let result = chain
        .upcast_to_version(v2_json.clone(), 2, 2)
        .expect("Should not fail");

    // Assert - Should return unchanged
    assert_eq!(result, v2_json);
}
