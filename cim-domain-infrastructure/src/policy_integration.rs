//! CIM Domain Policy Integration for Infrastructure
//!
//! This module demonstrates event-based integration with cim-domain-policy.
//! Infrastructure emits PolicyApplied events, while cim-domain-policy handles
//! evaluation and enforcement.
//!
//! ## Architecture Principle
//!
//! **Infrastructure Domain Responsibility:**
//! - Track which policies are applied to infrastructure resources
//! - Emit PolicyApplied events when policies are associated
//! - Query policy compliance status
//!
//! **Policy Domain Responsibility** (via cim-domain-policy):
//! - Define and manage policy rules
//! - Evaluate compliance against policies
//! - Enforce policy violations
//! - Handle policy exemptions
//!
//! ## Integration Pattern
//!
//! ```rust
//! use cim_domain_infrastructure::policy_integration::PolicyApplicationAdapter;
//! use cim_domain_infrastructure::ResourceId;
//!
//! // Infrastructure emits event about policy application
//! let mut adapter = PolicyApplicationAdapter::new();
//! let resource_id = ResourceId::new("server-01").unwrap();
//!
//! adapter.apply_policy_to_resource(
//!     &resource_id,
//!     "security-policy-123".to_string(),  // References cim-domain-policy::PolicyId
//!     "admin-user".to_string(),
//!     "Enforced security policy on resource".to_string(),
//! );
//!
//! // cim-domain-policy evaluates compliance (separate domain)
//! // This happens asynchronously through NATS event streams
//! ```

#[cfg(feature = "policy")]
use cim_domain_policy::PolicyId as ExternalPolicyId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{ResourceId, NetworkId};

/// Adapter for integrating infrastructure with cim-domain-policy
///
/// This adapter records policy applications in the infrastructure domain
/// and provides a bridge to the policy domain for evaluation.
#[derive(Debug, Clone)]
pub struct PolicyApplicationAdapter {
    /// Policies applied to resources
    resource_policies: HashMap<String, Vec<PolicyApplication>>,
    /// Policies applied to networks
    network_policies: HashMap<String, Vec<PolicyApplication>>,
}

/// Record of a policy being applied to an infrastructure entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyApplication {
    /// Reference to policy in cim-domain-policy
    pub policy_reference: String,
    /// When the policy was applied
    pub applied_at: chrono::DateTime<chrono::Utc>,
    /// Who applied the policy
    pub applied_by: String,
    /// Reason for application
    pub reason: String,
}

impl PolicyApplicationAdapter {
    /// Create a new policy application adapter
    pub fn new() -> Self {
        Self {
            resource_policies: HashMap::new(),
            network_policies: HashMap::new(),
        }
    }

    /// Record a policy being applied to a compute resource
    ///
    /// Note: This only records the application in infrastructure domain.
    /// Actual policy evaluation happens in cim-domain-policy.
    pub fn apply_policy_to_resource(
        &mut self,
        resource_id: &ResourceId,
        policy_reference: String,
        applied_by: String,
        reason: String,
    ) -> PolicyApplication {
        let application = PolicyApplication {
            policy_reference: policy_reference.clone(),
            applied_at: chrono::Utc::now(),
            applied_by,
            reason,
        };

        self.resource_policies
            .entry(resource_id.to_string())
            .or_insert_with(Vec::new)
            .push(application.clone());

        application
    }

    /// Record a policy being applied to a network
    pub fn apply_policy_to_network(
        &mut self,
        network_id: &NetworkId,
        policy_reference: String,
        applied_by: String,
        reason: String,
    ) -> PolicyApplication {
        let application = PolicyApplication {
            policy_reference,
            applied_at: chrono::Utc::now(),
            applied_by,
            reason,
        };

        self.network_policies
            .entry(network_id.to_string())
            .or_insert_with(Vec::new)
            .push(application.clone());

        application
    }

    /// Get all policies applied to a resource
    pub fn get_resource_policies(&self, resource_id: &ResourceId) -> Vec<&PolicyApplication> {
        self.resource_policies
            .get(&resource_id.to_string())
            .map(|apps| apps.iter().collect())
            .unwrap_or_default()
    }

    /// Get all policies applied to a network
    pub fn get_network_policies(&self, network_id: &NetworkId) -> Vec<&PolicyApplication> {
        self.network_policies
            .get(&network_id.to_string())
            .map(|apps| apps.iter().collect())
            .unwrap_or_default()
    }

    /// Get all unique policy references across all infrastructure
    pub fn get_all_policy_references(&self) -> Vec<String> {
        let mut references: Vec<String> = self
            .resource_policies
            .values()
            .chain(self.network_policies.values())
            .flat_map(|apps| apps.iter().map(|app| app.policy_reference.clone()))
            .collect();

        references.sort();
        references.dedup();
        references
    }
}

impl Default for PolicyApplicationAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration point for policy evaluation requests
///
/// Infrastructure can request policy evaluation from cim-domain-policy
/// through NATS event streams.
#[cfg(feature = "policy")]
pub struct PolicyEvaluationRequest {
    /// ID of the policy to evaluate (from cim-domain-policy)
    pub policy_id: ExternalPolicyId,
    /// Context for evaluation (infrastructure state)
    pub context: HashMap<String, serde_json::Value>,
    /// Correlation ID for tracking
    pub correlation_id: Uuid,
}

