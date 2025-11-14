//! NATS subject hierarchy for infrastructure events
//!
//! Subject pattern: `infrastructure.{aggregate}.{operation}`
//!
//! # Examples
//!
//! - `infrastructure.compute.registered`
//! - `infrastructure.network.defined`
//! - `infrastructure.connection.established`
//! - `infrastructure.software.configured`
//! - `infrastructure.policy.set`

use std::fmt;

/// Root namespace for all infrastructure subjects
pub const INFRASTRUCTURE_ROOT: &str = "infrastructure";

/// Infrastructure aggregate types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregateType {
    /// Compute resources (servers, VMs, containers)
    Compute,
    /// Network topology
    Network,
    /// Network connections
    Connection,
    /// Software and configurations
    Software,
    /// Policy rules
    Policy,
}

impl fmt::Display for AggregateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateType::Compute => write!(f, "compute"),
            AggregateType::Network => write!(f, "network"),
            AggregateType::Connection => write!(f, "connection"),
            AggregateType::Software => write!(f, "software"),
            AggregateType::Policy => write!(f, "policy"),
        }
    }
}

/// Infrastructure operations (event types)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    // Compute operations
    Registered,
    Decommissioned,
    Updated,

    // Network operations
    Defined,
    Removed,

    // Connection operations
    Established,
    Severed,

    // Software operations
    Configured,
    Deployed,

    // Interface operations
    Added,

    // Policy operations
    Set,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Registered => write!(f, "registered"),
            Operation::Decommissioned => write!(f, "decommissioned"),
            Operation::Updated => write!(f, "updated"),
            Operation::Defined => write!(f, "defined"),
            Operation::Removed => write!(f, "removed"),
            Operation::Established => write!(f, "established"),
            Operation::Severed => write!(f, "severed"),
            Operation::Configured => write!(f, "configured"),
            Operation::Deployed => write!(f, "deployed"),
            Operation::Added => write!(f, "added"),
            Operation::Set => write!(f, "set"),
        }
    }
}

/// Builder for infrastructure NATS subjects
#[derive(Debug, Clone)]
pub struct SubjectBuilder {
    aggregate: Option<AggregateType>,
    operation: Option<Operation>,
}

impl SubjectBuilder {
    /// Create a new subject builder
    pub fn new() -> Self {
        Self {
            aggregate: None,
            operation: None,
        }
    }

    /// Set the aggregate type
    pub fn aggregate(mut self, aggregate: AggregateType) -> Self {
        self.aggregate = Some(aggregate);
        self
    }

    /// Set the operation
    pub fn operation(mut self, operation: Operation) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Build the complete subject string
    ///
    /// # Panics
    ///
    /// Panics if aggregate or operation is not set
    pub fn build(self) -> String {
        let aggregate = self.aggregate.expect("aggregate must be set");
        let operation = self.operation.expect("operation must be set");
        format!("{}.{}.{}", INFRASTRUCTURE_ROOT, aggregate, operation)
    }

    /// Build the subject without operation (for subscriptions)
    ///
    /// Returns: `infrastructure.{aggregate}.*`
    pub fn build_wildcard(self) -> String {
        let aggregate = self.aggregate.expect("aggregate must be set");
        format!("{}.{}.>", INFRASTRUCTURE_ROOT, aggregate)
    }
}

impl Default for SubjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common subject patterns
pub mod subjects {
    use super::*;

    // Compute subjects
    pub fn compute_registered() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Compute)
            .operation(Operation::Registered)
            .build()
    }

    pub fn compute_decommissioned() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Compute)
            .operation(Operation::Decommissioned)
            .build()
    }

    pub fn compute_updated() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Compute)
            .operation(Operation::Updated)
            .build()
    }

    // Network subjects
    pub fn network_defined() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Network)
            .operation(Operation::Defined)
            .build()
    }

    pub fn network_removed() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Network)
            .operation(Operation::Removed)
            .build()
    }

    // Connection subjects
    pub fn connection_established() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Connection)
            .operation(Operation::Established)
            .build()
    }

    pub fn connection_severed() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Connection)
            .operation(Operation::Severed)
            .build()
    }

    // Software subjects
    pub fn software_configured() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Software)
            .operation(Operation::Configured)
            .build()
    }

    pub fn software_deployed() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Software)
            .operation(Operation::Deployed)
            .build()
    }

    // Policy subjects
    pub fn policy_set() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Policy)
            .operation(Operation::Set)
            .build()
    }

    // Wildcard subscriptions
    pub fn all_compute_events() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Compute)
            .build_wildcard()
    }

    pub fn all_network_events() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Network)
            .build_wildcard()
    }

    pub fn all_infrastructure_events() -> String {
        format!("{}.>", INFRASTRUCTURE_ROOT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_builder() {
        let subject = SubjectBuilder::new()
            .aggregate(AggregateType::Compute)
            .operation(Operation::Registered)
            .build();

        assert_eq!(subject, "infrastructure.compute.registered");
    }

    #[test]
    fn test_wildcard_subject() {
        let subject = SubjectBuilder::new()
            .aggregate(AggregateType::Network)
            .build_wildcard();

        assert_eq!(subject, "infrastructure.network.>");
    }

    #[test]
    fn test_convenience_functions() {
        assert_eq!(subjects::compute_registered(), "infrastructure.compute.registered");
        assert_eq!(subjects::network_defined(), "infrastructure.network.defined");
        assert_eq!(subjects::connection_established(), "infrastructure.connection.established");
        assert_eq!(subjects::software_configured(), "infrastructure.software.configured");
        assert_eq!(subjects::policy_set(), "infrastructure.policy.set");
    }

    #[test]
    fn test_wildcard_subscriptions() {
        assert_eq!(subjects::all_compute_events(), "infrastructure.compute.>");
        assert_eq!(subjects::all_network_events(), "infrastructure.network.>");
        assert_eq!(subjects::all_infrastructure_events(), "infrastructure.>");
    }
}
