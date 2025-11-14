// Copyright 2025 Cowboy AI, LLC.

//! Simple Neo4j connection test
//!
//! Run with:
//! ```bash
//! cargo run --example test_connection
//! ```

use cim_infrastructure_neo4j::Neo4jConfig;
use neo4rs::{Graph, Query};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Neo4j Connection");
    println!("============================\n");

    let config = Neo4jConfig {
        uri: "bolt://localhost:7687".to_string(),
        user: "".to_string(),  // No auth
        password: "".to_string(),
        database: None,
    };

    println!("📡 Connecting to: {}", config.uri);
    println!("👤 User: {}", config.user);
    println!("🗄️  Database: {}\n", config.database());

    // Try to connect
    match Graph::new(&config.uri, &config.user, &config.password).await {
        Ok(graph) => {
            println!("✅ Connection successful!\n");

            // Try a simple query
            println!("🔍 Running test query...");
            let query = Query::new("RETURN 1 as test".to_string());

            match graph.execute(query).await {
                Ok(mut result) => {
                    if let Ok(Some(row)) = result.next().await {
                        let test_value: i64 = row.get("test").unwrap_or(0);
                        println!("✅ Query successful! Result: {}\n", test_value);
                    }
                    println!("🎉 Neo4j is ready for use!");
                }
                Err(e) => {
                    println!("❌ Query failed: {}", e);
                    println!("\n💡 Tip: Make sure you're using the correct credentials");
                }
            }
        }
        Err(e) => {
            println!("❌ Connection failed: {}\n", e);
            println!("💡 Troubleshooting:");
            println!("   1. Is Neo4j running? Check with:");
            println!("      docker ps | grep neo4j");
            println!("\n   2. Start Neo4j with:");
            println!("      docker run -d --name neo4j \\");
            println!("        -p 7474:7474 -p 7687:7687 \\");
            println!("        -e NEO4J_AUTH=none \\");
            println!("        neo4j:latest");
            println!("\n   3. Or if using existing Neo4j, update credentials in:");
            println!("      cim-infrastructure-neo4j/examples/test_connection.rs");
            println!("\n   4. Neo4j Browser: http://localhost:7474");
        }
    }

    Ok(())
}
