// Copyright (c) 2025 - Cowboy AI, Inc.
//! Side Effect Executor
//!
//! This module provides executors that interpret side effects returned by
//! pure projections and perform the actual I/O operations.
//!
//! # Architecture
//!
//! ```text
//! Pure Projection              Executor
//! ────────────────            ──────────
//!
//! (State, Event)              Effects
//!      │                          │
//!      ▼                          ▼
//! ┌─────────────┐           ┌──────────────┐
//! │  project()  │  Effects  │  execute()   │
//! │ (pure func) │ ───────>  │  (async I/O) │
//! └─────────────┘           └──────────────┘
//!      │                          │
//!      ▼                          ▼
//! New State                  Side Effects
//!                            Performed
//! ```
//!
//! # Design Philosophy
//!
//! By separating pure projection logic from side effect execution:
//! 1. **Testability**: Projection logic can be tested without I/O
//! 2. **Flexibility**: Same projection can target different databases
//! 3. **Composability**: Effects can be batched, optimized, or transformed
//!
//! # Example
//!
//! ```rust,ignore
//! use cim_infrastructure::projection::executor::*;
//!
//! // Pure projection produces effects
//! let (new_state, effects) = my_projection(state, event);
//!
//! // Executor performs the effects
//! let mut executor = LoggingExecutor::new();
//! executor.execute(effects).await?;
//! ```

use super::pure::SideEffect;
use async_trait::async_trait;

/// Trait for executing side effects
///
/// Implementations interpret `SideEffect` data structures and perform
/// the actual I/O operations.
#[async_trait]
pub trait SideEffectExecutor: Send + Sync {
    /// Execute a batch of side effects
    ///
    /// Effects are executed in order. If any effect fails, the entire
    /// batch fails and returns an error.
    ///
    /// # Arguments
    ///
    /// * `effects` - Side effects to execute
    ///
    /// # Returns
    ///
    /// Ok(()) if all effects succeeded, Err otherwise
    async fn execute(&mut self, effects: Vec<SideEffect>) -> Result<(), ExecutorError>;

    /// Execute a single side effect
    ///
    /// Helper method for executing individual effects.
    ///
    /// # Arguments
    ///
    /// * `effect` - Side effect to execute
    ///
    /// # Returns
    ///
    /// Ok(()) if effect succeeded, Err otherwise
    async fn execute_one(&mut self, effect: SideEffect) -> Result<(), ExecutorError> {
        self.execute(vec![effect]).await
    }
}

/// Errors that can occur during side effect execution
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    /// Database operation failed
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Effect is not supported by this executor
    #[error("Unsupported effect: {0}")]
    UnsupportedEffect(String),

    /// Effect execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

/// Logging executor - logs effects but doesn't perform them
///
/// Useful for testing and debugging. Logs each effect instead of executing it.
#[derive(Debug, Clone)]
pub struct LoggingExecutor {
    /// Effects that have been logged
    pub logged_effects: Vec<SideEffect>,
}

impl LoggingExecutor {
    /// Create a new logging executor
    pub fn new() -> Self {
        Self {
            logged_effects: Vec::new(),
        }
    }

    /// Get all logged effects
    pub fn effects(&self) -> &[SideEffect] {
        &self.logged_effects
    }

    /// Clear logged effects
    pub fn clear(&mut self) {
        self.logged_effects.clear();
    }
}

impl Default for LoggingExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SideEffectExecutor for LoggingExecutor {
    async fn execute(&mut self, effects: Vec<SideEffect>) -> Result<(), ExecutorError> {
        for effect in effects {
            println!("[LoggingExecutor] Effect: {:?}", effect);
            self.logged_effects.push(effect);
        }
        Ok(())
    }
}

/// Null executor - discards all effects
///
/// Useful when you only care about the projection state, not the side effects.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullExecutor;

impl NullExecutor {
    /// Create a new null executor
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SideEffectExecutor for NullExecutor {
    async fn execute(&mut self, _effects: Vec<SideEffect>) -> Result<(), ExecutorError> {
        // Discard all effects
        Ok(())
    }
}

/// Collecting executor - collects effects for later execution
///
/// Useful for batching effects or deferring execution.
#[derive(Debug, Clone)]
pub struct CollectingExecutor {
    /// Collected effects
    pub collected: Vec<SideEffect>,
}

impl CollectingExecutor {
    /// Create a new collecting executor
    pub fn new() -> Self {
        Self {
            collected: Vec::new(),
        }
    }

