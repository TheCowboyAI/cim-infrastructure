// Copyright (c) 2025 - Cowboy AI, Inc.

//! NetBox DCIM Projection Adapter
//!
//! Implements the ProjectionAdapter Functor for projecting infrastructure events
//! into NetBox (Network Source of Truth).
//!
//! NetBox is a widely-used Data Center Infrastructure Management (DCIM) tool that
//! models infrastructure assets including devices, IP addresses, cables, and more.
//!
//! # Architecture
//!
//! This adapter projects infrastructure events into NetBox via its REST API:
//!
//! ```text
//! F: InfrastructureEvents → NetBox DCIM
//!
//! F(ComputeRegistered) = POST /api/dcim/devices/
//! F(NetworkDefined) = POST /api/ipam/prefixes/
//! F(ConnectionEstablished) = POST /api/dcim/cables/
//! ```
//!
//! # NetBox Data Model
//!
//! - **Devices**: Physical servers, VMs, network equipment
//! - **Interfaces**: Network interfaces on devices
//! - **IP Addresses**: IPs assigned to interfaces
//! - **Prefixes**: Network segments (CIDR blocks)
//! - **Cables**: Physical connections between interfaces
//! - **Sites**: Physical locations
//! - **Racks**: Equipment racks
//!
//! # Example
//!
//! ```rust,no_run
//! use cim_infrastructure::adapters::NetBoxProjectionAdapter;
//! use cim_infrastructure::projection::ProjectionAdapter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = NetBoxConfig {
//!         base_url: "http://10.0.224.131".to_string(),
//!         api_token: "your-token-here".to_string(),
//!         default_site_id: Some(1),
//!     };
//!
//!     let mut projection = NetBoxProjectionAdapter::new(config).await?;
//!     projection.initialize().await?;
//!
//!     // Project events...
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::domain::ResourceType;
use crate::projection::{ProjectionAdapter, ProjectionError};

/// Configuration for NetBox connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxConfig {
    /// NetBox base URL (e.g., "http://10.0.224.131")
    pub base_url: String,

    /// API token for authentication
    pub api_token: String,

    /// Default site ID for devices (if not specified in events)
    pub default_site_id: Option<i32>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

impl Default for NetBoxConfig {
    fn default() -> Self {
        Self {
            base_url: "http://10.0.224.131".to_string(),
            api_token: String::new(),
            default_site_id: Some(1),
            timeout_secs: 30,
        }
    }
}

/// NetBox device representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxDevice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub name: String,
    pub device_type: i32,
    pub device_role: i32,
    pub site: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// NetBox interface representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxInterface {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub device: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub interface_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// NetBox IP address representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxIPAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub address: String, // CIDR format: "192.168.1.10/24"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// NetBox prefix (network segment) representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxPrefix {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub prefix: String, // CIDR format: "192.168.1.0/24"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Infrastructure event type for NetBox projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureEvent {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub data: serde_json::Value,
}

/// NetBox projection adapter implementing the Functor F: Events → NetBox
pub struct NetBoxProjectionAdapter {
    config: NetBoxConfig,
    client: Client,
}

