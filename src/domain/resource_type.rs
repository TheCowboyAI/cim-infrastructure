// Copyright (c) 2025 - Cowboy AI, Inc.
//! Infrastructure Resource Type Domain Model
//!
//! Defines the taxonomy of infrastructure resources that can be registered
//! and managed within the CIM system. This provides a standardized vocabulary
//! for describing different types of physical and virtual infrastructure.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Infrastructure resource type taxonomy
///
/// This enum defines the complete set of infrastructure resource types
/// that CIM can model and project into various systems (NetBox, monitoring, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    // Compute Resources
    /// Physical server (bare metal)
    PhysicalServer,
    /// Virtual machine
    VirtualMachine,
    /// Container host
    ContainerHost,
    /// Hypervisor
    Hypervisor,

    // Network Infrastructure
    /// Network router
    Router,
    /// Network switch
    Switch,
    /// Layer 3 switch (routing + switching)
    Layer3Switch,
    /// Wireless access point
    AccessPoint,
    /// Load balancer
    LoadBalancer,

    // Security Appliances
    /// Firewall appliance
    Firewall,
    /// Intrusion detection/prevention system
    IDS,
    /// VPN concentrator
    VPNGateway,
    /// Web application firewall
    WAF,
    /// Security/surveillance camera
    Camera,

    // Storage Devices
    /// Storage array
    StorageArray,
    /// Network attached storage
    NAS,
    /// Storage area network switch
    SANSwitch,

    // Specialized Appliances
    /// Generic appliance (catch-all)
    Appliance,
    /// Backup appliance
    BackupAppliance,
    /// Monitoring appliance
    MonitoringAppliance,
    /// Authentication server
    AuthServer,
    /// KVM switch (keyboard-video-mouse)
    KVM,
    /// Display monitor
    Monitor,

    // Edge/IoT Devices
    /// Edge computing device
    EdgeDevice,
    /// IoT gateway
    IoTGateway,
    /// Sensor device
    Sensor,

    // Power/Environmental
    /// Power distribution unit
    PDU,
    /// UPS (uninterruptible power supply)
    UPS,
    /// Environmental monitoring
    EnvironmentalMonitor,

    // Telecommunications
    /// PBX/phone system
    PBX,
    /// Video conferencing system
    VideoConference,

    // Other/Unknown
    /// Other/uncategorized device
    Other,
    /// Unknown device type
    Unknown,
}

