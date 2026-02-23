//! Core traits for Fabryk domain abstraction.
//!
//! These traits define the extension points that domain applications implement
//! to customise Fabryk's behaviour. The primary trait is [`ConfigProvider`],
//! which abstracts domain-specific configuration.

use std::path::PathBuf;

use crate::Result;

/// Trait for domain-specific configuration.
///
/// Every Fabryk-based application implements this trait to provide
/// the configuration that Fabryk crates need: paths to content,
/// project identity, and domain-specific settings.
///
/// # Bounds
///
/// - `Send + Sync`: Configuration must be shareable across threads
/// - `Clone`: Configuration can be duplicated for passing to subsystems
/// - `'static`: Configuration lifetime is not borrowed
///
/// # Example
///
/// ```
/// use std::path::PathBuf;
/// use fabryk_core::traits::ConfigProvider;
/// use fabryk_core::Result;
///
/// #[derive(Clone)]
/// struct MusicTheoryConfig {
///     data_dir: PathBuf,
/// }
///
/// impl ConfigProvider for MusicTheoryConfig {
///     fn project_name(&self) -> &str {
///         "music-theory"
///     }
///
///     fn base_path(&self) -> Result<PathBuf> {
///         Ok(self.data_dir.clone())
///     }
///
///     fn content_path(&self, content_type: &str) -> Result<PathBuf> {
///         Ok(self.data_dir.join(content_type))
///     }
/// }
/// ```
pub trait ConfigProvider: Send + Sync + Clone + 'static {
    /// The project name, used for env var prefixes and default paths.
    ///
    /// This name is used by [`PathResolver`](crate::PathResolver) to generate
    /// environment variable names. For example, `"music-theory"` produces
    /// env vars like `MUSIC_THEORY_CONFIG_DIR`.
    ///
    /// # Example
    ///
    /// ```
    /// # use fabryk_core::traits::ConfigProvider;
    /// # #[derive(Clone)]
    /// # struct Config;
    /// # impl ConfigProvider for Config {
    /// fn project_name(&self) -> &str {
    ///     "music-theory"
    /// }
    /// #     fn base_path(&self) -> fabryk_core::Result<std::path::PathBuf> { todo!() }
    /// #     fn content_path(&self, _: &str) -> fabryk_core::Result<std::path::PathBuf> { todo!() }
    /// # }
    /// ```
    fn project_name(&self) -> &str;

    /// Base path for all project data.
    ///
    /// This is the root directory under which all content, caches,
    /// and generated files are stored.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be determined (e.g., missing
    /// environment variable or invalid configuration).
    fn base_path(&self) -> Result<PathBuf>;

    /// Path for a specific content type.
    ///
    /// `content_type` is a domain-defined key like `"concepts"`,
    /// `"sources"`, `"guides"`. The implementation decides how to
    /// map these to actual filesystem paths.
    ///
    /// # Arguments
    ///
    /// * `content_type` — A domain-specific content category identifier
    ///
    /// # Errors
    ///
    /// Returns an error if the content type is unknown or the path
    /// cannot be resolved.
    ///
    /// # Example
    ///
    /// ```
    /// # use fabryk_core::traits::ConfigProvider;
    /// # use std::path::PathBuf;
    /// # #[derive(Clone)]
    /// # struct Config { base: PathBuf }
    /// # impl ConfigProvider for Config {
    /// #     fn project_name(&self) -> &str { "test" }
    /// #     fn base_path(&self) -> fabryk_core::Result<PathBuf> { Ok(self.base.clone()) }
    /// fn content_path(&self, content_type: &str) -> fabryk_core::Result<PathBuf> {
    ///     match content_type {
    ///         "concepts" => Ok(self.base.join("data/concepts")),
    ///         "sources" => Ok(self.base.join("data/sources")),
    ///         "guides" => Ok(self.base.join("guides")),
    ///         _ => Err(fabryk_core::Error::config(
    ///             format!("Unknown content type: {}", content_type)
    ///         )),
    ///     }
    /// }
    /// # }
    /// ```
    fn content_path(&self, content_type: &str) -> Result<PathBuf>;

    /// Path for a specific cache type.
    ///
    /// `cache_type` is a framework-defined key like `"graph"`, `"fts"`,
    /// `"vector"`. The default implementation derives cache paths from
    /// [`base_path()`](Self::base_path) as `{base}/.cache/{cache_type}`.
    ///
    /// Products can override this to customise cache locations.
    ///
    /// # Standard cache types
    ///
    /// - `"graph"` — Knowledge graph cache (rkyv/JSON serialized)
    /// - `"fts"` — Full-text search index (Tantivy)
    /// - `"vector"` — Vector embedding index
    ///
    /// # Example
    ///
    /// ```
    /// # use fabryk_core::traits::ConfigProvider;
    /// # use std::path::PathBuf;
    /// # #[derive(Clone)]
    /// # struct Config { base: PathBuf }
    /// # impl ConfigProvider for Config {
    /// #     fn project_name(&self) -> &str { "test" }
    /// #     fn base_path(&self) -> fabryk_core::Result<PathBuf> { Ok(self.base.clone()) }
    /// #     fn content_path(&self, _: &str) -> fabryk_core::Result<PathBuf> { todo!() }
    /// # }
    /// let config = Config { base: PathBuf::from("/project") };
    /// // Default: /project/.cache/graph
    /// assert_eq!(
    ///     config.cache_path("graph").unwrap(),
    ///     PathBuf::from("/project/.cache/graph")
    /// );
    /// ```
    fn cache_path(&self, cache_type: &str) -> Result<PathBuf> {
        Ok(self.base_path()?.join(".cache").join(cache_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
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

    #[test]
    fn test_config_provider_project_name() {
        let config = TestConfig {
            name: "test-project".into(),
            base: PathBuf::from("/tmp/test"),
        };
        assert_eq!(config.project_name(), "test-project");
    }

    #[test]
    fn test_config_provider_base_path() {
        let config = TestConfig {
            name: "test".into(),
            base: PathBuf::from("/data"),
        };
        let path = config.base_path().unwrap();
        assert_eq!(path, PathBuf::from("/data"));
    }

    #[test]
    fn test_config_provider_content_path() {
        let config = TestConfig {
            name: "test".into(),
            base: PathBuf::from("/data"),
        };
        let path = config.content_path("concepts").unwrap();
        assert_eq!(path, PathBuf::from("/data/concepts"));
    }

    #[test]
    fn test_config_provider_content_path_multiple() {
        let config = TestConfig {
            name: "test".into(),
            base: PathBuf::from("/project"),
        };

        assert_eq!(
            config.content_path("sources").unwrap(),
            PathBuf::from("/project/sources")
        );
        assert_eq!(
            config.content_path("guides").unwrap(),
            PathBuf::from("/project/guides")
        );
        assert_eq!(
            config.content_path("graphs").unwrap(),
            PathBuf::from("/project/graphs")
        );
    }

    #[test]
    fn test_config_provider_cache_path_default() {
        let config = TestConfig {
            name: "test".into(),
            base: PathBuf::from("/project"),
        };
        assert_eq!(
            config.cache_path("graph").unwrap(),
            PathBuf::from("/project/.cache/graph")
        );
        assert_eq!(
            config.cache_path("fts").unwrap(),
            PathBuf::from("/project/.cache/fts")
        );
        assert_eq!(
            config.cache_path("vector").unwrap(),
            PathBuf::from("/project/.cache/vector")
        );
    }

    #[test]
    fn test_config_provider_cache_path_override() {
        #[derive(Clone)]
        struct CustomCacheConfig;

        impl ConfigProvider for CustomCacheConfig {
            fn project_name(&self) -> &str {
                "custom"
            }
            fn base_path(&self) -> Result<PathBuf> {
                Ok(PathBuf::from("/data"))
            }
            fn content_path(&self, _content_type: &str) -> Result<PathBuf> {
                Ok(PathBuf::from("/data/content"))
            }
            fn cache_path(&self, cache_type: &str) -> Result<PathBuf> {
                Ok(PathBuf::from("/var/cache/custom").join(cache_type))
            }
        }

        let config = CustomCacheConfig;
        assert_eq!(
            config.cache_path("graph").unwrap(),
            PathBuf::from("/var/cache/custom/graph")
        );
    }

    #[test]
    fn test_config_provider_is_clone() {
        let config = TestConfig {
            name: "test".into(),
            base: PathBuf::from("/data"),
        };
        let cloned = config.clone();
        assert_eq!(config.project_name(), cloned.project_name());
    }

    #[test]
    fn test_config_provider_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TestConfig>();
    }
}
