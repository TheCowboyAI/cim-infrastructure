// Copyright 2025 Cowboy AI, LLC.

//! Infrastructure Domain Value Objects
//!
//! These are the building blocks of the Infrastructure domain model.
//! All value objects are immutable and validated on construction.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

/// Error types for Infrastructure value objects
#[derive(Debug, Error, Clone, PartialEq)]
pub enum InfrastructureError {
    #[error("Invalid resource ID: {0}")]
    InvalidResourceId(String),

    #[error("Invalid network ID: {0}")]
    InvalidNetworkId(String),

    #[error("Invalid CIDR notation: {0}")]
    InvalidCidr(String),

    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    #[error("Invalid interface name: {0}")]
    InvalidInterface(String),

    #[error("Invalid software ID: {0}")]
    InvalidSoftwareId(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, InfrastructureError>;

// ============================================================================
// Identity Value Objects
// ============================================================================

/// Unique identifier for Infrastructure aggregate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InfrastructureId(Uuid);

impl InfrastructureId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for InfrastructureId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for InfrastructureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for compute resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(String);

impl ResourceId {
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(InfrastructureError::InvalidResourceId(
                "Resource ID cannot be empty".into(),
            ));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ResourceId {
    type Err = InfrastructureError;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

/// Unique identifier for networks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(String);

impl NetworkId {
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(InfrastructureError::InvalidNetworkId(
                "Network ID cannot be empty".into(),
            ));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for NetworkId {
    type Err = InfrastructureError;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

/// Unique identifier for network interfaces
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InterfaceId(String);

impl InterfaceId {
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(InfrastructureError::InvalidInterface(
                "Interface ID cannot be empty".into(),
            ));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InterfaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for topology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopologyId(Uuid);

impl TopologyId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for TopologyId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TopologyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for software artifacts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SoftwareId(String);

impl SoftwareId {
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(InfrastructureError::InvalidSoftwareId(
                "Software ID cannot be empty".into(),
            ));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SoftwareId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigurationId(Uuid);

impl ConfigurationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for ConfigurationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ConfigurationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyId(Uuid);

impl PolicyId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PolicyId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PolicyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Compute Resource Value Objects
// ============================================================================

/// Type of compute resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComputeType {
    /// Physical server hardware
    Physical,
    /// Virtual machine
    VirtualMachine,
    /// Container (Docker, Podman, etc.)
    Container,
}

impl fmt::Display for ComputeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComputeType::Physical => write!(f, "physical"),
            ComputeType::VirtualMachine => write!(f, "vm"),
            ComputeType::Container => write!(f, "container"),
        }
    }
}

/// System architecture (maps to Nix system string)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemArchitecture(String);

impl SystemArchitecture {
    pub fn new(arch: impl Into<String>) -> Self {
        Self(arch.into())
    }

    pub fn x86_64_linux() -> Self {
        Self("x86_64-linux".into())
    }

    pub fn aarch64_linux() -> Self {
        Self("aarch64-linux".into())
    }

    pub fn x86_64_darwin() -> Self {
        Self("x86_64-darwin".into())
    }

    pub fn aarch64_darwin() -> Self {
        Self("aarch64-darwin".into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SystemArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for SystemArchitecture {
    fn default() -> Self {
        Self::x86_64_linux()
    }
}

/// Hostname value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hostname(String);

impl Hostname {
    pub fn new(hostname: impl Into<String>) -> Result<Self> {
        let hostname = hostname.into();

        // Basic hostname validation
        if hostname.is_empty() {
            return Err(InfrastructureError::InvalidHostname(
                "Hostname cannot be empty".into(),
            ));
        }

        if hostname.len() > 253 {
            return Err(InfrastructureError::InvalidHostname(
                "Hostname too long (max 253 characters)".into(),
            ));
        }

        // Check valid characters
        if !hostname
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        {
            return Err(InfrastructureError::InvalidHostname(
                "Hostname contains invalid characters".into(),
            ));
        }

        Ok(Self(hostname))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Hostname {
    type Err = InfrastructureError;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

/// Resource capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    pub cpu_cores: Option<u32>,
    pub memory_mb: Option<u64>,
    pub storage_gb: Option<u64>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ResourceCapabilities {
    pub fn new() -> Self {
        Self {
            cpu_cores: None,
            memory_mb: None,
            storage_gb: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_cpu(mut self, cores: u32) -> Self {
        self.cpu_cores = Some(cores);
        self
    }

    pub fn with_memory(mut self, mb: u64) -> Self {
        self.memory_mb = Some(mb);
        self
    }

    pub fn with_storage(mut self, gb: u64) -> Self {
        self.storage_gb = Some(gb);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Default for ResourceCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Network Value Objects
// ============================================================================

/// IPv4 network with CIDR notation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ipv4Network {
    pub address: Ipv4Addr,
    pub prefix_len: u8,
}

impl Ipv4Network {
    pub fn new(address: Ipv4Addr, prefix_len: u8) -> Result<Self> {
        if prefix_len > 32 {
            return Err(InfrastructureError::InvalidCidr(
                "IPv4 prefix length must be <= 32".into(),
            ));
        }
        Ok(Self { address, prefix_len })
    }
}

impl fmt::Display for Ipv4Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix_len)
    }
}

impl FromStr for Ipv4Network {
    type Err = InfrastructureError;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(InfrastructureError::InvalidCidr(format!(
                "Invalid CIDR notation: {}",
                s
            )));
        }

        let address = parts[0]
            .parse::<Ipv4Addr>()
            .map_err(|e| InfrastructureError::InvalidCidr(format!("Invalid IPv4 address: {}", e)))?;

        let prefix_len = parts[1]
            .parse::<u8>()
            .map_err(|e| InfrastructureError::InvalidCidr(format!("Invalid prefix length: {}", e)))?;

        Self::new(address, prefix_len)
    }
}

/// IPv6 network with CIDR notation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ipv6Network {
    pub address: Ipv6Addr,
    pub prefix_len: u8,
}

impl Ipv6Network {
    pub fn new(address: Ipv6Addr, prefix_len: u8) -> Result<Self> {
        if prefix_len > 128 {
            return Err(InfrastructureError::InvalidCidr(
                "IPv6 prefix length must be <= 128".into(),
            ));
        }
        Ok(Self { address, prefix_len })
    }
}

impl fmt::Display for Ipv6Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix_len)
    }
}

impl FromStr for Ipv6Network {
    type Err = InfrastructureError;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(InfrastructureError::InvalidCidr(format!(
                "Invalid CIDR notation: {}",
                s
            )));
        }

        let address = parts[0]
            .parse::<Ipv6Addr>()
            .map_err(|e| InfrastructureError::InvalidCidr(format!("Invalid IPv6 address: {}", e)))?;

        let prefix_len = parts[1]
            .parse::<u8>()
            .map_err(|e| InfrastructureError::InvalidCidr(format!("Invalid prefix length: {}", e)))?;

        Self::new(address, prefix_len)
    }
}

// ============================================================================
// Software Value Objects
// ============================================================================

/// Software version
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version(String);

impl Version {
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    pub fn parse(version: &str) -> Result<Self> {
        // Basic validation - can be extended
        if version.is_empty() {
            return Err(InfrastructureError::ValidationError(
                "Version cannot be empty".into(),
            ));
        }
        Ok(Self(version.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Policy scope
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyScope {
    /// Applied to specific resource
    Resource(ResourceId),
    /// Applied to network
    Network(NetworkId),
    /// Applied globally
    Global,
}

impl fmt::Display for PolicyScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyScope::Resource(id) => write!(f, "resource:{}", id),
            PolicyScope::Network(id) => write!(f, "network:{}", id),
            PolicyScope::Global => write!(f, "global"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_id_creation() {
        let id = ResourceId::new("server01").unwrap();
        assert_eq!(id.as_str(), "server01");
    }

    #[test]
    fn test_resource_id_empty_fails() {
        assert!(ResourceId::new("").is_err());
    }

    #[test]
    fn test_hostname_validation() {
        assert!(Hostname::new("server01").is_ok());
        assert!(Hostname::new("web-01.example.com").is_ok());
        assert!(Hostname::new("").is_err());
        assert!(Hostname::new("invalid_hostname").is_err());
    }

    #[test]
    fn test_ipv4_network_parsing() {
        let net: Ipv4Network = "10.0.1.0/24".parse().unwrap();
        assert_eq!(net.prefix_len, 24);
        assert_eq!(net.to_string(), "10.0.1.0/24");
    }

    #[test]
    fn test_ipv6_network_parsing() {
        let net: Ipv6Network = "2001:db8::/32".parse().unwrap();
        assert_eq!(net.prefix_len, 32);
    }

    #[test]
    fn test_system_architecture() {
        let arch = SystemArchitecture::x86_64_linux();
        assert_eq!(arch.as_str(), "x86_64-linux");
    }

    #[test]
    fn test_compute_type_display() {
        assert_eq!(ComputeType::Physical.to_string(), "physical");
        assert_eq!(ComputeType::VirtualMachine.to_string(), "vm");
        assert_eq!(ComputeType::Container.to_string(), "container");
    }

    #[test]
    fn test_uuid_based_ids() {
        let id1 = InfrastructureId::new();
        let id2 = InfrastructureId::new();
        assert_ne!(id1, id2);

        let topology_id = TopologyId::new();
        assert_eq!(topology_id.as_uuid().get_version_num(), 7);
    }

    #[test]
    fn test_resource_capabilities_builder() {
        let caps = ResourceCapabilities::new()
            .with_cpu(8)
            .with_memory(16384)
            .with_storage(500)
            .with_metadata("vendor".into(), "Dell".into());

        assert_eq!(caps.cpu_cores, Some(8));
        assert_eq!(caps.memory_mb, Some(16384));
        assert_eq!(caps.storage_gb, Some(500));
        assert_eq!(caps.metadata.get("vendor"), Some(&"Dell".to_string()));
    }
}
