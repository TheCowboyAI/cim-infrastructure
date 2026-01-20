// Copyright (c) 2025 - Cowboy AI, Inc.
//! Integration tests for aggregate event application and state reconstruction
//!
//! These tests verify the complete flow:
//! 1. Handle command → generate event
//! 2. Apply event → produce new state
//! 3. Reconstruct state from event stream
//!
//! This demonstrates the core event sourcing pattern.

use cim_domain::EntityId;
use cim_domain_policy::PolicyId;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use cim_infrastructure::aggregate::{
    AddPolicyCommand, AssignOrganizationCommand, ChangeStatusCommand, CommandError,
    ComputeResourceState, RegisterResourceCommand, apply_event,
    handle_add_policy, handle_assign_organization, handle_change_status, handle_register_resource,
};
use cim_infrastructure::domain::{Hostname, ResourceType};
use cim_infrastructure::events::{ComputeResourceEvent, ResourceStatus};

// Test fixtures
fn test_timestamp() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-01-19T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

fn test_aggregate_id() -> Uuid {
    Uuid::parse_str("01934f4a-1000-7000-8000-000000001000").unwrap()
}

/// Test: Complete resource lifecycle from registration to active
#[test]
fn test_complete_resource_lifecycle() {
    // Start with empty state
    let aggregate_id = test_aggregate_id();
    let mut state = ComputeResourceState::default_for(aggregate_id);
    let mut events = Vec::new();

    // Step 1: Register resource
    let register_cmd = RegisterResourceCommand {
        hostname: Hostname::new("server01.example.com").unwrap(),
        resource_type: ResourceType::PhysicalServer,
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
    };

    let register_event = handle_register_resource(&state, register_cmd, aggregate_id)
        .expect("Failed to register resource");
    let register_event_id = register_event.event_id;
    events.push(ComputeResourceEvent::ResourceRegistered(register_event.clone()));
    state = apply_event(state, &ComputeResourceEvent::ResourceRegistered(register_event));

    // Verify state after registration
    assert!(state.is_initialized());
    assert_eq!(state.hostname.as_str(), "server01.example.com");
    assert_eq!(state.status, ResourceStatus::Provisioning);

    // Step 2: Assign organization
    let org_id = EntityId::new();
    let org_cmd = AssignOrganizationCommand {
        organization_id: org_id.clone(),
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(register_event_id),
    };

    let org_event = handle_assign_organization(&state, org_cmd)
        .expect("Failed to assign organization");
    let org_event_id = org_event.event_id;
    events.push(ComputeResourceEvent::OrganizationAssigned(org_event.clone()));
    state = apply_event(state, &ComputeResourceEvent::OrganizationAssigned(org_event));

    // Verify organization assigned
    assert_eq!(state.organization_id, Some(org_id.clone()));

    // Step 3: Add policies
    let policy1_id = PolicyId::new();
    let policy1_cmd = AddPolicyCommand {
        policy_id: policy1_id.clone(),
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(org_event_id),
    };

    let policy1_event = handle_add_policy(&state, policy1_cmd)
        .expect("Failed to add policy");
    let policy1_event_id = policy1_event.event_id;
    events.push(ComputeResourceEvent::PolicyAdded(policy1_event.clone()));
    state = apply_event(state, &ComputeResourceEvent::PolicyAdded(policy1_event));

    // Verify policy added
    assert_eq!(state.policy_ids.len(), 1);
    assert!(state.policy_ids.contains(&policy1_id));

    // Step 4: Change status to Active
    let status_cmd = ChangeStatusCommand {
        to_status: ResourceStatus::Active,
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(policy1_event_id),
    };

    let status_event = handle_change_status(&state, status_cmd)
        .expect("Failed to change status");
    events.push(ComputeResourceEvent::StatusChanged(status_event.clone()));
    state = apply_event(state, &ComputeResourceEvent::StatusChanged(status_event));

    // Verify status changed
    assert_eq!(state.status, ResourceStatus::Active);

    // Step 5: Reconstruct state from events
    let reconstructed_state = ComputeResourceState::from_events(&events);

    // Verify reconstructed state matches current state
    assert_eq!(reconstructed_state.id, state.id);
    assert_eq!(reconstructed_state.hostname, state.hostname);
    assert_eq!(reconstructed_state.organization_id, state.organization_id);
    assert_eq!(reconstructed_state.policy_ids, state.policy_ids);
    assert_eq!(reconstructed_state.status, state.status);
    assert_eq!(reconstructed_state, state);
}

/// Test: Command validation prevents invalid operations
#[test]
fn test_command_validation() {
    let aggregate_id = test_aggregate_id();
    let state = ComputeResourceState::default_for(aggregate_id);

    // Attempt to assign organization before registration
    let org_cmd = AssignOrganizationCommand {
        organization_id: EntityId::new(),
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    };

    let result = handle_assign_organization(&state, org_cmd);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CommandError::NotInitialized);
}

