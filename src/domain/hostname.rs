// Copyright (c) 2025 - Cowboy AI, Inc.
//! Hostname Value Object with DNS Validation Invariants

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Hostname validation error
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HostnameError {
    #[error("Hostname is empty")]
    Empty,

    #[error("Hostname exceeds maximum length of 253 characters: {0}")]
    TooLong(usize),

    #[error("Label exceeds maximum length of 63 characters: {0}")]
    LabelTooLong(String),

    #[error("Invalid character in hostname: {0}")]
    InvalidCharacter(char),

    #[error("Label cannot start or end with hyphen: {0}")]
    InvalidLabelFormat(String),

    #[error("Label cannot be all numeric: {0}")]
    NumericLabel(String),
}

/// Fully Qualified Domain Name (FQDN) value object
///
/// Represents a valid DNS hostname following RFC 1123 with invariants:
/// - Total length ≤ 253 characters
/// - Each label ≤ 63 characters
/// - Labels separated by dots
/// - Labels contain only alphanumeric and hyphens
/// - Labels cannot start or end with hyphens
/// - Labels cannot be all numeric (for compatibility)
///
/// # Examples
///
/// ```rust
/// use cim_infrastructure::domain::Hostname;
///
/// // Valid hostnames
/// let host = Hostname::new("web01.example.com").unwrap();
/// let short = Hostname::new("localhost").unwrap();
///
/// // Invalid hostnames
/// assert!(Hostname::new("").is_err());  // Empty
/// assert!(Hostname::new("-invalid").is_err());  // Starts with hyphen
/// assert!(Hostname::new("invalid-.com").is_err());  // Ends with hyphen
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hostname(String);

impl Hostname {
    /// Maximum total length for FQDN (RFC 1123)
    pub const MAX_LENGTH: usize = 253;

    /// Maximum length for a single label (RFC 1123)
    pub const MAX_LABEL_LENGTH: usize = 63;

    /// Create a new hostname with validation
    ///
    /// # Invariants
    /// - Non-empty
    /// - Total length ≤ 253 characters
    /// - Each label ≤ 63 characters
    /// - Valid DNS characters only
    /// - Proper label format
    pub fn new(hostname: impl Into<String>) -> Result<Self, HostnameError> {
        let hostname = hostname.into();

        // Invariant 1: Non-empty
        if hostname.is_empty() {
            return Err(HostnameError::Empty);
        }

        // Invariant 2: Maximum total length
        if hostname.len() > Self::MAX_LENGTH {
            return Err(HostnameError::TooLong(hostname.len()));
        }

        // Validate each label
        for label in hostname.split('.') {
            Self::validate_label(label)?;
        }

        Ok(Self(hostname))
    }

    /// Validate a single DNS label
    fn validate_label(label: &str) -> Result<(), HostnameError> {
        // Invariant 3: Label not empty
        if label.is_empty() {
            return Err(HostnameError::Empty);
        }

        // Invariant 4: Label maximum length
        if label.len() > Self::MAX_LABEL_LENGTH {
            return Err(HostnameError::LabelTooLong(label.to_string()));
        }

        // Invariant 5: Valid characters (alphanumeric + hyphen)
        for ch in label.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '-' {
                return Err(HostnameError::InvalidCharacter(ch));
            }
        }

        // Invariant 6: Cannot start or end with hyphen
        if label.starts_with('-') || label.ends_with('-') {
            return Err(HostnameError::InvalidLabelFormat(label.to_string()));
        }

        // Invariant 7: TLD cannot be all numeric (RFC recommendation)
        let is_tld = !label.contains('.');
        if is_tld && label.chars().all(|c| c.is_ascii_digit()) {
            return Err(HostnameError::NumericLabel(label.to_string()));
        }

        Ok(())
    }

    /// Get the hostname as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the short name (first label before first dot)
    pub fn short_name(&self) -> &str {
        self.0.split('.').next().unwrap_or(&self.0)
    }

    /// Get the domain name (everything after first dot)
    pub fn domain(&self) -> Option<&str> {
        self.0.split_once('.').map(|(_, domain)| domain)
    }

    /// Check if this is a fully qualified domain name (contains dots)
    pub fn is_fqdn(&self) -> bool {
        self.0.contains('.')
    }

    /// Get labels as a vector
    pub fn labels(&self) -> Vec<&str> {
        self.0.split('.').collect()
    }

    /// Convert to lowercase (canonical form)
    pub fn to_lowercase(&self) -> Self {
        Self(self.0.to_lowercase())
    }
}

impl fmt::Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Hostname {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Hostname {
    type Error = HostnameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Hostname {
    type Error = HostnameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_hostnames() {
        assert!(Hostname::new("localhost").is_ok());
        assert!(Hostname::new("web01.example.com").is_ok());
        assert!(Hostname::new("api-server.prod.us-east-1.example.com").is_ok());
        assert!(Hostname::new("a").is_ok());
        assert!(Hostname::new("a.b").is_ok());
    }

    #[test]
    fn test_invalid_hostnames() {
        assert!(Hostname::new("").is_err());  // Empty
        assert!(Hostname::new("-invalid").is_err());  // Starts with hyphen
        assert!(Hostname::new("invalid-").is_err());  // Ends with hyphen
        assert!(Hostname::new("invalid..com").is_err());  // Empty label
        assert!(Hostname::new("invalid_.com").is_err());  // Invalid character
        assert!(Hostname::new("123").is_err());  // All numeric TLD
    }

    #[test]
    fn test_length_limits() {
        // Label too long (64 characters)
        let long_label = "a".repeat(64);
        assert!(Hostname::new(format!("{}.com", long_label)).is_err());

        // Valid label (63 characters)
        let max_label = "a".repeat(63);
        assert!(Hostname::new(format!("{}.com", max_label)).is_ok());

        // Total length too long (254 characters)
        let long_fqdn = format!("{}.{}.com", "a".repeat(125), "b".repeat(125));
        assert!(Hostname::new(long_fqdn).is_err());
    }

    #[test]
    fn test_hostname_parsing() {
        let host = Hostname::new("web01.prod.example.com").unwrap();
        assert_eq!(host.short_name(), "web01");
        assert_eq!(host.domain(), Some("prod.example.com"));
        assert!(host.is_fqdn());
        assert_eq!(host.labels(), vec!["web01", "prod", "example", "com"]);
    }

    #[test]
    fn test_hostname_display() {
        let host = Hostname::new("web01.example.com").unwrap();
        assert_eq!(format!("{}", host), "web01.example.com");
        assert_eq!(host.as_str(), "web01.example.com");
    }

    #[test]
    fn test_hostname_case() {
        let host1 = Hostname::new("WEB01.EXAMPLE.COM").unwrap();
        let host2 = host1.to_lowercase();
        assert_eq!(host2.as_str(), "web01.example.com");
    }
}
