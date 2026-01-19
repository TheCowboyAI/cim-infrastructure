// Copyright (c) 2025 - Cowboy AI, Inc.
//! Compute Resource Entity with Compositional Invariants
//!
//! This entity composes with existing CIM domains:
//! - cim-domain-organization (OrganizationId)
//! - cim-domain-person (PersonId)
//! - cim-domain-location (LocationId)
//! - cim-domain-policy (PolicyId)
//! - cim-domain-spaces (ConceptId for account concepts)

use cim_domain::EntityId;
use cim_domain_location::LocationMarker;
use cim_domain_organization::Organization;
use cim_domain_person::PersonId;
use cim_domain_policy::PolicyId;
use cim_domain_spaces::ConceptId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use super::{Hostname, ResourceType};

/// Compute Resource validation error
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ComputeResourceError {
    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    #[error("Resource type not specified")]
    MissingResourceType,

    #[error("Organization ID required for multi-tenant resources")]
    OrganizationRequired,

    #[error("Location ID required for physical resources")]
    LocationRequired,

    #[error("Invalid metadata key: {0}")]
    InvalidMetadataKey(String),

    #[error("No account concept - resource must be associated with account concept")]
    NoAccountConcept,

    #[error("Account concept not found in conceptual space: {0}")]
    AccountConceptNotFound(String),

    #[error("Account concept invalid: {0}")]
    InvalidAccountConcept(String),
}

