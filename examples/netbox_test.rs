// NetBox Projection Adapter Test Example
//
// Run with: cargo run --example netbox_test --features netbox
//
// Prerequisites:
// 1. Source secrets: source ~/.secrets/cim-env.sh
// 2. Or set: export NETBOX_API_TOKEN="your-token"
// 3. NetBox running at http://10.0.224.131

use cim_infrastructure::adapters::{NetBoxConfig, NetBoxProjectionAdapter};
use cim_infrastructure::projection::ProjectionAdapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = NetBoxConfig {
        base_url: std::env::var("NETBOX_URL")
            .unwrap_or_else(|_| "http://10.0.224.131".to_string()),
        api_token: std::env::var("NETBOX_API_TOKEN")
            .expect("NETBOX_API_TOKEN not set. Run: source ~/.secrets/cim-env.sh"),
        default_site_id: Some(1),
        timeout_secs: 30,
    };

    println!("🔧 Connecting to NetBox at {}", config.base_url);

    // Create adapter
    let mut adapter = NetBoxProjectionAdapter::new(config).await?;

    println!("✅ Adapter created");

    // Initialize (no-op for NetBox, but required by trait)
    adapter.initialize().await?;
    println!("✅ Adapter initialized");

    // Health check
    match adapter.health_check().await {
        Ok(_) => println!("✅ NetBox is healthy and responding"),
        Err(e) => {
            eprintln!("❌ Health check failed: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🎉 NetBox projection adapter is ready!");
    println!("\nNext steps:");
    println!("  1. Configure device types in NetBox");
    println!("  2. Create at least one site");
    println!("  3. Test event projections");

    Ok(())
}
