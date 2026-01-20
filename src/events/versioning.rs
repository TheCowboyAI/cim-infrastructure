// Copyright (c) 2025 - Cowboy AI, Inc.
//! Event Versioning Infrastructure
//!
//! Provides upcasting support for event schema evolution. When event schemas change
//! over time, upcasters transform old versions to the latest version on-read.
//!
//! # Design Principles
//!
//! 1. **Upcasting on Read**: Old events are transformed when loaded from event store
//! 2. **Application Sees Latest Only**: Business logic only handles current version
//! 3. **Chain of Upcasters**: Multiple version migrations can be composed
//! 4. **Type Safety**: Upcasters are strongly typed
//!
//! # Architecture
//!
//! ```text
//! Event Store → Raw JSON → Deserialize → Upcast → Application
//!                                          ↓
//!                           V1 → V2 → V3 (chain)
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use cim_infrastructure::events::versioning::*;
//!
//! // Define upcaster for ResourceRegistered v1 → v2
//! struct ResourceRegisteredV1ToV2;
//!
//! impl Upcaster<ResourceRegistered> for ResourceRegisteredV1ToV2 {
//!     fn from_version(&self) -> u32 { 1 }
//!     fn to_version(&self) -> u32 { 2 }
//!
//!     fn upcast(&self, value: serde_json::Value) -> Result<serde_json::Value, UpcastError> {
//!         // Transform v1 JSON to v2 JSON
//!         let mut v2 = value;
//!         v2["new_field"] = serde_json::json!("default_value");
//!         Ok(v2)
//!     }
//! }
//! ```
//!
//! # References
//!
//! - [Event Sourcing: What is Upcasting?](https://artium.ai/insights/event-sourcing-what-is-upcasting-a-deep-dive)
//! - [Simple patterns for events schema versioning](https://event-driven.io/en/simple_events_versioning_patterns/)
//! - [Marten Events Versioning](https://martendb.io/events/versioning.html)

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::errors::InfrastructureError;

/// Error type for upcasting operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpcastError {
    /// Version not supported by this upcaster
    UnsupportedVersion { from: u32, to: u32, found: u32 },

    /// JSON transformation failed
    TransformationFailed(String),

    /// Deserialization failed after upcast
    DeserializationFailed(String),

    /// Missing required field in old version
    MissingField(String),

    /// Invalid field value that cannot be migrated
    InvalidFieldValue { field: String, reason: String },
}

impl fmt::Display for UpcastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpcastError::UnsupportedVersion { from, to, found } => {
                write!(
                    f,
                    "Upcaster expects version {}, got version {}. Can only upcast to version {}",
                    from, found, to
                )
            }
            UpcastError::TransformationFailed(msg) => {
                write!(f, "Event transformation failed: {}", msg)
            }
            UpcastError::DeserializationFailed(msg) => {
                write!(f, "Deserialization after upcast failed: {}", msg)
            }
            UpcastError::MissingField(field) => {
                write!(f, "Required field '{}' missing in old event version", field)
            }
            UpcastError::InvalidFieldValue { field, reason } => {
                write!(f, "Invalid value in field '{}': {}", field, reason)
            }
        }
    }
}

impl std::error::Error for UpcastError {}

impl From<UpcastError> for InfrastructureError {
    fn from(err: UpcastError) -> Self {
        InfrastructureError::Serialization(err.to_string())
    }
}

/// Trait for upcasting events from one version to another
///
/// Upcasters work on JSON values to provide maximum flexibility.
/// The transformation happens between deserialization and application logic.
pub trait Upcaster<T>: Send + Sync {
    /// Version this upcaster expects as input
    fn from_version(&self) -> u32;

    /// Version this upcaster produces as output
    fn to_version(&self) -> u32;

    /// Transform event JSON from old version to new version
    ///
    /// This method should:
    /// 1. Verify the input is actually the expected version
    /// 2. Apply schema transformations (add/remove/rename fields)
    /// 3. Set default values for new fields
    /// 4. Update the event_version field
    fn upcast(&self, value: serde_json::Value) -> Result<serde_json::Value, UpcastError>;