/// Test: Cannot register resource twice
#[test]
fn test_cannot_register_twice() {
    let aggregate_id = test_aggregate_id();
    let mut state = ComputeResourceState::default_for(aggregate_id);

    // First registration
    let register_cmd1 = RegisterResourceCommand {
        hostname: Hostname::new("server01.example.com").unwrap(),
        resource_type: ResourceType::PhysicalServer,
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
    };

    let event1 = handle_register_resource(&state, register_cmd1, aggregate_id)
        .expect("First registration should succeed");
    state = apply_event(state, &ComputeResourceEvent::ResourceRegistered(event1));

    // Second registration attempt
    let register_cmd2 = RegisterResourceCommand {
        hostname: Hostname::new("server02.example.com").unwrap(),
        resource_type: ResourceType::VirtualMachine,
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
    };

    let result = handle_register_resource(&state, register_cmd2, aggregate_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CommandError::AlreadyInitialized);
}

/// Test: Cannot add same policy twice
#[test]
fn test_cannot_add_policy_twice() {
    let aggregate_id = test_aggregate_id();
    let mut state = ComputeResourceState::default_for(aggregate_id);

    // Register resource
    let register_cmd = RegisterResourceCommand {
        hostname: Hostname::new("server01.example.com").unwrap(),
        resource_type: ResourceType::PhysicalServer,
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
    };

    let register_event = handle_register_resource(&state, register_cmd, aggregate_id).unwrap();
    state = apply_event(state, &ComputeResourceEvent::ResourceRegistered(register_event));

    // Add policy first time
    let policy_id = PolicyId::new();
    let add_cmd1 = AddPolicyCommand {
        policy_id: policy_id.clone(),
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    };

    let policy_event = handle_add_policy(&state, add_cmd1).unwrap();
    state = apply_event(state, &ComputeResourceEvent::PolicyAdded(policy_event));

    // Try to add same policy again
    let add_cmd2 = AddPolicyCommand {
        policy_id: policy_id.clone(),
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    };

    let result = handle_add_policy(&state, add_cmd2);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CommandError::PolicyAlreadyAdded(_)
    ));
}

/// Test: Invalid status transition is rejected
#[test]
fn test_invalid_status_transition() {
    let aggregate_id = test_aggregate_id();
    let mut state = ComputeResourceState::default_for(aggregate_id);

    // Register and activate resource
    let register_cmd = RegisterResourceCommand {
        hostname: Hostname::new("server01.example.com").unwrap(),
        resource_type: ResourceType::PhysicalServer,
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
    };

    let register_event = handle_register_resource(&state, register_cmd, aggregate_id).unwrap();
    state = apply_event(state, &ComputeResourceEvent::ResourceRegistered(register_event));

    // Change to Active
    let activate_cmd = ChangeStatusCommand {
        to_status: ResourceStatus::Active,
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    };

    let activate_event = handle_change_status(&state, activate_cmd).unwrap();
    state = apply_event(state, &ComputeResourceEvent::StatusChanged(activate_event));

    // Try to go back to Provisioning (invalid)
    let invalid_cmd = ChangeStatusCommand {
        to_status: ResourceStatus::Provisioning,
        timestamp: test_timestamp(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    };

    let result = handle_change_status(&state, invalid_cmd);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CommandError::InvalidStatusTransition { .. }
    ));
}

/// Test: State reconstruction from event stream
#[test]
fn test_state_reconstruction_from_events() {
    let aggregate_id = test_aggregate_id();

    // Create event stream
    let correlation_id = Uuid::now_v7();
    let events = vec![
        ComputeResourceEvent::ResourceRegistered(cim_infrastructure::events::ResourceRegistered {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id,
            timestamp: test_timestamp(),
            correlation_id,
            causation_id: None,
            hostname: Hostname::new("server01.example.com").unwrap(),
            resource_type: ResourceType::PhysicalServer,
        }),
        ComputeResourceEvent::OrganizationAssigned(cim_infrastructure::events::OrganizationAssigned {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id,
            timestamp: test_timestamp(),
            correlation_id,
            causation_id: None,
            organization_id: EntityId::new(),
        }),
        ComputeResourceEvent::StatusChanged(cim_infrastructure::events::StatusChanged {
            event_version: 1,
            event_id: Uuid::now_v7(),
            aggregate_id,
            timestamp: test_timestamp(),
            correlation_id,
            causation_id: None,
            from_status: ResourceStatus::Provisioning,
            to_status: ResourceStatus::Active,
        }),
    ];

    // Reconstruct state
    let state = ComputeResourceState::from_events(&events);

    // Verify final state
    assert_eq!(state.hostname.as_str(), "server01.example.com");
    assert!(state.organization_id.is_some());
    assert_eq!(state.status, ResourceStatus::Active);
    assert!(state.is_initialized());
}

/// Test: Empty event stream produces default state
#[test]
fn test_empty_event_stream() {
    let events: Vec<ComputeResourceEvent> = vec![];
    let state = ComputeResourceState::from_events(&events);

    assert!(!state.is_initialized());
    assert_eq!(state.status, ResourceStatus::Provisioning);
    assert!(state.organization_id.is_none());
}
