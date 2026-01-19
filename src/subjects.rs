// Copyright (c) 2025 - Cowboy AI, Inc.

//! NATS subject hierarchy for infrastructure events
//!
//! Defines the semantic subject patterns used for infrastructure event routing.
//!
//! # Subject Pattern
//!
//! All infrastructure events follow the hierarchical pattern:
//!
//! ```text
//! infrastructure.{aggregate}.{operation}
//! ```
//!
//! This allows for:
//! - Precise subscriptions (`infrastructure.compute.registered`)
//! - Aggregate-level wildcards (`infrastructure.compute.>`)
//! - Global subscriptions (`infrastructure.>`)
//!
//! # Examples
//!
//! ```rust
//! use cim_infrastructure::subjects::{SubjectBuilder, AggregateType, Operation};
//!
//! // Build a specific subject
//! let subject = SubjectBuilder::new()
//!     .aggregate(AggregateType::Compute)
//!     .operation(Operation::Registered)
//!     .build();
//! assert_eq!(subject, "infrastructure.compute.registered");
//!
//! // Build a wildcard subscription
//! let wildcard = SubjectBuilder::new()
//!     .aggregate(AggregateType::Network)
//!     .build_wildcard();
//! assert_eq!(wildcard, "infrastructure.network.>");
//! ```

use std::fmt;

/// Root namespace for all infrastructure subjects
pub const INFRASTRUCTURE_ROOT: &str = "infrastructure";

/// Infrastructure aggregate types
///
/// These represent the bounded contexts within the infrastructure domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregateType {
    /// Compute resources (servers, VMs, containers)
    Compute,
    /// Network topology and configuration
    Network,
    /// Physical and logical connections between resources
    Connection,
    /// Software artifacts and configurations
    Software,
    /// Policy rules and enforcement
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
///
/// These represent domain events that can occur within each aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    // Compute operations
    /// A compute resource was registered in the system
    Registered,
    /// A compute resource was decommissioned
    Decommissioned,
    /// A compute resource was updated
    Updated,

    // Network operations
    /// A network was defined
    Defined,
    /// A network was removed
    Removed,

    // Connection operations
    /// A connection was established
    Established,
    /// A connection was severed
    Severed,

    // Software operations
    /// Software was configured
    Configured,
    /// Software was deployed
    Deployed,

    // Interface operations
    /// An interface was added
    Added,

    // Policy operations
    /// A policy was set or updated
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
///
/// Provides a type-safe way to construct NATS subject patterns.
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

    /// Build a wildcard subscription for all operations on this aggregate
    ///
    /// Returns: `infrastructure.{aggregate}.>`
    ///
    /// # Panics
    ///
    /// Panics if aggregate is not set
    pub fn build_wildcard(self) -> String {
        let aggregate = self.aggregate.expect("aggregate must be set");
        format!("{}.{}.>", INFRASTRUCTURE_ROOT, aggregate)
    }

    /// Build a subscription for all infrastructure events
    ///
    /// Returns: `infrastructure.>`
    pub fn build_all() -> String {
        format!("{}.>", INFRASTRUCTURE_ROOT)
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

    pub fn all_connection_events() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Connection)
            .build_wildcard()
    }

    pub fn all_software_events() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Software)
            .build_wildcard()
    }

    pub fn all_policy_events() -> String {
        SubjectBuilder::new()
            .aggregate(AggregateType::Policy)
            .build_wildcard()
    }

    pub fn all_infrastructure_events() -> String {
        SubjectBuilder::build_all()
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
    fn test_all_events_subscription() {
        assert_eq!(SubjectBuilder::build_all(), "infrastructure.>");
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

    #[test]
    fn test_aggregate_display() {
        assert_eq!(AggregateType::Compute.to_string(), "compute");
        assert_eq!(AggregateType::Network.to_string(), "network");
        assert_eq!(AggregateType::Connection.to_string(), "connection");
    }

    #[test]
    fn test_operation_display() {
        assert_eq!(Operation::Registered.to_string(), "registered");
        assert_eq!(Operation::Defined.to_string(), "defined");
        assert_eq!(Operation::Established.to_string(), "established");
    }
}