    /// Optional: Validate that the transformation was successful
    ///
    /// This can be used to verify invariants after upcasting.
    fn validate(&self, _value: &serde_json::Value) -> Result<(), UpcastError> {
        Ok(())
    }
}

/// Chain of upcasters that can migrate an event through multiple versions
///
/// # Example
///
/// ```rust,ignore
/// let mut chain = UpcasterChain::new();
/// chain.add(ResourceRegisteredV1ToV2);
/// chain.add(ResourceRegisteredV2ToV3);
///
/// // Automatically migrates v1 → v2 → v3
/// let v3_event = chain.upcast_to_latest(v1_json)?;
/// ```
pub struct UpcasterChain<T> {
    upcasters: Vec<Box<dyn Upcaster<T>>>,
}

impl<T> UpcasterChain<T> {
    /// Create a new empty upcaster chain
    pub fn new() -> Self {
        Self {
            upcasters: Vec::new(),
        }
    }

    /// Add an upcaster to the chain
    ///
    /// Upcasters should be added in version order (v1→v2, then v2→v3, etc.)
    pub fn add<U: Upcaster<T> + 'static>(&mut self, upcaster: U) {
        self.upcasters.push(Box::new(upcaster));
    }

    /// Get the latest version this chain can produce
    pub fn latest_version(&self) -> Option<u32> {
        self.upcasters.last().map(|u| u.to_version())
    }

    /// Upcast an event to the latest version
    ///
    /// This method will apply all upcasters in the chain sequentially
    /// until the event reaches the latest version.
    pub fn upcast_to_latest(
        &self,
        mut value: serde_json::Value,
        current_version: u32,
    ) -> Result<serde_json::Value, UpcastError> {
        let mut version = current_version;

        for upcaster in &self.upcasters {
            // If we've reached this upcaster's input version, apply it
            if version == upcaster.from_version() {
                value = upcaster.upcast(value)?;
                upcaster.validate(&value)?;
                version = upcaster.to_version();
            }
        }

        Ok(value)
    }

    /// Upcast an event to a specific target version
    ///
    /// This is useful when you need to migrate to an intermediate version,
    /// not necessarily the latest.
    pub fn upcast_to_version(
        &self,
        mut value: serde_json::Value,
        current_version: u32,
        target_version: u32,
    ) -> Result<serde_json::Value, UpcastError> {
        if current_version == target_version {
            return Ok(value);
        }

        if current_version > target_version {
            return Err(UpcastError::UnsupportedVersion {
                from: current_version,
                to: target_version,
                found: current_version,
            });
        }

        let mut version = current_version;

        for upcaster in &self.upcasters {
            if version == upcaster.from_version() && version < target_version {
                value = upcaster.upcast(value)?;
                upcaster.validate(&value)?;
                version = upcaster.to_version();

                // Stop if we've reached the target
                if version == target_version {
                    break;
                }
            }
        }

        if version != target_version {
            return Err(UpcastError::UnsupportedVersion {
                from: current_version,
                to: target_version,
                found: version,
            });
        }

        Ok(value)
    }
}

impl<T> Default for UpcasterChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to extract event version from JSON
pub fn get_event_version(value: &serde_json::Value) -> Result<u32, UpcastError> {
    value
        .get("event_version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .ok_or_else(|| {
            UpcastError::MissingField("event_version".to_string())
        })
}

/// Helper to set event version in JSON
pub fn set_event_version(value: &mut serde_json::Value, version: u32) -> Result<(), UpcastError> {
    if let Some(obj) = value.as_object_mut() {
        obj.insert("event_version".to_string(), serde_json::json!(version));
        Ok(())
    } else {
        Err(UpcastError::TransformationFailed(
            "Event is not a JSON object".to_string(),
        ))
    }
}

/// Metadata about an event version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventVersionInfo {
    /// Event type name
    pub event_type: String,

    /// Current version number
    pub version: u32,

    /// When this version was introduced
    pub introduced_at: String,

    /// Description of changes from previous version
    pub changes: Vec<String>,

    /// Whether this version is deprecated
    pub deprecated: bool,
}