impl NetBoxProjectionAdapter {
    /// Create a new NetBox projection adapter
    pub async fn new(config: NetBoxConfig) -> Result<Self, ProjectionError> {
        info!("Connecting to NetBox at {}", config.base_url);

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "Authorization",
                    format!("Token {}", config.api_token)
                        .parse()
                        .map_err(|e| {
                            ProjectionError::TargetUnavailable(format!(
                                "Invalid API token: {}",
                                e
                            ))
                        })?,
                );
                headers.insert(
                    "Content-Type",
                    "application/json".parse().map_err(|e| {
                        ProjectionError::TargetUnavailable(format!("Invalid header: {}", e))
                    })?,
                );
                headers
            })
            .build()
            .map_err(|e| {
                ProjectionError::TargetUnavailable(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { config, client })
    }

    /// Get or create a device type in NetBox
    async fn get_or_create_device_type(
        &self,
        manufacturer: &str,
        model: &str,
    ) -> Result<i32, ProjectionError> {
        let url = format!("{}/api/dcim/device-types/", self.config.base_url);

        // Search for existing device type
        let search_url = format!("{}?model={}", url, urlencoding::encode(model));
        let response = self.client.get(&search_url).send().await
            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to search device types: {}", e)))?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await
                .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

            if let Some(results) = data["results"].as_array() {
                if !results.is_empty() {
                    if let Some(id) = results[0]["id"].as_i64() {
                        debug!("Found existing device type: {} (id: {})", model, id);
                        return Ok(id as i32);
                    }
                }
            }
        }

        // Create new device type
        warn!("Device type '{}' not found, creating placeholder", model);
        let device_type = serde_json::json!({
            "manufacturer": {"name": manufacturer, "slug": manufacturer.to_lowercase().replace(" ", "-")},
            "model": model,
            "slug": model.to_lowercase().replace(" ", "-")
        });

        let response = self.client.post(&url).json(&device_type).send().await
            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to create device type: {}", e)))?;

        if response.status() == StatusCode::CREATED || response.status() == StatusCode::OK {
            let data: serde_json::Value = response.json().await
                .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

            if let Some(id) = data["id"].as_i64() {
                info!("Created device type: {} (id: {})", model, id);
                return Ok(id as i32);
            }
        }

        Err(ProjectionError::DatabaseError(
            "Failed to get or create device type".to_string()
        ))
    }

    /// Get or create a device role in NetBox based on resource type
    ///
    /// Uses the domain ResourceType taxonomy to determine role name, slug, and color.
    /// This ensures consistent categorization across all infrastructure resources.
    async fn get_or_create_device_role(
        &self,
        resource_type: ResourceType,
    ) -> Result<i32, ProjectionError> {
        let url = format!("{}/api/dcim/device-roles/", self.config.base_url);
        let role_name = resource_type.display_name();
        let slug = resource_type.as_str().replace('_', "-");
        let color = resource_type.netbox_color();

        // Search for existing role by name
        let search_url = format!("{}?name={}", url, urlencoding::encode(role_name));
        let response = self.client.get(&search_url).send().await
            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to search device roles: {}", e)))?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await
                .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

            if let Some(results) = data["results"].as_array() {
                if !results.is_empty() {
                    if let Some(id) = results[0]["id"].as_i64() {
                        debug!(
                            "Found existing device role: {} ({}) (id: {})",
                            role_name,
                            resource_type.category(),
                            id
                        );
                        return Ok(id as i32);
                    }
                }
            }
        }

        // Create new device role with taxonomy metadata
        info!(
            "Device role '{}' not found, creating with color #{} (category: {})",
            role_name, color, resource_type.category()
        );
        let device_role = serde_json::json!({
            "name": role_name,
            "slug": slug,
            "color": color,
            "description": format!(
                "{} - Auto-created from CIM ResourceType taxonomy",
                resource_type.category()
            )
        });

        let response = self.client.post(&url).json(&device_role).send().await
            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to create device role: {}", e)))?;

        if response.status() == StatusCode::CREATED || response.status() == StatusCode::OK {
            let data: serde_json::Value = response.json().await
                .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

            if let Some(id) = data["id"].as_i64() {
                info!(
                    "Created device role: {} ({}) (id: {})",
                    role_name,
                    resource_type.category(),
                    id
                );
                return Ok(id as i32);
            }
        }

        Err(ProjectionError::DatabaseError(
            "Failed to get or create device role".to_string()
        ))
    }

    /// Check if device already exists by name (idempotency)
    async fn device_exists(&self, hostname: &str) -> Result<Option<i32>, ProjectionError> {
        let url = format!(
            "{}/api/dcim/devices/?name={}",
            self.config.base_url,
            urlencoding::encode(hostname)
        );

        let response = self.client.get(&url).send().await
            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to check device existence: {}", e)))?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await
                .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

            if let Some(results) = data["results"].as_array() {
                if !results.is_empty() {
                    if let Some(id) = results[0]["id"].as_i64() {
                        return Ok(Some(id as i32));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Project a compute resource registered event
    async fn project_compute_registered(
        &self,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        let hostname = data["hostname"]
            .as_str()
            .ok_or_else(|| ProjectionError::InvalidEvent("Missing 'hostname'".to_string()))?;

        // Check idempotency - device already exists?
        if let Some(device_id) = self.device_exists(hostname).await? {
            info!("Device '{}' already exists (id: {}), skipping", hostname, device_id);
            return Ok(());
        }

        // Parse resource type from domain taxonomy
        let resource_type = data["resource_type"]
            .as_str()
            .map(ResourceType::from_str)
            .unwrap_or(ResourceType::Unknown);

        let manufacturer = data["manufacturer"].as_str().unwrap_or("Generic");
        let model = data["model"].as_str().unwrap_or("Generic Server");

        // Get or create device type and role using domain taxonomy
        let device_type_id = self.get_or_create_device_type(manufacturer, model).await?;
        let device_role_id = self.get_or_create_device_role(resource_type).await?;

        let device = NetBoxDevice {
            id: None,
            name: hostname.to_string(),
            device_type: device_type_id,
            device_role: device_role_id,
            site: self.config.default_site_id.unwrap_or(1),
            status: Some("active".to_string()),
            comments: Some(format!("Created from CIM event - type: {}", resource_type)),
            custom_fields: Some(serde_json::json!({
                "cim_aggregate_id": data["id"],
            })),
        };

        let url = format!("{}/api/dcim/devices/", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .json(&device)
            .send()
            .await
            .map_err(|e| ProjectionError::DatabaseError(format!("NetBox API error: {}", e)))?;

        if response.status() == StatusCode::CREATED || response.status() == StatusCode::OK {
            info!("Projected ComputeRegistered to NetBox: {} (type: {}, role: {})",
                  hostname, device_type_id, device_role_id);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "".to_string());
            Err(ProjectionError::DatabaseError(format!(
                "NetBox API returned {}: {}",
                status, body
            )))
        }
    }

    /// Project a network defined event
    async fn project_network_defined(
        &self,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        let cidr = data["cidr"]
            .as_str()
            .ok_or_else(|| ProjectionError::InvalidEvent("Missing 'cidr'".to_string()))?;

        let name = data["name"].as_str().unwrap_or("unnamed");

        // Check idempotency - prefix already exists?
        let search_url = format!(
            "{}/api/ipam/prefixes/?prefix={}",
            self.config.base_url,
            urlencoding::encode(cidr)
        );
        let response = self.client.get(&search_url).send().await
            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to check prefix existence: {}", e)))?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await
                .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

            if let Some(results) = data["results"].as_array() {
                if !results.is_empty() {
                    info!("Prefix '{}' already exists, skipping", cidr);
                    return Ok(());
                }
            }
        }

        let prefix = NetBoxPrefix {
            id: None,
            prefix: cidr.to_string(),
            site: self.config.default_site_id,
            status: Some("active".to_string()),
            description: Some(format!("CIM Network: {}", name)),
        };

        let url = format!("{}/api/ipam/prefixes/", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .json(&prefix)
            .send()
            .await
            .map_err(|e| ProjectionError::DatabaseError(format!("NetBox API error: {}", e)))?;

        if response.status() == StatusCode::CREATED || response.status() == StatusCode::OK {
            info!("Projected NetworkDefined to NetBox: {}", cidr);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "".to_string());
            Err(ProjectionError::DatabaseError(format!(
                "NetBox API returned {}: {}",
                status, body
            )))
        }
    }

    /// Project an interface added event
    async fn project_interface_added(
        &self,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        let device_name = data["device"]
            .as_str()
            .ok_or_else(|| ProjectionError::InvalidEvent("Missing 'device'".to_string()))?;

        let interface_name = data["name"]
            .as_str()
            .ok_or_else(|| ProjectionError::InvalidEvent("Missing 'name'".to_string()))?;

        // Look up device ID by name
        let device_id = self.device_exists(device_name).await?
            .ok_or_else(|| ProjectionError::InvalidEvent(
                format!("Device '{}' not found in NetBox", device_name)
            ))?;

        // Check idempotency - interface already exists?
        let search_url = format!(
            "{}/api/dcim/interfaces/?device_id={}&name={}",
            self.config.base_url,
            device_id,
            urlencoding::encode(interface_name)
        );
        let response = self.client.get(&search_url).send().await
            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to check interface existence: {}", e)))?;

        if response.status().is_success() {
            let check_data: serde_json::Value = response.json().await
                .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

            if let Some(results) = check_data["results"].as_array() {
                if !results.is_empty() {
                    info!("Interface '{}' on device '{}' already exists, skipping",
                          interface_name, device_name);
                    return Ok(());
                }
            }
        }

        let interface_type = data["type"].as_str().unwrap_or("1000base-t");
        let mac_address = data["mac_address"].as_str().map(|s| s.to_string());
        let mtu = data["mtu"].as_i64().map(|m| m as i32);

        let interface = NetBoxInterface {
            id: None,
            device: device_id,
            name: interface_name.to_string(),
            interface_type: interface_type.to_string(),
            enabled: Some(true),
            mtu,
            mac_address,
            description: data["description"].as_str().map(|s| s.to_string()),
        };

        let url = format!("{}/api/dcim/interfaces/", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .json(&interface)
            .send()
            .await
            .map_err(|e| ProjectionError::DatabaseError(format!("NetBox API error: {}", e)))?;

        if response.status() == StatusCode::CREATED || response.status() == StatusCode::OK {
            info!("Projected InterfaceAdded to NetBox: {} on {}",
                  interface_name, device_name);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "".to_string());
            Err(ProjectionError::DatabaseError(format!(
                "NetBox API returned {}: {}",
                status, body
            )))
        }
    }

    /// Project an IP assigned event
    async fn project_ip_assigned(
        &self,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        let address = data["address"]
            .as_str()
            .ok_or_else(|| ProjectionError::InvalidEvent("Missing 'address'".to_string()))?;

        // Check idempotency - IP already exists?
        let search_url = format!(
            "{}/api/ipam/ip-addresses/?address={}",
            self.config.base_url,
            urlencoding::encode(address)
        );
        let response = self.client.get(&search_url).send().await
            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to check IP existence: {}", e)))?;

        if response.status().is_success() {
            let check_data: serde_json::Value = response.json().await
                .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

            if let Some(results) = check_data["results"].as_array() {
                if !results.is_empty() {
                    info!("IP address '{}' already exists, skipping", address);
                    return Ok(());
                }
            }
        }

        // Optional: Look up interface if specified
        let (assigned_object_type, assigned_object_id) = if let Some(interface_name) = data["interface"].as_str() {
            if let Some(device_name) = data["device"].as_str() {
                if let Some(device_id) = self.device_exists(device_name).await? {
                    let iface_url = format!(
                        "{}/api/dcim/interfaces/?device_id={}&name={}",
                        self.config.base_url,
                        device_id,
                        urlencoding::encode(interface_name)
                    );
                    let iface_response = self.client.get(&iface_url).send().await
                        .map_err(|e| ProjectionError::DatabaseError(format!("Failed to find interface: {}", e)))?;

                    if iface_response.status().is_success() {
                        let iface_data: serde_json::Value = iface_response.json().await
                            .map_err(|e| ProjectionError::DatabaseError(format!("Failed to parse response: {}", e)))?;

                        if let Some(results) = iface_data["results"].as_array() {
                            if !results.is_empty() {
                                if let Some(iface_id) = results[0]["id"].as_i64() {
                                    (Some("dcim.interface".to_string()), Some(iface_id as i32))
                                } else {
                                    (None, None)
                                }
                            } else {
                                (None, None)
                            }
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let ip_address = NetBoxIPAddress {
            id: None,
            address: address.to_string(),
            status: Some("active".to_string()),
            assigned_object_type,
            assigned_object_id,
            description: data["description"].as_str().map(|s| s.to_string()),
        };

        let url = format!("{}/api/ipam/ip-addresses/", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .json(&ip_address)
            .send()
            .await
            .map_err(|e| ProjectionError::DatabaseError(format!("NetBox API error: {}", e)))?;

        if response.status() == StatusCode::CREATED || response.status() == StatusCode::OK {
            info!("Projected IPAssigned to NetBox: {}", address);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "".to_string());
            Err(ProjectionError::DatabaseError(format!(
                "NetBox API returned {}: {}",
                status, body
            )))
        }
    }
}

#[async_trait]
impl ProjectionAdapter for NetBoxProjectionAdapter {
    type Event = InfrastructureEvent;
    type Error = ProjectionError;

    async fn project(&mut self, event: Self::Event) -> Result<(), Self::Error> {
        debug!(
            "Projecting event to NetBox: {} ({})",
            event.event_type, event.event_id
        );

        // Route events to specific projection handlers
        match event.event_type.as_str() {
            "ComputeRegistered" | "compute.registered" => {
                self.project_compute_registered(&event.data).await?
            }
            "NetworkDefined" | "network.defined" => {
                self.project_network_defined(&event.data).await?
            }
            "InterfaceAdded" | "interface.added" => {
                self.project_interface_added(&event.data).await?
            }
            "IPAssigned" | "ip.assigned" => {
                self.project_ip_assigned(&event.data).await?
            }
            unknown => {
                warn!("Unknown event type for NetBox projection: {}", unknown);
                // Don't fail on unknown events - allows graceful evolution
            }
        }

        Ok(())
    }

    async fn initialize(&mut self) -> Result<(), Self::Error> {
        info!("Initializing NetBox projection adapter");

        // Verify connectivity by checking API status
        self.health_check().await?;

        info!("NetBox projection adapter initialized successfully");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Self::Error> {
        let url = format!("{}/api/status/", self.config.base_url);
        let response = self.client.get(&url).send().await.map_err(|e| {
            ProjectionError::TargetUnavailable(format!("NetBox health check failed: {}", e))
        })?;

        if response.status().is_success() {
            debug!("NetBox health check passed");
            Ok(())
        } else {
            Err(ProjectionError::TargetUnavailable(format!(
                "NetBox returned status: {}",
                response.status()
            )))
        }
    }

    async fn reset(&mut self) -> Result<(), Self::Error> {
        error!("NetBox projection reset is not supported - would require deleting all NetBox data");
        Err(ProjectionError::Other(
            "Reset not supported for NetBox projection".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "netbox-dcim-projection"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = NetBoxConfig::default();
        assert_eq!(config.base_url, "http://10.0.224.131");
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_infrastructure_event_creation() {
        let event = InfrastructureEvent {
            event_id: Uuid::now_v7(),
            aggregate_id: Uuid::now_v7(),
            event_type: "ComputeRegistered".to_string(),
            data: serde_json::json!({
                "id": "server-1",
                "hostname": "web01.example.com",
                "resource_type": "physical_server"
            }),
        };

        assert_eq!(event.event_type, "ComputeRegistered");
        assert!(event.data["hostname"].is_string());
    }

    #[test]
    fn test_netbox_device_serialization() {
        let device = NetBoxDevice {
            id: None,
            name: "test-server".to_string(),
            device_type: 1,
            device_role: 1,
            site: 1,
            status: Some("active".to_string()),
            comments: Some("Test device".to_string()),
            custom_fields: None,
        };

        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("test-server"));
        assert!(json.contains("active"));
    }
}
