// Copyright (c) 2025 - Cowboy AI, Inc.
//! Network Value Objects with Validation Invariants

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;
use thiserror::Error;

/// Network validation error
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NetworkError {
    #[error("Invalid IP address format: {0}")]
    InvalidIpAddress(String),

    #[error("Invalid CIDR notation: {0}")]
    InvalidCidr(String),

    #[error("Invalid prefix length: {0} (must be 0-32 for IPv4, 0-128 for IPv6)")]
    InvalidPrefixLength(u8),

    #[error("Invalid MAC address format: {0}")]
    InvalidMacAddress(String),

    #[error("Invalid VLAN ID: {0} (must be 1-4094)")]
    InvalidVlanId(u16),

    #[error("Invalid MTU: {0} (must be 68-9000)")]
    InvalidMtu(u32),
}

/// IP Address with CIDR notation value object
///
/// Represents an IPv4 or IPv6 address with optional prefix length.
/// Invariants:
/// - Valid IP address format
/// - Prefix length within valid range
/// - Canonical representation
///
/// # Examples
///
/// ```rust
/// use cim_infrastructure::domain::IpAddressWithCidr;
///
/// let ip = IpAddressWithCidr::new("192.168.1.10/24").unwrap();
/// assert_eq!(ip.address().to_string(), "192.168.1.10");
/// assert_eq!(ip.prefix_length(), Some(24));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IpAddressWithCidr {
    address: IpAddr,
    prefix_length: Option<u8>,
}

impl IpAddressWithCidr {
    /// Create a new IP address with optional CIDR notation
    ///
    /// # Invariants
    /// - Valid IP address format
    /// - Prefix length 0-32 for IPv4, 0-128 for IPv6
    pub fn new(cidr: impl AsRef<str>) -> Result<Self, NetworkError> {
        let cidr = cidr.as_ref();

        // Parse CIDR notation (e.g., "192.168.1.10/24")
        if let Some((addr_str, prefix_str)) = cidr.split_once('/') {
            let address = IpAddr::from_str(addr_str)
                .map_err(|_| NetworkError::InvalidIpAddress(addr_str.to_string()))?;

            let prefix_length = prefix_str
                .parse::<u8>()
                .map_err(|_| NetworkError::InvalidCidr(cidr.to_string()))?;

            // Invariant: Validate prefix length based on IP version
            let max_prefix = match address {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            };

            if prefix_length > max_prefix {
                return Err(NetworkError::InvalidPrefixLength(prefix_length));
            }

            Ok(Self {
                address,
                prefix_length: Some(prefix_length),
            })
        } else {
            // Just an IP address without prefix
            let address = IpAddr::from_str(cidr)
                .map_err(|_| NetworkError::InvalidIpAddress(cidr.to_string()))?;

            Ok(Self {
                address,
                prefix_length: None,
            })
        }
    }

    /// Create from separate address and prefix
    pub fn from_parts(address: IpAddr, prefix_length: Option<u8>) -> Result<Self, NetworkError> {
        if let Some(prefix) = prefix_length {
            let max_prefix = match address {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            };

            if prefix > max_prefix {
                return Err(NetworkError::InvalidPrefixLength(prefix));
            }
        }

        Ok(Self {
            address,
            prefix_length,
        })
    }

    /// Get the IP address
    pub fn address(&self) -> IpAddr {
        self.address
    }

    /// Get the prefix length
    pub fn prefix_length(&self) -> Option<u8> {
        self.prefix_length
    }

    /// Check if this is an IPv4 address
    pub fn is_ipv4(&self) -> bool {
        matches!(self.address, IpAddr::V4(_))
    }

    /// Check if this is an IPv6 address
    pub fn is_ipv6(&self) -> bool {
        matches!(self.address, IpAddr::V6(_))
    }

    /// Get as CIDR notation string
    pub fn as_cidr(&self) -> String {
        if let Some(prefix) = self.prefix_length {
            format!("{}/{}", self.address, prefix)
        } else {
            self.address.to_string()
        }
    }
}

impl fmt::Display for IpAddressWithCidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_cidr())
    }
}

impl FromStr for IpAddressWithCidr {
    type Err = NetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// MAC Address value object
///
/// Represents a 48-bit MAC address with validation.
/// Invariants:
/// - Valid MAC address format (6 octets)
/// - Canonical representation (lowercase, colon-separated)
///
/// # Examples
///
/// ```rust
/// use cim_infrastructure::domain::MacAddress;
///
/// let mac = MacAddress::new("00:11:22:33:44:55").unwrap();
/// assert_eq!(mac.as_str(), "00:11:22:33:44:55");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Create a new MAC address with validation
    ///
    /// # Invariants
    /// - Valid MAC address format
    /// - 6 octets (48 bits)
    pub fn new(mac: impl AsRef<str>) -> Result<Self, NetworkError> {
        let mac = mac.as_ref();
        let mac_clean = mac.replace([':', '-'], "");

        // Invariant: Must be exactly 12 hex digits (6 octets)
        if mac_clean.len() != 12 {
            return Err(NetworkError::InvalidMacAddress(mac.to_string()));
        }

        let mut octets = [0u8; 6];
        for (i, chunk) in mac_clean.as_bytes().chunks(2).enumerate() {
            let hex_str = std::str::from_utf8(chunk)
                .map_err(|_| NetworkError::InvalidMacAddress(mac.to_string()))?;
            octets[i] = u8::from_str_radix(hex_str, 16)
                .map_err(|_| NetworkError::InvalidMacAddress(mac.to_string()))?;
        }

        Ok(Self(octets))
    }

    /// Create from raw octets
    pub fn from_octets(octets: [u8; 6]) -> Self {
        Self(octets)
    }

