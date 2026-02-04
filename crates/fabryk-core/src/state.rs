//! Application state management.
//!
//! Provides [`AppState<C>`], a thread-safe container for shared application
//! state that is generic over the configuration provider.
//!
//! # Design
//!
//! `AppState` in fabryk-core is intentionally minimal — it holds only the
//! configuration. Domain applications and higher-level Fabryk crates
//! (fabryk-fts, fabryk-graph) may wrap or extend this with their own
//! state (search backends, graph caches, etc.).
//!
//! # Example
//!
//! ```
//! use std::path::PathBuf;
//! use fabryk_core::{AppState, ConfigProvider, Result};
//!
//! #[derive(Clone)]
//! struct MyConfig {
//!     name: String,
//!     base: PathBuf,
//! }
//!
//! impl ConfigProvider for MyConfig {
//!     fn project_name(&self) -> &str { &self.name }
//!     fn base_path(&self) -> Result<PathBuf> { Ok(self.base.clone()) }
//!     fn content_path(&self, t: &str) -> Result<PathBuf> { Ok(self.base.join(t)) }
//! }
//!
//! let config = MyConfig {
//!     name: "my-project".into(),
//!     base: PathBuf::from("/data"),
//! };
//! let state = AppState::new(config);
//!
//! assert_eq!(state.config().project_name(), "my-project");
//! ```

use std::sync::Arc;

use crate::traits::ConfigProvider;

/// Thread-safe shared application state.
///
/// Generic over `C: ConfigProvider` so that any domain can use it
/// with their own configuration type. The configuration is wrapped
/// in an `Arc` for cheap cloning and thread-safe sharing.
///
/// # Type Parameters
///
/// - `C` — The domain-specific configuration provider
///
/// # Thread Safety
///
/// `AppState` is `Clone`, `Send`, and `Sync`. Cloning is cheap (Arc clone).
/// Multiple request handlers can share the same state concurrently.
#[derive(Debug)]
pub struct AppState<C: ConfigProvider> {
    config: Arc<C>,
}

impl<C: ConfigProvider> AppState<C> {
    /// Create a new AppState wrapping the given configuration.
    ///
    /// The configuration is moved into an `Arc` for shared ownership.
    ///
    /// # Arguments
    ///
    /// * `config` — The domain-specific configuration
    ///
    /// # Example
    ///
    /// ```
    /// # use fabryk_core::{AppState, ConfigProvider, Result};
    /// # use std::path::PathBuf;
    /// # #[derive(Clone)]
    /// # struct Config { base: PathBuf }
    /// # impl ConfigProvider for Config {
    /// #     fn project_name(&self) -> &str { "test" }
    /// #     fn base_path(&self) -> Result<PathBuf> { Ok(self.base.clone()) }
    /// #     fn content_path(&self, t: &str) -> Result<PathBuf> { Ok(self.base.join(t)) }
    /// # }
    /// let config = Config { base: PathBuf::from("/data") };
    /// let state = AppState::new(config);
    /// ```
    pub fn new(config: C) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Create AppState from an existing Arc-wrapped configuration.
    ///
    /// Useful when the configuration is already shared elsewhere.
    ///
    /// # Arguments
    ///
    /// * `config` — Arc-wrapped configuration
    pub fn from_arc(config: Arc<C>) -> Self {
        Self { config }
    }

    /// Get a reference to the configuration.
    ///
    /// # Example
    ///
    /// ```
    /// # use fabryk_core::{AppState, ConfigProvider, Result};
    /// # use std::path::PathBuf;
    /// # #[derive(Clone)]
    /// # struct Config { name: String, base: PathBuf }
    /// # impl ConfigProvider for Config {
    /// #     fn project_name(&self) -> &str { &self.name }
    /// #     fn base_path(&self) -> Result<PathBuf> { Ok(self.base.clone()) }
    /// #     fn content_path(&self, t: &str) -> Result<PathBuf> { Ok(self.base.join(t)) }
    /// # }
    /// # let config = Config { name: "test".into(), base: PathBuf::from("/data") };
    /// # let state = AppState::new(config);
    /// let project = state.config().project_name();
    /// ```
    pub fn config(&self) -> &C {
        &self.config
    }

    /// Get a cloneable handle to the configuration.
    ///
    /// Returns an `Arc<C>` that can be passed to subsystems that need
    /// their own owned reference to the configuration.
    ///
    /// # Example
    ///
    /// ```
    /// # use fabryk_core::{AppState, ConfigProvider, Result};
    /// # use std::path::PathBuf;
    /// # #[derive(Clone)]
    /// # struct Config { base: PathBuf }
    /// # impl ConfigProvider for Config {
    /// #     fn project_name(&self) -> &str { "test" }
    /// #     fn base_path(&self) -> Result<PathBuf> { Ok(self.base.clone()) }
    /// #     fn content_path(&self, t: &str) -> Result<PathBuf> { Ok(self.base.join(t)) }
    /// # }
    /// # let config = Config { base: PathBuf::from("/data") };
    /// # let state = AppState::new(config);
    /// let config_arc = state.config_arc();
    /// // Pass config_arc to another subsystem
    /// ```
    pub fn config_arc(&self) -> Arc<C> {
        Arc::clone(&self.config)
    }

