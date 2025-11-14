// Copyright 2025 Cowboy AI, LLC.

//! Neo4j connection configuration

use serde::{Deserialize, Serialize};

/// Neo4j connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    /// Neo4j URI (e.g., "bolt://localhost:7687")
    pub uri: String,

    /// Username for authentication
    pub user: String,

    /// Password for authentication
    pub password: String,

    /// Optional database name (defaults to "neo4j")
    pub database: Option<String>,
}

impl Neo4jConfig {
    /// Create a new Neo4j configuration
    pub fn new(uri: String, user: String, password: String) -> Self {
        Self {
            uri,
            user,
            password,
            database: None,
        }
    }

    /// Set the database name
    pub fn with_database(mut self, database: String) -> Self {
        self.database = Some(database);
        self
    }

    /// Get the database name (defaults to "neo4j" if not set)
    pub fn database(&self) -> &str {
        self.database.as_deref().unwrap_or("neo4j")
    }
}

impl Default for Neo4jConfig {
    fn default() -> Self {
        Self {
            uri: "bolt://localhost:7687".to_string(),
            user: "".to_string(),  // No auth by default
            password: "".to_string(),
            database: None,
        }
    }
}