impl ResourceType {
    /// Get the canonical string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PhysicalServer => "physical_server",
            Self::VirtualMachine => "virtual_machine",
            Self::ContainerHost => "container_host",
            Self::Hypervisor => "hypervisor",
            Self::Router => "router",
            Self::Switch => "switch",
            Self::Layer3Switch => "layer3_switch",
            Self::AccessPoint => "access_point",
            Self::LoadBalancer => "load_balancer",
            Self::Firewall => "firewall",
            Self::IDS => "ids",
            Self::VPNGateway => "vpn_gateway",
            Self::WAF => "waf",
            Self::Camera => "camera",
            Self::StorageArray => "storage_array",
            Self::NAS => "nas",
            Self::SANSwitch => "san_switch",
            Self::Appliance => "appliance",
            Self::BackupAppliance => "backup_appliance",
            Self::MonitoringAppliance => "monitoring_appliance",
            Self::AuthServer => "auth_server",
            Self::KVM => "kvm",
            Self::Monitor => "monitor",
            Self::EdgeDevice => "edge_device",
            Self::IoTGateway => "iot_gateway",
            Self::Sensor => "sensor",
            Self::PDU => "pdu",
            Self::UPS => "ups",
            Self::EnvironmentalMonitor => "environmental_monitor",
            Self::PBX => "pbx",
            Self::VideoConference => "video_conference",
            Self::Other => "other",
            Self::Unknown => "unknown",
        }
    }

    /// Parse from string representation
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "physical_server" | "server" | "bare_metal" => Self::PhysicalServer,
            "virtual_machine" | "vm" => Self::VirtualMachine,
            "container_host" | "container" => Self::ContainerHost,
            "hypervisor" | "host" => Self::Hypervisor,
            "router" => Self::Router,
            "switch" => Self::Switch,
            "layer3_switch" | "l3_switch" | "multilayer_switch" => Self::Layer3Switch,
            "access_point" | "ap" | "wap" => Self::AccessPoint,
            "load_balancer" | "lb" | "balancer" => Self::LoadBalancer,
            "firewall" | "fw" => Self::Firewall,
            "ids" | "ips" | "intrusion_detection" => Self::IDS,
            "vpn_gateway" | "vpn" => Self::VPNGateway,
            "waf" | "web_firewall" => Self::WAF,
            "camera" | "surveillance" | "ip_camera" | "cctv" => Self::Camera,
            "storage_array" | "storage" => Self::StorageArray,
            "nas" | "network_storage" => Self::NAS,
            "san_switch" | "san" => Self::SANSwitch,
            "appliance" => Self::Appliance,
            "backup_appliance" | "backup" => Self::BackupAppliance,
            "monitoring_appliance" | "monitoring" => Self::MonitoringAppliance,
            "auth_server" | "authentication" | "ldap" | "active_directory" => Self::AuthServer,
            "kvm" | "kvm_switch" => Self::KVM,
            "monitor" | "display" | "screen" => Self::Monitor,
            "edge_device" | "edge" => Self::EdgeDevice,
            "iot_gateway" | "iot" => Self::IoTGateway,
            "sensor" => Self::Sensor,
            "pdu" | "power_distribution" => Self::PDU,
            "ups" | "battery" => Self::UPS,
            "environmental_monitor" | "environmental" => Self::EnvironmentalMonitor,
            "pbx" | "phone_system" => Self::PBX,
            "video_conference" | "video" | "conferencing" => Self::VideoConference,
            "other" => Self::Other,
            _ => Self::Unknown,
        }
    }

    /// Get human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PhysicalServer => "Physical Server",
            Self::VirtualMachine => "Virtual Machine",
            Self::ContainerHost => "Container Host",
            Self::Hypervisor => "Hypervisor",
            Self::Router => "Router",
            Self::Switch => "Switch",
            Self::Layer3Switch => "Layer 3 Switch",
            Self::AccessPoint => "Access Point",
            Self::LoadBalancer => "Load Balancer",
            Self::Firewall => "Firewall",
            Self::IDS => "IDS/IPS",
            Self::VPNGateway => "VPN Gateway",
            Self::WAF => "Web Application Firewall",
            Self::StorageArray => "Storage Array",
            Self::NAS => "Network Attached Storage",
            Self::SANSwitch => "SAN Switch",
            Self::Camera => "Security Camera",
            Self::Appliance => "Appliance",
            Self::BackupAppliance => "Backup Appliance",
            Self::MonitoringAppliance => "Monitoring Appliance",
            Self::AuthServer => "Authentication Server",
            Self::KVM => "KVM Switch",
            Self::Monitor => "Display Monitor",
            Self::EdgeDevice => "Edge Device",
            Self::IoTGateway => "IoT Gateway",
            Self::Sensor => "Sensor",
            Self::PDU => "Power Distribution Unit",
            Self::UPS => "UPS",
            Self::EnvironmentalMonitor => "Environmental Monitor",
            Self::PBX => "PBX/Phone System",
            Self::VideoConference => "Video Conference System",
            Self::Other => "Other",
            Self::Unknown => "Unknown",
        }
    }

    /// Get the primary category for this resource type
    pub fn category(&self) -> ResourceCategory {
        match self {
            Self::PhysicalServer
            | Self::VirtualMachine
            | Self::ContainerHost
            | Self::Hypervisor => ResourceCategory::Compute,

            Self::Router
            | Self::Switch
            | Self::Layer3Switch
            | Self::AccessPoint
            | Self::LoadBalancer => ResourceCategory::Network,

            Self::Firewall
            | Self::IDS
            | Self::VPNGateway
            | Self::WAF
            | Self::Camera => ResourceCategory::Security,

            Self::StorageArray
            | Self::NAS
            | Self::SANSwitch => ResourceCategory::Storage,

            Self::EdgeDevice
            | Self::IoTGateway
            | Self::Sensor => ResourceCategory::Edge,

            Self::PDU
            | Self::UPS
            | Self::EnvironmentalMonitor => ResourceCategory::Power,

            Self::PBX
            | Self::VideoConference => ResourceCategory::Telecom,

            Self::Appliance
            | Self::BackupAppliance
            | Self::MonitoringAppliance
            | Self::AuthServer
            | Self::KVM
            | Self::Monitor => ResourceCategory::Appliance,

            Self::Other
            | Self::Unknown => ResourceCategory::Other,
        }
    }

    /// Get NetBox device role color (hex color code)
    pub fn netbox_color(&self) -> &'static str {
        match self.category() {
            ResourceCategory::Compute => "4caf50",    // Green
            ResourceCategory::Network => "2196f3",    // Blue
            ResourceCategory::Security => "f44336",   // Red
            ResourceCategory::Storage => "ff9800",    // Orange
            ResourceCategory::Edge => "9c27b0",       // Purple
            ResourceCategory::Power => "ffeb3b",      // Yellow
            ResourceCategory::Telecom => "00bcd4",    // Cyan
            ResourceCategory::Appliance => "795548",  // Brown
            ResourceCategory::Other => "9e9e9e",      // Grey
        }
    }

    /// Check if this is a network device (router, switch, etc.)
    pub fn is_network_device(&self) -> bool {
        matches!(
            self,
            Self::Router
                | Self::Switch
                | Self::Layer3Switch
                | Self::AccessPoint
                | Self::LoadBalancer
        )
    }

    /// Check if this is a security device
    pub fn is_security_device(&self) -> bool {
        matches!(self, Self::Firewall | Self::IDS | Self::VPNGateway | Self::WAF)
    }

    /// Check if this is a compute resource
    pub fn is_compute_resource(&self) -> bool {
        matches!(
            self,
            Self::PhysicalServer
                | Self::VirtualMachine
                | Self::ContainerHost
                | Self::Hypervisor
        )
    }
}