    /// Get all collected effects
    pub fn effects(&self) -> &[SideEffect] {
        &self.collected
    }

    /// Take all collected effects, leaving the collector empty
    pub fn take_effects(&mut self) -> Vec<SideEffect> {
        std::mem::take(&mut self.collected)
    }

    /// Clear collected effects
    pub fn clear(&mut self) {
        self.collected.clear();
    }
}

impl Default for CollectingExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SideEffectExecutor for CollectingExecutor {
    async fn execute(&mut self, mut effects: Vec<SideEffect>) -> Result<(), ExecutorError> {
        self.collected.append(&mut effects);
        Ok(())
    }
}

/// Filtering executor - wraps another executor and filters effects
///
/// Useful for selectively executing certain types of effects.
pub struct FilteringExecutor<E: SideEffectExecutor> {
    inner: E,
    filter: Box<dyn Fn(&SideEffect) -> bool + Send + Sync>,
}

impl<E: SideEffectExecutor> FilteringExecutor<E> {
    /// Create a new filtering executor
    ///
    /// # Arguments
    ///
    /// * `inner` - Underlying executor
    /// * `filter` - Predicate to determine which effects to execute
    pub fn new<F>(inner: E, filter: F) -> Self
    where
        F: Fn(&SideEffect) -> bool + Send + Sync + 'static,
    {
        Self {
            inner,
            filter: Box::new(filter),
        }
    }
}

#[async_trait]
impl<E: SideEffectExecutor> SideEffectExecutor for FilteringExecutor<E> {
    async fn execute(&mut self, effects: Vec<SideEffect>) -> Result<(), ExecutorError> {
        let filtered: Vec<_> = effects
            .into_iter()
            .filter(|effect| (self.filter)(effect))
            .collect();

        self.inner.execute(filtered).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection::pure::LogLevel;
    use serde_json::json;

    #[tokio::test]
    async fn test_logging_executor() {
        let mut executor = LoggingExecutor::new();

        let effects = vec![
            SideEffect::Log {
                level: LogLevel::Info,
                message: "Test message".to_string(),
            },
            SideEffect::DatabaseWrite {
                collection: "test".to_string(),
                data: json!({"key": "value"}),
            },
        ];

        executor.execute(effects).await.unwrap();

        assert_eq!(executor.effects().len(), 2);
        assert!(matches!(executor.effects()[0], SideEffect::Log { .. }));
        assert!(matches!(
            executor.effects()[1],
            SideEffect::DatabaseWrite { .. }
        ));
    }

    #[tokio::test]
    async fn test_null_executor() {
        let mut executor = NullExecutor::new();

        let effects = vec![SideEffect::Log {
            level: LogLevel::Info,
            message: "Discarded".to_string(),
        }];

        // Should succeed but do nothing
        executor.execute(effects).await.unwrap();
    }

    #[tokio::test]
    async fn test_collecting_executor() {
        let mut executor = CollectingExecutor::new();

        let effects1 = vec![SideEffect::Log {
            level: LogLevel::Info,
            message: "First".to_string(),
        }];

        let effects2 = vec![SideEffect::Log {
            level: LogLevel::Info,
            message: "Second".to_string(),
        }];

        executor.execute(effects1).await.unwrap();
        executor.execute(effects2).await.unwrap();

        assert_eq!(executor.effects().len(), 2);

        // Take effects
        let taken = executor.take_effects();
        assert_eq!(taken.len(), 2);
        assert_eq!(executor.effects().len(), 0);
    }

    #[tokio::test]
    async fn test_filtering_executor() {
        let mut executor = FilteringExecutor::new(LoggingExecutor::new(), |effect| {
            // Only allow Log effects
            matches!(effect, SideEffect::Log { .. })
        });

        let effects = vec![
            SideEffect::Log {
                level: LogLevel::Info,
                message: "Allowed".to_string(),
            },
            SideEffect::DatabaseWrite {
                collection: "test".to_string(),
                data: json!({}),
            },
        ];

        executor.execute(effects).await.unwrap();

        // Only the Log effect should have been executed
        assert_eq!(executor.inner.effects().len(), 1);
        assert!(matches!(
            executor.inner.effects()[0],
            SideEffect::Log { .. }
        ));
    }
}
