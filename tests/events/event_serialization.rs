// Copyright (c) 2025 - Cowboy AI, Inc.
//! Event Serialization/Deserialization Tests
//!
//! Task 1.4: Create comprehensive event serialization tests
//!
//! Tests verify:
//! - JSON serialization for all event types
//! - JSON deserialization (round-trip)
//! - Schema stability (field names, structure)
//! - Polymorphic InfrastructureEvent serialization
//!
//! All tests use deterministic fixtures (no side effects)

use cim_infrastructure::domain::ResourceType;
use cim_infrastructure::events::compute_resource::*;
use cim_infrastructure::events::infrastructure::InfrastructureEvent;

// Import test fixtures
use crate::fixtures::*;

#[test]
fn test_resource_registered_serialization() {
    // Arrange
    let event = resource_registered_fixture();

    // Act
    let json = serde_json::to_string(&event).expect("Failed to serialize");

    // Assert - Check that key data is serialized
    assert!(json.contains("server01.example.com"));
    assert!(json.contains(EVENT_ID_1));
    assert!(json.contains(AGGREGATE_ID_1));
    // ResourceType serializes as lowercase snake_case
    assert!(json.contains("physical_server") || json.contains("PhysicalServer"));
}

#[test]
fn test_resource_registered_deserialization() {
    // Arrange
    let event = resource_registered_fixture();
    let json = serde_json::to_string(&event).expect("Failed to serialize");

    // Act
    let deserialized: ResourceRegistered =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Assert
    assert_eq!(deserialized, event);
    assert_eq!(deserialized.event_version, 1);
    assert_eq!(deserialized.event_id, parse_uuid(EVENT_ID_1));
    assert_eq!(deserialized.aggregate_id, parse_uuid(AGGREGATE_ID_1));
    assert_eq!(deserialized.hostname.as_str(), "server01.example.com");
    assert_eq!(deserialized.resource_type, ResourceType::PhysicalServer);
}

#[test]
fn test_resource_registered_round_trip() {
    // Arrange
    let original = resource_registered_fixture();

    // Act
    let json = serde_json::to_string(&original).expect("Serialization failed");
    let deserialized: ResourceRegistered =
        serde_json::from_str(&json).expect("Deserialization failed");

    // Assert
    assert_eq!(deserialized, original);
}

#[test]
fn test_organization_assigned_serialization() {
    // Arrange
    let event = organization_assigned_fixture();

    // Act
    let json = serde_json::to_string(&event).expect("Failed to serialize");

    // Assert - Check that key IDs are serialized
    assert!(json.contains(EVENT_ID_2));
    // EntityId is serialized with its inner UUID
    assert!(json.contains("organization_id")); // field name exists
    assert!(json.contains(CAUSATION_ID_1)); // Has causation
}

#[test]
fn test_organization_assigned_round_trip() {
    // Arrange
    let original = organization_assigned_fixture();

    // Act
    let json = serde_json::to_string(&original).expect("Serialization failed");
    let deserialized: OrganizationAssigned =
        serde_json::from_str(&json).expect("Deserialization failed");

    // Assert
    assert_eq!(deserialized, original);
}

#[test]
fn test_status_changed_serialization() {
    // Arrange
    let event = status_changed_fixture(
        ResourceStatus::Provisioning,
        ResourceStatus::Active,
    );

    // Act
    let json = serde_json::to_string(&event).expect("Failed to serialize");

    // Assert - ResourceStatus serializes as PascalCase variants
    assert!(json.contains("Provisioning") || json.contains("provisioning"));
    assert!(json.contains("Active") || json.contains("active"));
}

#[test]
fn test_status_changed_round_trip() {
    // Arrange
    let original = status_changed_fixture(
        ResourceStatus::Active,
        ResourceStatus::Maintenance,
    );

    // Act
    let json = serde_json::to_string(&original).expect("Serialization failed");
    let deserialized: StatusChanged =
        serde_json::from_str(&json).expect("Deserialization failed");

    // Assert
    assert_eq!(deserialized, original);
}

#[test]
fn test_compute_resource_event_enum_serialization() {
    // Arrange
    let event = ComputeResourceEvent::ResourceRegistered(resource_registered_fixture());

    // Act
    let json = serde_json::to_string(&event).expect("Failed to serialize");

    // Assert
    assert!(json.contains("resource_registered")); // serde tag
    assert!(json.contains("server01.example.com"));
}

