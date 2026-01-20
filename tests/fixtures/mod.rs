// Copyright (c) 2025 - Cowboy AI, Inc.
//! Test Fixtures for cim-infrastructure
//!
//! Provides deterministic test data for event serialization and deserialization tests.
//! All UUIDs and timestamps are fixed constants to ensure tests are reproducible.
//!
//! # Design Principles
//! - All test data is deterministic (no `Uuid::now_v7()` or `Utc::now()`)
//! - Fixtures are the ONLY place that constructs domain events
//! - Tests use fixtures, never direct construction
//! - Follow FRP axioms: no side effects in tests

use chrono::{DateTime, Utc};
use uuid::Uuid;

use cim_infrastructure::domain::{Hostname, ResourceType};
use cim_infrastructure::events::compute_resource::*;
use cim_infrastructure::events::infrastructure::InfrastructureEvent;

// Fixed test UUIDs (UUID v7 format, but deterministic for testing)
pub const EVENT_ID_1: &str = "01934f4a-0001-7000-8000-000000000001";
pub const EVENT_ID_2: &str = "01934f4a-0002-7000-8000-000000000002";
pub const EVENT_ID_3: &str = "01934f4a-0003-7000-8000-000000000003";

pub const AGGREGATE_ID_1: &str = "01934f4a-1000-7000-8000-000000001000";

pub const ORGANIZATION_ID_1: &str = "01934f4a-2000-7000-8000-000000002000";

pub const CORRELATION_ID_1: &str = "01934f4a-c001-7000-8000-00000000c001";

pub const CAUSATION_ID_1: &str = "01934f4a-a001-7000-8000-00000000a001";

// Fixed test timestamp (2026-01-19T12:00:00Z)
pub const FIXED_TIMESTAMP: &str = "2026-01-19T12:00:00Z";

/// Parse a fixed UUID from a constant string
pub fn parse_uuid(s: &str) -> Uuid {
    Uuid::parse_str(s).expect("Invalid UUID in test fixture")
}

/// Parse the fixed timestamp
pub fn fixed_timestamp() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(FIXED_TIMESTAMP)
        .expect("Invalid timestamp in test fixture")
        .with_timezone(&Utc)
}

/// Create a test ResourceRegistered event with fixed data
pub fn resource_registered_fixture() -> ResourceRegistered {
    ResourceRegistered {
        event_version: 1,
        event_id: parse_uuid(EVENT_ID_1),
        aggregate_id: parse_uuid(AGGREGATE_ID_1),
        timestamp: fixed_timestamp(),
        correlation_id: parse_uuid(CORRELATION_ID_1),
        causation_id: None,
        hostname: Hostname::new("server01.example.com").expect("Invalid hostname"),
        resource_type: ResourceType::PhysicalServer,
    }
}

/// Create a test OrganizationAssigned event
pub fn organization_assigned_fixture() -> OrganizationAssigned {
    use cim_domain::EntityId;
    use cim_domain_organization::Organization;

    OrganizationAssigned {
        event_version: 1,
        event_id: parse_uuid(EVENT_ID_2),
        aggregate_id: parse_uuid(AGGREGATE_ID_1),
        timestamp: fixed_timestamp(),
        correlation_id: parse_uuid(CORRELATION_ID_1),
        causation_id: Some(parse_uuid(CAUSATION_ID_1)),
        organization_id: EntityId::new(),
    }
}

/// Create a test StatusChanged event
pub fn status_changed_fixture(from: ResourceStatus, to: ResourceStatus) -> StatusChanged {
    StatusChanged {
        event_version: 1,
        event_id: parse_uuid(EVENT_ID_3),
        aggregate_id: parse_uuid(AGGREGATE_ID_1),
        timestamp: fixed_timestamp(),
        correlation_id: parse_uuid(CORRELATION_ID_1),
        causation_id: Some(parse_uuid(CAUSATION_ID_1)),
        from_status: from,
        to_status: to,
    }
}

/// Create a test InfrastructureEvent::ComputeResource variant
pub fn infrastructure_event_fixture() -> InfrastructureEvent {
    InfrastructureEvent::ComputeResource(
        ComputeResourceEvent::ResourceRegistered(resource_registered_fixture())
    )
}