impl EventVersionInfo {
    /// Create version info for an event type
    pub fn new(event_type: impl Into<String>, version: u32) -> Self {
        Self {
            event_type: event_type.into(),
            version,
            introduced_at: String::new(),
            changes: Vec::new(),
            deprecated: false,
        }
    }

    /// Set when this version was introduced
    pub fn introduced_at(mut self, date: impl Into<String>) -> Self {
        self.introduced_at = date.into();
        self
    }

    /// Add a change description
    pub fn with_change(mut self, change: impl Into<String>) -> Self {
        self.changes.push(change.into());
        self
    }

    /// Mark this version as deprecated
    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock event type for testing
    struct TestEvent;

    #[test]
    fn test_upcast_error_display() {
        let err = UpcastError::UnsupportedVersion {
            from: 1,
            to: 2,
            found: 3,
        };
        assert!(err.to_string().contains("version 1"));
        assert!(err.to_string().contains("version 3"));
    }

    #[test]
    fn test_get_event_version() {
        let json = serde_json::json!({
            "event_version": 2,
            "data": "test"
        });

        assert_eq!(get_event_version(&json).unwrap(), 2);
    }

    #[test]
    fn test_get_event_version_missing() {
        let json = serde_json::json!({
            "data": "test"
        });

        assert!(get_event_version(&json).is_err());
    }

    #[test]
    fn test_set_event_version() {
        let mut json = serde_json::json!({
            "event_version": 1,
            "data": "test"
        });

        set_event_version(&mut json, 2).unwrap();
        assert_eq!(get_event_version(&json).unwrap(), 2);
    }

    #[test]
    fn test_event_version_info_builder() {
        let info = EventVersionInfo::new("ResourceRegistered", 2)
            .introduced_at("2026-01-19")
            .with_change("Added new_field")
            .with_change("Renamed old_field to better_field")
            .deprecated();

        assert_eq!(info.event_type, "ResourceRegistered");
        assert_eq!(info.version, 2);
        assert_eq!(info.changes.len(), 2);
        assert!(info.deprecated);
    }

    // Test upcaster implementation
    struct TestV1ToV2Upcaster;

    impl Upcaster<TestEvent> for TestV1ToV2Upcaster {
        fn from_version(&self) -> u32 {
            1
        }

        fn to_version(&self) -> u32 {
            2
        }

        fn upcast(&self, mut value: serde_json::Value) -> Result<serde_json::Value, UpcastError> {
            // Add new field with default value
            if let Some(obj) = value.as_object_mut() {
                obj.insert("new_field".to_string(), serde_json::json!("default"));
                set_event_version(&mut value, 2)?;
                Ok(value)
            } else {
                Err(UpcastError::TransformationFailed(
                    "Not an object".to_string(),
                ))
            }
        }
    }

    #[test]
    fn test_upcaster_chain_empty() {
        let chain: UpcasterChain<TestEvent> = UpcasterChain::new();
        assert_eq!(chain.latest_version(), None);
    }

    #[test]
    fn test_upcaster_chain_single() {
        let mut chain: UpcasterChain<TestEvent> = UpcasterChain::new();
        chain.add(TestV1ToV2Upcaster);

        assert_eq!(chain.latest_version(), Some(2));

        let v1_json = serde_json::json!({
            "event_version": 1,
            "data": "test"
        });

        let v2_json = chain.upcast_to_latest(v1_json, 1).unwrap();
        assert_eq!(get_event_version(&v2_json).unwrap(), 2);
        assert_eq!(v2_json["new_field"], "default");
    }

    #[test]
    fn test_upcaster_chain_already_latest() {
        let mut chain: UpcasterChain<TestEvent> = UpcasterChain::new();
        chain.add(TestV1ToV2Upcaster);

        let v2_json = serde_json::json!({
            "event_version": 2,
            "data": "test",
            "new_field": "existing"
        });

        let result = chain.upcast_to_latest(v2_json.clone(), 2).unwrap();
        assert_eq!(result, v2_json);
    }
}