#[test]
fn test_compute_resource_event_enum_deserialization() {
    // Arrange
    let original = ComputeResourceEvent::ResourceRegistered(resource_registered_fixture());
    let json = serde_json::to_string(&original).expect("Failed to serialize");

    // Act
    let deserialized: ComputeResourceEvent =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Assert
    match deserialized {
        ComputeResourceEvent::ResourceRegistered(e) => {
            assert_eq!(e.hostname.as_str(), "server01.example.com");
            assert_eq!(e.event_id, parse_uuid(EVENT_ID_1));
        }
        _ => panic!("Wrong event type after deserialization"),
    }
}

#[test]
fn test_infrastructure_event_polymorphic_serialization() {
    // Arrange
    let event = infrastructure_event_fixture();

    // Act
    let json = serde_json::to_string(&event).expect("Failed to serialize");

    // Assert
    assert!(json.contains("compute_resource")); // aggregate_type tag
    assert!(json.contains("resource_registered")); // event type tag
    assert!(json.contains("server01.example.com"));
}

#[test]
fn test_infrastructure_event_polymorphic_deserialization() {
    // Arrange
    let original = infrastructure_event_fixture();
    let json = serde_json::to_string(&original).expect("Failed to serialize");

    // Act
    let deserialized: InfrastructureEvent =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Assert
    match deserialized {
        InfrastructureEvent::ComputeResource(
            ComputeResourceEvent::ResourceRegistered(e)
        ) => {
            assert_eq!(e.hostname.as_str(), "server01.example.com");
            assert_eq!(e.aggregate_id, parse_uuid(AGGREGATE_ID_1));
        }
        _ => panic!("Wrong event type after deserialization"),
    }
}

#[test]
fn test_infrastructure_event_polymorphic_round_trip() {
    // Arrange
    let original = infrastructure_event_fixture();

    // Act
    let json = serde_json::to_string(&original).expect("Serialization failed");
    let deserialized: InfrastructureEvent =
        serde_json::from_str(&json).expect("Deserialization failed");

    // Assert
    assert_eq!(deserialized, original);
}

#[test]
fn test_event_metadata_extraction() {
    // Arrange
    let event = infrastructure_event_fixture();

    // Act
    let aggregate_id = event.aggregate_id();
    let correlation_id = event.correlation_id();
    let timestamp = event.timestamp();
    let version = event.event_version();
    let type_name = event.event_type_name();

    // Assert
    assert_eq!(aggregate_id, parse_uuid(AGGREGATE_ID_1));
    assert_eq!(correlation_id, parse_uuid(CORRELATION_ID_1));
    assert_eq!(timestamp, fixed_timestamp());
    assert_eq!(version, 1);
    assert_eq!(type_name, "ResourceRegistered");
}

#[test]
fn test_json_schema_stability() {
    // Arrange
    let event = resource_registered_fixture();
    let json = serde_json::to_string_pretty(&event).expect("Failed to serialize");

    // Act - Parse as generic JSON to inspect structure
    let value: serde_json::Value =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    // Assert - Verify expected field names exist
    assert!(value.get("event_version").is_some());
    assert!(value.get("event_id").is_some());
    assert!(value.get("aggregate_id").is_some());
    assert!(value.get("timestamp").is_some());
    assert!(value.get("correlation_id").is_some());
    assert!(value.get("causation_id").is_some());
    assert!(value.get("hostname").is_some());
    assert!(value.get("resource_type").is_some());
}

#[test]
fn test_optional_causation_id_serialization() {
    // Arrange - ResourceRegistered has no causation (root event)
    let event1 = resource_registered_fixture();
    assert_eq!(event1.causation_id, None);

    // Act
    let json1 = serde_json::to_string(&event1).expect("Failed to serialize");

    // Assert - Should serialize null
    assert!(json1.contains("\"causation_id\":null"));

    // Arrange - OrganizationAssigned has causation
    let event2 = organization_assigned_fixture();
    assert!(event2.causation_id.is_some());

    // Act
    let json2 = serde_json::to_string(&event2).expect("Failed to serialize");

    // Assert - Should serialize the UUID
    assert!(json2.contains(CAUSATION_ID_1));
}
