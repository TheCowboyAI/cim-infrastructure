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

    /// Project a compute resource registered event
    async fn project_compute_registered(
        &self,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        let hostname = data["hostname"]
            .as_str()
            .ok_or_else(|| ProjectionError::InvalidEvent("Missing 'hostname'".to_string()))?;

        let resource_type = data["resource_type"].as_str().unwrap_or("server");

        // In real implementation, we'd need to:
        // 1. Look up or create device_type ID
        // 2. Look up or create device_role ID
        // 3. Use configured site ID

        let device = NetBoxDevice {
            id: None,
            name: hostname.to_string(),
            device_type: 1, // Placeholder - needs lookup
            device_role: 1, // Placeholder - needs lookup
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
            debug!("Projected ComputeRegistered to NetBox: {}", hostname);
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
            debug!("Projected NetworkDefined to NetBox: {}", cidr);
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