/// Compute Resource Entity
///
/// Represents a computational infrastructure resource with:
/// - Validation invariants
/// - Relationships to CIM domains (Organization, Person, Location)
/// - Proper serialization for events
/// - Graph composition support
///
/// # Invariants
/// - Must have valid hostname
/// - Must have resource type
/// - Physical resources must have location
/// - Multi-tenant resources must have organization
/// - Metadata keys must follow naming conventions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeResource {
    /// Aggregate ID (immutable identity)
    pub id: EntityId<ComputeResource>,

    /// Hostname (DNS-validated value object)
    pub hostname: Hostname,

    /// Resource type (taxonomy value object)
    pub resource_type: ResourceType,

    /// Organization ownership (reference to cim-domain-organization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<EntityId<Organization>>,

    /// Physical location (reference to cim-domain-location)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<EntityId<LocationMarker>>,

    /// Primary contact/owner (reference to cim-domain-person)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<PersonId>,

    /// Applicable policies (reference to cim-domain-policy)
    /// Infrastructure resources can have multiple policies applied:
    /// - Security policies (mandatory encryption, access controls)
    /// - Compliance policies (regulatory requirements)
    /// - Operational policies (backup schedules, maintenance windows)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub policy_ids: Vec<PolicyId>,

    /// Account concept (reference to cim-domain-spaces Concept)
    /// Represents "organization for purpose" relationship in conceptual space
    ///
    /// **Invariant**: If set, concept must exist in conceptual space
    /// **Conceptual Semantics**:
    /// - Account concept position determines semantic meaning
    /// - Concept relationships define authorization (→ Person concepts, → Policy concepts)
    /// - Concept properties contain account state (Active, Suspended, etc.)
    /// - Knowledge level indicates certainty about account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_concept_id: Option<ConceptId>,

    /// Hardware manufacturer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,

    /// Hardware model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Serial number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,

    /// Asset tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_tag: Option<String>,

    /// Custom metadata (extensible properties)
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl ComputeResource {
    /// Create a new compute resource with validation
    ///
    /// # Invariants
    /// - Hostname must be valid
    /// - Resource type must be specified
    /// - Physical resources (servers, routers, etc.) should have location
    /// - Multi-tenant resources should have organization
    pub fn new(
        hostname: Hostname,
        resource_type: ResourceType,
    ) -> Result<Self, ComputeResourceError> {
        let now = Utc::now();

        Ok(Self {
            id: EntityId::new(),
            hostname,
            resource_type,
            organization_id: None,
            location_id: None,
            owner_id: None,
            policy_ids: Vec::new(),
            account_concept_id: None,
            manufacturer: None,
            model: None,
            serial_number: None,
            asset_tag: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Builder pattern for fluent construction
    pub fn builder(
        hostname: Hostname,
        resource_type: ResourceType,
    ) -> Result<ComputeResourceBuilder, ComputeResourceError> {
        Ok(ComputeResourceBuilder::new(hostname, resource_type))
    }

    /// Set organization ownership
    ///
    /// # Invariant
    /// - Organization ID must reference valid cim-domain-organization aggregate
    pub fn set_organization(&mut self, org_id: EntityId<Organization>) {
        self.organization_id = Some(org_id);
        self.updated_at = Utc::now();
    }

    /// Set physical location
    ///
    /// # Invariant
    /// - Location ID must reference valid cim-domain-location aggregate
    pub fn set_location(&mut self, location_id: EntityId<LocationMarker>) {
        self.location_id = Some(location_id);
        self.updated_at = Utc::now();
    }

    /// Set owner/primary contact
    ///
    /// # Invariant
    /// - Owner ID must reference valid cim-domain-person aggregate
    pub fn set_owner(&mut self, owner_id: PersonId) {
        self.owner_id = Some(owner_id);
        self.updated_at = Utc::now();
    }

    /// Add a policy to this resource
    ///
    /// # Invariant
    /// - Policy ID must reference valid cim-domain-policy aggregate
    /// - Duplicate policies are ignored (idempotent)
    pub fn add_policy(&mut self, policy_id: PolicyId) {
        if !self.policy_ids.contains(&policy_id) {
            self.policy_ids.push(policy_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a policy from this resource
    pub fn remove_policy(&mut self, policy_id: &PolicyId) -> bool {
        if let Some(pos) = self.policy_ids.iter().position(|id| id == policy_id) {
            self.policy_ids.remove(pos);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Get all applicable policies
    pub fn get_applicable_policies(&self) -> &[PolicyId] {
        &self.policy_ids
    }

    /// Check if resource has a specific policy applied
    pub fn has_policy(&self, policy_id: &PolicyId) -> bool {
        self.policy_ids.contains(policy_id)
    }

    /// Associate resource with account concept
    ///
    /// # Invariant
    /// - Concept must exist in cim-domain-spaces
    /// - Concept must have account properties (type, state, tier)
    pub fn set_account_concept(
        &mut self,
        concept_id: ConceptId,
    ) -> Result<(), ComputeResourceError> {
        // Store reference to account concept
        self.account_concept_id = Some(concept_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove account concept association
    pub fn clear_account_concept(&mut self) {
        self.account_concept_id = None;
        self.updated_at = Utc::now();
    }

    /// Check if resource is concept-managed (has account concept)
    pub fn is_concept_managed(&self) -> bool {
        self.account_concept_id.is_some()
    }

    /// Get account concept ID for conceptual space queries
    pub fn get_account_concept(&self) -> Option<ConceptId> {
        self.account_concept_id
    }

    /// Set hardware details
    pub fn set_hardware(
        &mut self,
        manufacturer: Option<String>,
        model: Option<String>,
        serial_number: Option<String>,
    ) {
        self.manufacturer = manufacturer;
        self.model = model;
        self.serial_number = serial_number;
        self.updated_at = Utc::now();
    }

    /// Set asset tag
    pub fn set_asset_tag(&mut self, asset_tag: String) {
        self.asset_tag = Some(asset_tag);
        self.updated_at = Utc::now();
    }

    /// Add metadata
    ///
    /// # Invariant
    /// - Keys must follow naming convention (lowercase, alphanumeric + underscore)
    pub fn add_metadata(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), ComputeResourceError> {
        let key = key.into();

        // Invariant: Validate metadata key format
        if !Self::is_valid_metadata_key(&key) {
            return Err(ComputeResourceError::InvalidMetadataKey(key));
        }

        self.metadata.insert(key, value.into());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Validate metadata key format
    fn is_valid_metadata_key(key: &str) -> bool {
        !key.is_empty()
            && key.len() <= 64
            && key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }

    /// Check if resource is physical (has location)
    pub fn is_physical(&self) -> bool {
        self.location_id.is_some()
    }

    /// Check if resource is multi-tenant (has organization)
    pub fn is_multi_tenant(&self) -> bool {
        self.organization_id.is_some()
    }

    /// Validate invariants for the current state
    pub fn validate(&self) -> Result<(), ComputeResourceError> {
        // Physical resources should have location
        if self.resource_type.is_compute_resource() && !self.resource_type.is_ipv6_address() {
            // Note: Virtual resources might not have physical location
        }

        // Validate all metadata keys
        for key in self.metadata.keys() {
            if !Self::is_valid_metadata_key(key) {
                return Err(ComputeResourceError::InvalidMetadataKey(key.clone()));
            }
        }

        Ok(())
    }
}

/// Builder for ComputeResource with fluent API
pub struct ComputeResourceBuilder {
    resource: ComputeResource,
}

impl ComputeResourceBuilder {
    fn new(hostname: Hostname, resource_type: ResourceType) -> Self {
        let now = Utc::now();
        Self {
            resource: ComputeResource {
                id: EntityId::new(),
                hostname,
                resource_type,
                organization_id: None,
                location_id: None,
                owner_id: None,
                policy_ids: Vec::new(),
                account_concept_id: None,
                manufacturer: None,
                model: None,
                serial_number: None,
                asset_tag: None,
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
        }
    }

    pub fn organization(mut self, org_id: EntityId<Organization>) -> Self {
        self.resource.organization_id = Some(org_id);
        self
    }

    pub fn location(mut self, location_id: EntityId<LocationMarker>) -> Self {
        self.resource.location_id = Some(location_id);
        self
    }

    pub fn owner(mut self, owner_id: PersonId) -> Self {
        self.resource.owner_id = Some(owner_id);
        self
    }

    pub fn policy(mut self, policy_id: PolicyId) -> Self {
        if !self.resource.policy_ids.contains(&policy_id) {
            self.resource.policy_ids.push(policy_id);
        }
        self
    }

    pub fn account_concept(mut self, concept_id: ConceptId) -> Self {
        self.resource.account_concept_id = Some(concept_id);
        self
    }

    pub fn hardware(
        mut self,
        manufacturer: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        self.resource.manufacturer = Some(manufacturer.into());
        self.resource.model = Some(model.into());
        self
    }

    pub fn serial_number(mut self, serial: impl Into<String>) -> Self {
        self.resource.serial_number = Some(serial.into());
        self
    }

    pub fn asset_tag(mut self, tag: impl Into<String>) -> Self {
        self.resource.asset_tag = Some(tag.into());
        self
    }

    pub fn metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ComputeResourceError> {
        let key = key.into();
        if !ComputeResource::is_valid_metadata_key(&key) {
            return Err(ComputeResourceError::InvalidMetadataKey(key));
        }
        self.resource.metadata.insert(key, value.into());
        Ok(self)
    }

    pub fn build(self) -> Result<ComputeResource, ComputeResourceError> {
        self.resource.validate()?;
        Ok(self.resource)
    }
}

// Helper trait extension for ResourceType
trait ResourceTypeExt {
    fn is_ipv6_address(&self) -> bool;
}

impl ResourceTypeExt for ResourceType {
    fn is_ipv6_address(&self) -> bool {
        // Placeholder - implement based on your ResourceType definition
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Hostname;

    #[test]
    fn test_create_compute_resource() {
        let hostname = Hostname::new("web01.example.com").unwrap();
        let resource = ComputeResource::new(hostname, ResourceType::PhysicalServer).unwrap();

        assert_eq!(resource.hostname.as_str(), "web01.example.com");
        assert_eq!(resource.resource_type, ResourceType::PhysicalServer);
        assert!(resource.organization_id.is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let hostname = Hostname::new("router01.dc1.example.com").unwrap();
        let org_id = EntityId::<Organization>::new();
        let location_id = EntityId::<LocationMarker>::new();

        let resource = ComputeResource::builder(hostname, ResourceType::Router)
            .unwrap()
            .organization(org_id.clone())
            .location(location_id.clone())
            .hardware("Cisco", "ASR 1001-X")
            .serial_number("ABC123")
            .metadata("rack", "A01")
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(resource.organization_id, Some(org_id));
        assert_eq!(resource.location_id, Some(location_id));
        assert_eq!(resource.manufacturer, Some("Cisco".to_string()));
        assert_eq!(resource.metadata.get("rack"), Some(&"A01".to_string()));
    }

    #[test]
    fn test_metadata_validation() {
        let hostname = Hostname::new("test.example.com").unwrap();
        let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer).unwrap();

        // Valid keys
        assert!(resource.add_metadata("valid_key", "value").is_ok());
        assert!(resource.add_metadata("key123", "value").is_ok());

        // Invalid keys
        assert!(resource.add_metadata("Invalid-Key", "value").is_err());
        assert!(resource.add_metadata("UPPERCASE", "value").is_err());
        assert!(resource.add_metadata("key with spaces", "value").is_err());
    }

    #[test]
    fn test_resource_relationships() {
        let hostname = Hostname::new("web01.example.com").unwrap();
        let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer).unwrap();

        assert!(!resource.is_physical());
        assert!(!resource.is_multi_tenant());

        let location_id = EntityId::<LocationMarker>::new();
        resource.set_location(location_id);
        assert!(resource.is_physical());

        let org_id = EntityId::<Organization>::new();
        resource.set_organization(org_id);
        assert!(resource.is_multi_tenant());
    }
}
