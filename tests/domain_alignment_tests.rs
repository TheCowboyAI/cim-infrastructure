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

// =========================================================================
// VitalConcept Projection Tests (cim-domain-spaces v0.9.7+ integration)
// =========================================================================

#[test]
fn test_compute_resource_to_vital_concept() -> Result<()> {
    let hostname = Hostname::new("web-server01")?;
    let resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Project to VitalConcept
    let concept = resource.to_vital_concept();

    // Verify concept properties
    assert_eq!(concept.name, "web-server01");
    assert!(concept.description.contains("Physical Server"));
    assert_eq!(concept.position.len(), 5); // 5-dimensional space

    // Verify dimensions are valid (0.0-1.0 range)
    for &value in &concept.position {
        assert!(value >= 0.0 && value <= 1.0, "Dimension value {} out of range", value);
    }

    Ok(())
}

#[test]
fn test_vital_concept_with_organization() -> Result<()> {
    let hostname = Hostname::new("db-server01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    let org_id = EntityId::<Organization>::new();
    resource.set_organization(org_id.clone());

    // Project to VitalConcept
    let concept = resource.to_vital_concept();

    // Description should include organization reference
    assert!(concept.description.contains("Organization"));
    assert!(concept.description.contains(&org_id.to_string()));

    Ok(())
}

#[test]
fn test_vital_concept_dimensions_by_resource_type() -> Result<()> {
    // Test different resource types have different dimensional positions

    // Physical server - high scale (0.9)
    let physical_server = ComputeResource::new(
        Hostname::new("physical01")?,
        ResourceType::PhysicalServer,
    )?;
    let physical_concept = physical_server.to_vital_concept();
    let physical_scale = physical_concept.position[0]; // First dimension is scale
    assert!(physical_scale > 0.8, "Physical server should have high scale");

    // Virtual machine - medium scale (0.5)
    let vm = ComputeResource::new(
        Hostname::new("vm01")?,
        ResourceType::VirtualMachine,
    )?;
    let vm_concept = vm.to_vital_concept();
    let vm_scale = vm_concept.position[0];
    assert!(vm_scale < physical_scale, "VM should have lower scale than physical");
    assert!(vm_scale > 0.4 && vm_scale < 0.6, "VM scale should be around 0.5");

    // Container host - higher than VM (0.7)
    let container = ComputeResource::new(
        Hostname::new("container01")?,
        ResourceType::ContainerHost,
    )?;
    let container_concept = container.to_vital_concept();
    let container_scale = container_concept.position[0];
    assert!(container_scale > vm_scale, "Container host should have higher scale than VM");
    assert!(container_scale > 0.6 && container_scale < 0.8, "Container scale should be around 0.7");

    Ok(())
}

#[test]
fn test_vital_concept_complexity_dimension() -> Result<()> {
    let hostname = Hostname::new("complex-server")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Simple resource - low complexity
    let simple_concept = resource.to_vital_concept();
    let simple_complexity = simple_concept.position[1]; // Second dimension is complexity

    // Add metadata and policies to increase complexity
    resource.add_metadata("role", "database")?;
    resource.add_metadata("environment", "production")?;
    resource.add_metadata("tier", "critical")?;
    resource.add_policy(PolicyId::new());
    resource.add_policy(PolicyId::new());
    resource.add_policy(PolicyId::new());

    let complex_concept = resource.to_vital_concept();
    let complex_complexity = complex_concept.position[1];

    // Complexity should increase with metadata and policies
    assert!(complex_complexity > simple_complexity, "Complexity should increase with metadata and policies");

    Ok(())
}

#[test]
fn test_vital_concept_reliability_dimension() -> Result<()> {
    let hostname = Hostname::new("reliable-server")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Base reliability
    let base_concept = resource.to_vital_concept();
    let base_reliability = base_concept.position[2]; // Third dimension is reliability

    // Add location, organization, owner, policies
    resource.set_location(EntityId::<LocationMarker>::new());
    resource.set_organization(EntityId::<Organization>::new());
    resource.set_owner(PersonId::new());
    resource.add_policy(PolicyId::new());

    let enhanced_concept = resource.to_vital_concept();
    let enhanced_reliability = enhanced_concept.position[2];

    // Reliability should increase with proper governance
    assert!(enhanced_reliability > base_reliability, "Reliability should increase with location, org, owner, and policies");

    Ok(())
}

#[test]
fn test_vital_concept_serialization() -> Result<()> {
    let hostname = Hostname::new("api-server01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    resource.set_organization(EntityId::<Organization>::new());
    resource.add_metadata("role", "api")?;

    // Project to VitalConcept
    let concept = resource.to_vital_concept();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&concept)?;
    println!("VitalConcept JSON:\n{}", json);

    // Deserialize back
    let deserialized: cim_domain_spaces::base_concepts::VitalConcept = serde_json::from_str(&json)?;

    // Verify core properties
    assert_eq!(deserialized.name, concept.name);
    assert_eq!(deserialized.description, concept.description);

    // Verify positions are approximately equal (floating point precision)
    assert_eq!(deserialized.position.len(), concept.position.len());
    for (i, (&expected, &actual)) in concept.position.iter().zip(&deserialized.position).enumerate() {
        assert!(
            (expected - actual).abs() < 0.0001,
            "Position dimension {} differs: expected {}, got {}",
            i,
            expected,
            actual
        );
    }

    Ok(())
}

#[test]
fn test_vital_concept_complete_integration() -> Result<()> {
    // Create a fully-configured resource
    let hostname = Hostname::new("prod-db-primary")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Full domain integration
    let org_id = EntityId::<Organization>::new();
    let location_id = EntityId::<LocationMarker>::new();
    let owner_id = PersonId::new();
    let policy1 = PolicyId::new();
    let policy2 = PolicyId::new();
    let account_concept = ConceptId::new();

    resource.set_organization(org_id.clone());
    resource.set_location(location_id.clone());
    resource.set_owner(owner_id.clone());
    resource.add_policy(policy1);
    resource.add_policy(policy2);
    resource.set_account_concept(account_concept)?;

    resource.add_metadata("role", "primary_database")?;
    resource.add_metadata("environment", "production")?;
    resource.add_metadata("tier", "critical")?;
    resource.add_metadata("backup_frequency", "hourly")?;

    resource.set_hardware(
        Some("Dell".into()),
        Some("PowerEdge R740".into()),
        Some("SN-DB-001".into()),
    );

    // Project to VitalConcept
    let concept = resource.to_vital_concept();

    // Verify rich conceptual representation
    assert_eq!(concept.name, "prod-db-primary");
    assert!(concept.description.contains("Physical Server"));
    assert!(concept.description.contains(&org_id.to_string()));

    // Verify all 5 dimensions are present and valid
    assert_eq!(concept.position.len(), 5);
    for (idx, &value) in concept.position.iter().enumerate() {
        assert!(
            value >= 0.0 && value <= 1.0,
            "Dimension {} value {} out of range",
            idx,
            value
        );
    }

    // High reliability due to full governance
    let reliability = concept.position[2];
    assert!(reliability > 0.9, "Fully configured resource should have high reliability");

    // High complexity due to rich metadata
    let complexity = concept.position[1];
    assert!(complexity > 0.6, "Rich metadata should result in higher complexity");

    println!("Complete VitalConcept:");
    println!("  Name: {}", concept.name);
    println!("  Description: {}", concept.description);
    println!("  Dimensions: {:?}", concept.position);
    println!("    [0] Scale: {:.2}", concept.position[0]);
    println!("    [1] Complexity: {:.2}", concept.position[1]);
    println!("    [2] Reliability: {:.2}", concept.position[2]);
    println!("    [3] Performance: {:.2}", concept.position[3]);
    println!("    [4] Cost Efficiency: {:.2}", concept.position[4]);

    Ok(())
}