    /// Get the octets
    pub fn octets(&self) -> [u8; 6] {
        self.0
    }

    /// Get as canonical string (lowercase, colon-separated)
    pub fn as_str(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }

    /// Check if this is a broadcast MAC address
    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
    }

    /// Check if this is a multicast MAC address
    pub fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }

    /// Check if this is a unicast MAC address
    pub fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for MacAddress {
    type Err = NetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// VLAN ID value object
///
/// Represents a VLAN ID (IEEE 802.1Q) with validation.
/// Invariants:
/// - Valid VLAN ID range (1-4094)
/// - VLAN 0 and 4095 are reserved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VlanId(u16);

impl VlanId {
    /// Minimum valid VLAN ID
    pub const MIN: u16 = 1;

    /// Maximum valid VLAN ID
    pub const MAX: u16 = 4094;

    /// Create a new VLAN ID with validation
    ///
    /// # Invariants
    /// - VLAN ID must be 1-4094 (0 and 4095 are reserved)
    pub fn new(id: u16) -> Result<Self, NetworkError> {
        if id < Self::MIN || id > Self::MAX {
            return Err(NetworkError::InvalidVlanId(id));
        }

        Ok(Self(id))
    }

    /// Get the VLAN ID value
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for VlanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u16> for VlanId {
    type Error = NetworkError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// MTU (Maximum Transmission Unit) value object
///
/// Represents an MTU size with validation.
/// Invariants:
/// - Valid MTU range (68-9000 bytes)
/// - 68 = minimum IPv4 MTU
/// - 9000 = jumbo frames
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Mtu(u32);

impl Mtu {
    /// Minimum MTU (IPv4 minimum)
    pub const MIN: u32 = 68;

    /// Maximum MTU (jumbo frames)
    pub const MAX: u32 = 9000;

    /// Standard Ethernet MTU
    pub const STANDARD_ETHERNET: u32 = 1500;

    /// Jumbo frame MTU
    pub const JUMBO: u32 = 9000;

    /// Create a new MTU with validation
    ///
    /// # Invariants
    /// - MTU must be 68-9000 bytes
    pub fn new(size: u32) -> Result<Self, NetworkError> {
        if size < Self::MIN || size > Self::MAX {
            return Err(NetworkError::InvalidMtu(size));
        }

        Ok(Self(size))
    }

    /// Get the MTU value
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Check if this is standard Ethernet MTU (1500)
    pub fn is_standard_ethernet(&self) -> bool {
        self.0 == Self::STANDARD_ETHERNET
    }

    /// Check if this is a jumbo frame MTU (>1500)
    pub fn is_jumbo(&self) -> bool {
        self.0 > Self::STANDARD_ETHERNET
    }
}

impl Default for Mtu {
    fn default() -> Self {
        Self(Self::STANDARD_ETHERNET)
    }
}

impl fmt::Display for Mtu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u32> for Mtu {
    type Error = NetworkError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_address_cidr() {
        let ip = IpAddressWithCidr::new("192.168.1.10/24").unwrap();
        assert_eq!(ip.address().to_string(), "192.168.1.10");
        assert_eq!(ip.prefix_length(), Some(24));
        assert!(ip.is_ipv4());
        assert_eq!(ip.as_cidr(), "192.168.1.10/24");
    }

    #[test]
    fn test_ip_address_without_cidr() {
        let ip = IpAddressWithCidr::new("192.168.1.10").unwrap();
        assert_eq!(ip.prefix_length(), None);
        assert_eq!(ip.as_cidr(), "192.168.1.10");
    }

    #[test]
    fn test_ipv6_address() {
        let ip = IpAddressWithCidr::new("2001:db8::1/64").unwrap();
        assert!(ip.is_ipv6());
        assert_eq!(ip.prefix_length(), Some(64));
    }

    #[test]
    fn test_invalid_ip() {
        assert!(IpAddressWithCidr::new("999.999.999.999").is_err());
        assert!(IpAddressWithCidr::new("192.168.1.10/33").is_err());  // Invalid IPv4 prefix
        assert!(IpAddressWithCidr::new("2001:db8::1/129").is_err());  // Invalid IPv6 prefix
    }

    #[test]
    fn test_mac_address() {
        let mac = MacAddress::new("00:11:22:33:44:55").unwrap();
        assert_eq!(mac.as_str(), "00:11:22:33:44:55");
        assert!(mac.is_unicast());
        assert!(!mac.is_multicast());
    }

    #[test]
    fn test_mac_address_formats() {
        assert!(MacAddress::new("00:11:22:33:44:55").is_ok());
        assert!(MacAddress::new("00-11-22-33-44-55").is_ok());
        assert!(MacAddress::new("001122334455").is_ok());
    }

    #[test]
    fn test_mac_address_multicast() {
        let multicast = MacAddress::new("01:00:5e:00:00:01").unwrap();
        assert!(multicast.is_multicast());
        assert!(!multicast.is_unicast());
    }

    #[test]
    fn test_vlan_id() {
        assert!(VlanId::new(100).is_ok());
        assert!(VlanId::new(0).is_err());  // Reserved
        assert!(VlanId::new(4095).is_err());  // Reserved
        assert!(VlanId::new(5000).is_err());  // Out of range
    }

    #[test]
    fn test_mtu() {
        let mtu = Mtu::new(1500).unwrap();
        assert!(mtu.is_standard_ethernet());
        assert!(!mtu.is_jumbo());

        let jumbo = Mtu::new(9000).unwrap();
        assert!(jumbo.is_jumbo());

        assert!(Mtu::new(67).is_err());  // Too small
        assert!(Mtu::new(10000).is_err());  // Too large
    }
}
