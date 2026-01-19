// Copyright (c) 2025 - Cowboy AI, Inc.
//! Domain Alignment Tests
//!
//! Verifies that cim-infrastructure properly integrates with:
//! - cim-domain-organization (Domain Aggregates)
//! - cim-domain-person (Domain Aggregates)
//! - cim-domain-location (Domain Aggregates)
//! - cim-domain-policy (Domain Aggregates)
//! - cim-domain-spaces (Conceptual Space - Account Concepts)

use anyhow::Result;
use cim_domain::EntityId;
use cim_domain_location::LocationMarker;
use cim_domain_organization::Organization;
use cim_domain_person::PersonId;
use cim_domain_policy::PolicyId;
use cim_domain_spaces::ConceptId;
use cim_infrastructure::{ComputeResource, Hostname, ResourceType};

#[test]
fn test_organization_person_location_alignment() -> Result<()> {
    // Create resource
    let hostname = Hostname::new("test-server01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Add organization
    let org_id = EntityId::<Organization>::new();
    resource.set_organization(org_id.clone());
    assert_eq!(resource.organization_id, Some(org_id));
    assert!(resource.is_multi_tenant());

    // Add location
    let location_id = EntityId::<LocationMarker>::new();
    resource.set_location(location_id.clone());
    assert_eq!(resource.location_id, Some(location_id));
    assert!(resource.is_physical());

    // Add owner
    let owner_id = PersonId::new();
    resource.set_owner(owner_id.clone());
    assert_eq!(resource.owner_id, Some(owner_id));

    Ok(())
}

#[test]
fn test_policy_alignment() -> Result<()> {
    let hostname = Hostname::new("secure-server01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Add security policy
    let security_policy = PolicyId::new();
    resource.add_policy(security_policy);
    assert!(resource.has_policy(&security_policy));

    // Add compliance policy
    let compliance_policy = PolicyId::new();
    resource.add_policy(compliance_policy);
    assert_eq!(resource.get_applicable_policies().len(), 2);

    // Remove policy
    assert!(resource.remove_policy(&security_policy));
    assert!(!resource.has_policy(&security_policy));
    assert_eq!(resource.get_applicable_policies().len(), 1);

    // Idempotent add
    resource.add_policy(compliance_policy);
    assert_eq!(resource.get_applicable_policies().len(), 1);

    Ok(())
}

#[test]
fn test_policy_idempotent_addition() -> Result<()> {
    let hostname = Hostname::new("policy-test-server")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    let policy_id = PolicyId::new();

    // Add same policy multiple times
    resource.add_policy(policy_id);
    resource.add_policy(policy_id);
    resource.add_policy(policy_id);

    // Should only be added once (idempotent)
    assert_eq!(resource.get_applicable_policies().len(), 1);
    assert_eq!(resource.get_applicable_policies()[0], policy_id);

    Ok(())
}

#[test]
fn test_multiple_policies() -> Result<()> {
    let hostname = Hostname::new("multi-policy-server")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Add multiple distinct policies
    let security_policy = PolicyId::new();
    let compliance_policy = PolicyId::new();
    let operational_policy = PolicyId::new();
    let backup_policy = PolicyId::new();

    resource.add_policy(security_policy);
    resource.add_policy(compliance_policy);
    resource.add_policy(operational_policy);
    resource.add_policy(backup_policy);

    assert_eq!(resource.get_applicable_policies().len(), 4);
    assert!(resource.has_policy(&security_policy));
    assert!(resource.has_policy(&compliance_policy));
    assert!(resource.has_policy(&operational_policy));
    assert!(resource.has_policy(&backup_policy));

    // Remove one policy
    assert!(resource.remove_policy(&compliance_policy));
    assert_eq!(resource.get_applicable_policies().len(), 3);
    assert!(!resource.has_policy(&compliance_policy));

    Ok(())
}

#[test]
fn test_remove_nonexistent_policy() -> Result<()> {
    let hostname = Hostname::new("remove-test-server")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    let policy_id = PolicyId::new();
    let nonexistent_policy = PolicyId::new();

    resource.add_policy(policy_id);

    // Try to remove policy that doesn't exist
    assert!(!resource.remove_policy(&nonexistent_policy));
    assert_eq!(resource.get_applicable_policies().len(), 1);

    // Remove existing policy
    assert!(resource.remove_policy(&policy_id));
    assert_eq!(resource.get_applicable_policies().len(), 0);

    Ok(())
}

#[test]
fn test_account_concept_alignment() -> Result<()> {
    let hostname = Hostname::new("concept-managed-server")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Initially no account concept
    assert!(!resource.is_concept_managed());
    assert_eq!(resource.get_account_concept(), None);

    // Associate with account concept
    let account_concept_id = ConceptId::new();
    resource.set_account_concept(account_concept_id)?;

    assert!(resource.is_concept_managed());
    assert_eq!(resource.get_account_concept(), Some(account_concept_id));

    // Clear concept association
    resource.clear_account_concept();
    assert!(!resource.is_concept_managed());
    assert_eq!(resource.get_account_concept(), None);

    Ok(())
}

#[test]
fn test_account_concept_replacement() -> Result<()> {
    let hostname = Hostname::new("concept-replacement-server")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Set initial account concept
    let concept1 = ConceptId::new();
    resource.set_account_concept(concept1)?;
    assert_eq!(resource.get_account_concept(), Some(concept1));

    // Replace with new account concept
    let concept2 = ConceptId::new();
    resource.set_account_concept(concept2)?;
    assert_eq!(resource.get_account_concept(), Some(concept2));
    assert_ne!(resource.get_account_concept(), Some(concept1));

    Ok(())
}

#[test]
fn test_full_domain_integration() -> Result<()> {
    // Create fully-integrated resource
    let hostname = Hostname::new("production-server01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Organization + Location + Owner (Domain Aggregates)
    let org_id = EntityId::<Organization>::new();
    let location_id = EntityId::<LocationMarker>::new();
    let owner_id = PersonId::new();

    resource.set_organization(org_id.clone());
    resource.set_location(location_id.clone());
    resource.set_owner(owner_id.clone());

    // Account Concept (Conceptual Space)
    let account_concept_id = ConceptId::new();
    resource.set_account_concept(account_concept_id)?;

    // Policies (Domain Aggregates)
    let security_policy = PolicyId::new();
    let compliance_policy = PolicyId::new();
    let operational_policy = PolicyId::new();

    resource.add_policy(security_policy);
    resource.add_policy(compliance_policy);
    resource.add_policy(operational_policy);

    // Metadata
    resource.add_metadata("rack", "dc1_rack42")?;
    resource.add_metadata("role", "database_server")?;

    // Hardware
    resource.set_hardware(
        Some("Dell".to_string()),
        Some("PowerEdge R740".to_string()),
        Some("SN123456".to_string()),
    );

    // Verify all relationships
    assert!(resource.is_multi_tenant()); // Has organization
    assert!(resource.is_physical()); // Has location
    assert!(resource.is_concept_managed()); // Has account concept
    assert_eq!(resource.get_applicable_policies().len(), 3);
    assert_eq!(resource.metadata.len(), 2);
    assert!(resource.manufacturer.is_some());

    // Verify specific IDs
    assert_eq!(resource.organization_id, Some(org_id));
    assert_eq!(resource.location_id, Some(location_id));
    assert_eq!(resource.owner_id, Some(owner_id));
    assert_eq!(resource.get_account_concept(), Some(account_concept_id));
    assert!(resource.has_policy(&security_policy));
    assert!(resource.has_policy(&compliance_policy));
    assert!(resource.has_policy(&operational_policy));

    Ok(())
}

#[test]
fn test_builder_with_policies_and_concept() -> Result<()> {
    let hostname = Hostname::new("builder-test-server")?;
    let org_id = EntityId::<Organization>::new();
    let location_id = EntityId::<LocationMarker>::new();
    let owner_id = PersonId::new();
    let policy_id = PolicyId::new();
    let concept_id = ConceptId::new();

    let resource = ComputeResource::builder(hostname, ResourceType::PhysicalServer)?
        .organization(org_id.clone())
        .location(location_id.clone())
        .owner(owner_id.clone())
        .policy(policy_id)
        .account_concept(concept_id)
        .hardware("HP", "ProLiant DL380")
        .serial_number("HPE-12345")
        .metadata("environment", "production")?
        .build()?;

    // Verify all integrations
    assert_eq!(resource.organization_id, Some(org_id));
    assert_eq!(resource.location_id, Some(location_id));
    assert_eq!(resource.owner_id, Some(owner_id));
    assert!(resource.has_policy(&policy_id));
    assert_eq!(resource.get_account_concept(), Some(concept_id));
    assert_eq!(resource.manufacturer, Some("HP".to_string()));
    assert_eq!(resource.metadata.get("environment"), Some(&"production".to_string()));

    Ok(())
}

#[test]
fn test_builder_multiple_policies() -> Result<()> {
    let hostname = Hostname::new("multi-policy-builder")?;
    let policy1 = PolicyId::new();
    let policy2 = PolicyId::new();
    let policy3 = PolicyId::new();

    let resource = ComputeResource::builder(hostname, ResourceType::PhysicalServer)?
        .policy(policy1)
        .policy(policy2)
        .policy(policy3)
        .build()?;

    assert_eq!(resource.get_applicable_policies().len(), 3);
    assert!(resource.has_policy(&policy1));
    assert!(resource.has_policy(&policy2));
    assert!(resource.has_policy(&policy3));

    Ok(())
}

#[test]
fn test_domain_aggregate_vs_concept_distinction() -> Result<()> {
    let hostname = Hostname::new("distinction-test")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Domain Aggregates (referenced by typed IDs)
    let org_id = EntityId::<Organization>::new();
    let person_id = PersonId::new();
    let location_id = EntityId::<LocationMarker>::new();
    let policy_id = PolicyId::new();

    resource.set_organization(org_id.clone());
    resource.set_owner(person_id.clone());
    resource.set_location(location_id.clone());
    resource.add_policy(policy_id);

    // Concept (referenced by ConceptId)
    let account_concept = ConceptId::new();
    resource.set_account_concept(account_concept)?;

    // Verify types are distinct
    assert_eq!(resource.organization_id, Some(org_id));
    assert_eq!(resource.owner_id, Some(person_id));
    assert_eq!(resource.location_id, Some(location_id));
    assert!(resource.has_policy(&policy_id));
    assert_eq!(resource.get_account_concept(), Some(account_concept));

    // Account concept is NOT an AggregateId
    // It represents a Concept in conceptual space with:
    // - Geometric position
    // - ConceptRelationships to other concepts
    // - Properties (account state, type, tier)
    // - Knowledge level and evidence

    Ok(())
}

#[test]
fn test_serialization_with_all_fields() -> Result<()> {
    let hostname = Hostname::new("serialize-test")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::Router)?;

    resource.set_organization(EntityId::<Organization>::new());
    resource.set_location(EntityId::<LocationMarker>::new());
    resource.set_owner(PersonId::new());
    resource.add_policy(PolicyId::new());
    resource.add_policy(PolicyId::new());
    resource.set_account_concept(ConceptId::new())?;
    resource.add_metadata("zone", "dmz")?;
    resource.set_hardware(Some("Cisco".into()), Some("ASR 9000".into()), Some("CSC-123".into()));

    // Serialize to JSON
    let json = serde_json::to_string(&resource)?;

    // Deserialize back
    let deserialized: ComputeResource = serde_json::from_str(&json)?;

    // Verify all fields
    assert_eq!(deserialized.organization_id, resource.organization_id);
    assert_eq!(deserialized.location_id, resource.location_id);
    assert_eq!(deserialized.owner_id, resource.owner_id);
    assert_eq!(deserialized.policy_ids, resource.policy_ids);
    assert_eq!(deserialized.account_concept_id, resource.account_concept_id);
    assert_eq!(deserialized.metadata, resource.metadata);
    assert_eq!(deserialized.manufacturer, resource.manufacturer);

    Ok(())
}