#[cfg(feature = "policy")]
impl PolicyEvaluationRequest {
    /// Create a new policy evaluation request
    pub fn new(policy_id: ExternalPolicyId, context: HashMap<String, serde_json::Value>) -> Self {
        Self {
            policy_id,
            context,
            correlation_id: Uuid::now_v7(),
        }
    }

    /// Convert to NATS command for cim-domain-policy
    ///
    /// This would be sent to: `policy.commands.evaluate`
    pub fn to_nats_command(&self) -> serde_json::Value {
        serde_json::json!({
            "policy_id": self.policy_id,
            "context": self.context,
            "correlation_id": self.correlation_id,
        })
    }
}

/// Policy compliance status as reported by cim-domain-policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyComplianceStatus {
    /// The policy reference
    pub policy_reference: String,
    /// Whether the infrastructure is compliant
    pub is_compliant: bool,
    /// Violations detected (if any)
    pub violations: Vec<String>,
    /// Last checked timestamp
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

/// Integration example showing event-based communication
///
/// This demonstrates how infrastructure and policy domains communicate
/// through events rather than direct coupling.
pub mod integration_example {
    use super::*;

    /// Example: Infrastructure emits policy application event
    ///
    /// This event would be published to NATS:
    /// Subject: `infrastructure.policy.applied`
    pub fn create_policy_applied_event(
        resource_id: ResourceId,
        policy_reference: String,
        applied_by: String,
    ) -> serde_json::Value {
        serde_json::json!({
            "event_type": "PolicyAppliedToResource",
            "event_id": Uuid::now_v7(),
            "timestamp": chrono::Utc::now(),
            "payload": {
                "resource_id": resource_id.to_string(),
                "policy_reference": policy_reference,
                "applied_by": applied_by,
            }
        })
    }

    /// Example: Policy domain evaluates and responds
    ///
    /// cim-domain-policy would subscribe to infrastructure.policy.applied
    /// and respond with evaluation results on:
    /// Subject: `events.policy.{policy_id}.evaluated`
    pub fn handle_policy_evaluation_response(response: serde_json::Value) -> PolicyComplianceStatus {
        PolicyComplianceStatus {
            policy_reference: response["policy_id"].as_str().unwrap_or("unknown").to_string(),
            is_compliant: response["compliant"].as_bool().unwrap_or(false),
            violations: response["violations"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            checked_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResourceId, NetworkId};

    #[test]
    fn test_policy_adapter_creation() {
        let adapter = PolicyApplicationAdapter::new();
        assert_eq!(adapter.resource_policies.len(), 0);
        assert_eq!(adapter.network_policies.len(), 0);
    }

    #[test]
    fn test_apply_policy_to_resource() {
        let mut adapter = PolicyApplicationAdapter::new();
        let resource_id = ResourceId::new("test-vm").unwrap();

        let application = adapter.apply_policy_to_resource(
            &resource_id,
            "policy-123".to_string(),
            "admin".to_string(),
            "Security compliance".to_string(),
        );

        assert_eq!(application.policy_reference, "policy-123");
        assert_eq!(application.applied_by, "admin");

        let policies = adapter.get_resource_policies(&resource_id);
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0].policy_reference, "policy-123");
    }

    #[test]
    fn test_apply_policy_to_network() {
        let mut adapter = PolicyApplicationAdapter::new();
        let network_id = NetworkId::new("frontend-net").unwrap();

        let application = adapter.apply_policy_to_network(
            &network_id,
            "network-policy-456".to_string(),
            "network-admin".to_string(),
            "Network segmentation".to_string(),
        );

        assert_eq!(application.policy_reference, "network-policy-456");

        let policies = adapter.get_network_policies(&network_id);
        assert_eq!(policies.len(), 1);
    }

    #[test]
    fn test_get_all_policy_references() {
        let mut adapter = PolicyApplicationAdapter::new();
        let resource_id = ResourceId::new("vm1").unwrap();
        let network_id = NetworkId::new("net1").unwrap();

        adapter.apply_policy_to_resource(
            &resource_id,
            "policy-1".to_string(),
            "admin".to_string(),
            "test".to_string(),
        );

        adapter.apply_policy_to_network(
            &network_id,
            "policy-2".to_string(),
            "admin".to_string(),
            "test".to_string(),
        );

        adapter.apply_policy_to_resource(
            &resource_id,
            "policy-1".to_string(), // Duplicate
            "admin".to_string(),
            "test".to_string(),
        );

        let references = adapter.get_all_policy_references();
        assert_eq!(references.len(), 2);
        assert!(references.contains(&"policy-1".to_string()));
        assert!(references.contains(&"policy-2".to_string()));
    }

    #[test]
    fn test_integration_example() {
        let resource_id = ResourceId::new("test-resource").unwrap();
        let event = integration_example::create_policy_applied_event(
            resource_id,
            "security-policy-123".to_string(),
            "admin-user".to_string(),
        );

        assert_eq!(event["event_type"], "PolicyAppliedToResource");
        assert!(event["payload"]["resource_id"].is_string());
        assert_eq!(event["payload"]["policy_reference"], "security-policy-123");
    }

    #[test]
    fn test_policy_compliance_status() {
        let response = serde_json::json!({
            "policy_id": "policy-123",
            "compliant": false,
            "violations": ["Missing encryption", "Weak password policy"],
        });

        let status = integration_example::handle_policy_evaluation_response(response);

        assert_eq!(status.policy_reference, "policy-123");
        assert!(!status.is_compliant);
        assert_eq!(status.violations.len(), 2);
        assert!(status.violations.contains(&"Missing encryption".to_string()));
    }
}
