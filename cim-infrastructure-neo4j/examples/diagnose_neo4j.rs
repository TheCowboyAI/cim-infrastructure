// Copyright 2025 Cowboy AI, LLC.

//! Neo4j connection diagnostics
//!
//! Run with:
//! ```bash
//! cargo run --example diagnose_neo4j
//! ```

use neo4rs::{Graph, Query};

async fn test_connection(uri: &str, user: &str, password: &str) -> bool {
    println!("  Testing: {}@{}", user, uri);

    match Graph::new(uri, user, password).await {
        Ok(graph) => {
            let query = Query::new("RETURN 1 as test".to_string());
            match graph.execute(query).await {
                Ok(_) => {
                    println!("  ✅ Success!\n");
                    true
                }
                Err(e) => {
                    println!("  ❌ Query failed: {}\n", e);
                    false
                }
            }
        }
        Err(e) => {
            println!("  ❌ Connection failed: {}\n", e);
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Neo4j Connection Diagnostics");
    println!("================================\n");

    // Common configurations to try
    let configs = vec![
        ("bolt://localhost:7687", "", ""),  // No auth
        ("bolt://localhost:7687", "neo4j", ""),  // Default user, no password
        ("bolt://localhost:7687", "cim", "cim"),
        ("bolt://localhost:7687", "neo4j", "password"),
        ("bolt://localhost:7687", "neo4j", "neo4j"),
        ("bolt://127.0.0.1:7687", "", ""),  // No auth
        ("bolt://127.0.0.1:7687", "cim", "cim"),
        ("neo4j://localhost:7687", "", ""),  // No auth
    ];

    println!("Trying common configurations...\n");

    let mut success = false;
    for (uri, user, password) in configs {
        if test_connection(uri, user, password).await {
            success = true;
            println!("🎉 Working configuration found!");
            println!("   URI: {}", uri);
            println!("   User: {}", user);
            println!("   Password: {}", password);
            break;
        }
    }

    if !success {
        println!("❌ No working configuration found\n");
        println!("💡 Please verify:");
        println!("   1. Neo4j is running:");
        println!("      docker ps | grep neo4j");
        println!("\n   2. Check Neo4j logs:");
        println!("      docker logs neo4j");
        println!("\n   3. Try Neo4j Browser:");
        println!("      http://localhost:7474");
        println!("\n   4. Start fresh Neo4j:");
        println!("      docker run -d --name neo4j \\");
        println!("        -p 7474:7474 -p 7687:7687 \\");
        println!("        -e NEO4J_AUTH=none \\");
        println!("        neo4j:latest");
    }

    Ok(())
}