    /// Get the project name from the configuration.
    ///
    /// Convenience method equivalent to `state.config().project_name()`.
    pub fn project_name(&self) -> &str {
        self.config.project_name()
    }
}

impl<C: ConfigProvider> Clone for AppState<C> {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
        }
    }
}

// Safety: AppState is Send + Sync if C is Send + Sync (which ConfigProvider requires)
unsafe impl<C: ConfigProvider> Send for AppState<C> {}
unsafe impl<C: ConfigProvider> Sync for AppState<C> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;
    use std::path::PathBuf;

    #[derive(Clone, Debug)]
    struct TestConfig {
        name: String,
        base: PathBuf,
    }

    impl ConfigProvider for TestConfig {
        fn project_name(&self) -> &str {
            &self.name
        }

        fn base_path(&self) -> Result<PathBuf> {
            Ok(self.base.clone())
        }

        fn content_path(&self, content_type: &str) -> Result<PathBuf> {
            Ok(self.base.join(content_type))
        }
    }

    fn test_config() -> TestConfig {
        TestConfig {
            name: "test-project".into(),
            base: PathBuf::from("/tmp/test"),
        }
    }

    #[test]
    fn test_app_state_new() {
        let config = test_config();
        let state = AppState::new(config);
        assert_eq!(state.config().project_name(), "test-project");
    }

    #[test]
    fn test_app_state_from_arc() {
        let config = Arc::new(test_config());
        let state = AppState::from_arc(config);
        assert_eq!(state.config().project_name(), "test-project");
    }

    #[test]
    fn test_app_state_config_ref() {
        let config = test_config();
        let state = AppState::new(config);

        let config_ref = state.config();
        assert_eq!(config_ref.project_name(), "test-project");
        assert_eq!(config_ref.base_path().unwrap(), PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_app_state_config_arc() {
        let config = test_config();
        let state = AppState::new(config);

        let arc1 = state.config_arc();
        let arc2 = state.config_arc();

        // Both should point to same allocation
        assert!(Arc::ptr_eq(&arc1, &arc2));
    }

    #[test]
    fn test_app_state_project_name() {
        let config = test_config();
        let state = AppState::new(config);
        assert_eq!(state.project_name(), "test-project");
    }

    #[test]
    fn test_app_state_clone() {
        let config = test_config();
        let state1 = AppState::new(config);
        let state2 = state1.clone();

        // Both should share the same config
        assert_eq!(state1.project_name(), state2.project_name());
        assert!(Arc::ptr_eq(&state1.config_arc(), &state2.config_arc()));
    }

    #[test]
    fn test_app_state_clone_independence() {
        let config = test_config();
        let state1 = AppState::new(config);
        let state2 = state1.clone();

        // Dropping one shouldn't affect the other
        drop(state1);
        assert_eq!(state2.project_name(), "test-project");
    }

    #[test]
    fn test_app_state_debug() {
        let config = test_config();
        let state = AppState::new(config);
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("AppState"));
    }

    #[test]
    fn test_app_state_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AppState<TestConfig>>();
    }

    #[test]
    fn test_app_state_content_path() {
        let config = test_config();
        let state = AppState::new(config);

        let concepts_path = state.config().content_path("concepts").unwrap();
        assert_eq!(concepts_path, PathBuf::from("/tmp/test/concepts"));

        let sources_path = state.config().content_path("sources").unwrap();
        assert_eq!(sources_path, PathBuf::from("/tmp/test/sources"));
    }

    #[test]
    fn test_app_state_arc_count() {
        let config = test_config();
        let state = AppState::new(config);

        // Get arcs and check count
        let arc1 = state.config_arc();
        let arc2 = state.config_arc();

        // Count: state.config + arc1 + arc2 = 3
        assert_eq!(Arc::strong_count(&arc1), 3);

        drop(arc2);
        // Count: state.config + arc1 = 2
        assert_eq!(Arc::strong_count(&arc1), 2);
    }

    #[tokio::test]
    async fn test_app_state_across_tasks() {
        let config = test_config();
        let state = AppState::new(config);

        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            state_clone.project_name().to_string()
        });

        let result = handle.await.unwrap();
        assert_eq!(result, "test-project");
        assert_eq!(state.project_name(), "test-project");
    }
}