impl Default for ResourceType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<&str> for ResourceType {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for ResourceType {
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}

/// Resource category (high-level grouping)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceCategory {
    /// Compute resources (servers, VMs, etc.)
    Compute,
    /// Network infrastructure (routers, switches, etc.)
    Network,
    /// Security appliances (firewalls, IDS, etc.)
    Security,
    /// Storage devices (arrays, NAS, etc.)
    Storage,
    /// Edge/IoT devices
    Edge,
    /// Power/environmental systems
    Power,
    /// Telecommunications equipment
    Telecom,
    /// Appliances (general purpose)
    Appliance,
    /// Other/unknown
    Other,
}

impl fmt::Display for ResourceCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compute => write!(f, "Compute"),
            Self::Network => write!(f, "Network"),
            Self::Security => write!(f, "Security"),
            Self::Storage => write!(f, "Storage"),
            Self::Edge => write!(f, "Edge/IoT"),
            Self::Power => write!(f, "Power/Environmental"),
            Self::Telecom => write!(f, "Telecommunications"),
            Self::Appliance => write!(f, "Appliance"),
            Self::Other => write!(f, "Other"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_parsing() {
        assert_eq!(ResourceType::from_str("router"), ResourceType::Router);
        assert_eq!(ResourceType::from_str("switch"), ResourceType::Switch);
        assert_eq!(ResourceType::from_str("server"), ResourceType::PhysicalServer);
        assert_eq!(ResourceType::from_str("vm"), ResourceType::VirtualMachine);
        assert_eq!(ResourceType::from_str("firewall"), ResourceType::Firewall);
        assert_eq!(ResourceType::from_str("unknown_device"), ResourceType::Unknown);
    }

    #[test]
    fn test_resource_categories() {
        assert_eq!(ResourceType::Router.category(), ResourceCategory::Network);
        assert_eq!(ResourceType::PhysicalServer.category(), ResourceCategory::Compute);
        assert_eq!(ResourceType::Firewall.category(), ResourceCategory::Security);
        assert_eq!(ResourceType::StorageArray.category(), ResourceCategory::Storage);
    }

    #[test]
    fn test_is_network_device() {
        assert!(ResourceType::Router.is_network_device());
        assert!(ResourceType::Switch.is_network_device());
        assert!(!ResourceType::PhysicalServer.is_network_device());
        assert!(!ResourceType::Firewall.is_network_device());
    }

    #[test]
    fn test_display_name() {
        assert_eq!(ResourceType::Router.display_name(), "Router");
        assert_eq!(ResourceType::Layer3Switch.display_name(), "Layer 3 Switch");
        assert_eq!(ResourceType::VPNGateway.display_name(), "VPN Gateway");
        assert_eq!(ResourceType::Camera.display_name(), "Security Camera");
        assert_eq!(ResourceType::KVM.display_name(), "KVM Switch");
        assert_eq!(ResourceType::Monitor.display_name(), "Display Monitor");
    }

    #[test]
    fn test_new_device_types() {
        // Test Camera
        assert_eq!(ResourceType::from_str("camera"), ResourceType::Camera);
        assert_eq!(ResourceType::from_str("surveillance"), ResourceType::Camera);
        assert_eq!(ResourceType::from_str("cctv"), ResourceType::Camera);
        assert_eq!(ResourceType::Camera.category(), ResourceCategory::Security);
        assert_eq!(ResourceType::Camera.as_str(), "camera");

        // Test KVM
        assert_eq!(ResourceType::from_str("kvm"), ResourceType::KVM);
        assert_eq!(ResourceType::from_str("kvm_switch"), ResourceType::KVM);
        assert_eq!(ResourceType::KVM.category(), ResourceCategory::Appliance);
        assert_eq!(ResourceType::KVM.as_str(), "kvm");

        // Test Monitor
        assert_eq!(ResourceType::from_str("monitor"), ResourceType::Monitor);
        assert_eq!(ResourceType::from_str("display"), ResourceType::Monitor);
        assert_eq!(ResourceType::from_str("screen"), ResourceType::Monitor);
        assert_eq!(ResourceType::Monitor.category(), ResourceCategory::Appliance);
        assert_eq!(ResourceType::Monitor.as_str(), "monitor");
    }
}
