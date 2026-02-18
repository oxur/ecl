//! Handler functions for config CLI commands.
//!
//! Implements `fabryk config {path,get,set,init,export}` subcommands
//! and TOML dotted-key helper functions.

use crate::cli::ConfigAction;
use crate::config::FabrykConfig;
use fabryk_core::{Error, Result};
use std::path::PathBuf;

// ============================================================================
// Command dispatch
// ============================================================================

/// Handle a config subcommand.
///
/// Receives the raw `--config` path (not a loaded config) because some
/// commands (path, init) work before a config file exists.
pub fn handle_config_command(config_path: Option<&str>, action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Path => cmd_config_path(config_path),
        ConfigAction::Get { key } => cmd_config_get(config_path, &key),
        ConfigAction::Set { key, value } => cmd_config_set(config_path, &key, &value),
        ConfigAction::Init { file, force } => cmd_config_init(file.as_deref(), force),
        ConfigAction::Export { docker_env } => {
            let config = FabrykConfig::load(config_path)?;
            cmd_config_export(&config, docker_env)
        }
    }
}

// ============================================================================
// Command handlers
// ============================================================================

/// Show the resolved config file path.
fn cmd_config_path(config_path: Option<&str>) -> Result<()> {
    match FabrykConfig::resolve_config_path(config_path) {
        Some(path) => {
            let exists = path.exists();
            println!("{}", path.display());
            if !exists {
                eprintln!("(file does not exist — run `fabryk config init` to create it)");
            }
            Ok(())
        }
        None => Err(Error::config(
            "Could not determine config directory for this platform",
        )),
    }
}

/// Get a configuration value by dotted key.
fn cmd_config_get(config_path: Option<&str>, key: &str) -> Result<()> {
    let config = FabrykConfig::load(config_path)?;
    let value = toml::Value::try_from(&config).map_err(|e| Error::config(e.to_string()))?;
    match get_nested_value(&value, key) {
        Some(val) => {
            println!("{}", format_toml_value(val));
            Ok(())
        }
        None => Err(Error::config(format!(
            "Key '{key}' not found in configuration"
        ))),
    }
}

/// Set a configuration value by dotted key in the config file.
fn cmd_config_set(config_path: Option<&str>, key: &str, value: &str) -> Result<()> {
    let path = FabrykConfig::resolve_config_path(config_path)
        .ok_or_else(|| Error::config("Could not determine config directory"))?;

    let mut doc: toml::Value = if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| Error::io_with_path(e, &path))?;
        toml::from_str(&content)
            .map_err(|e| Error::config(format!("Failed to parse {}: {e}", path.display())))?
    } else {
        return Err(Error::config(format!(
            "Config file does not exist at {}. Run `fabryk config init` first.",
            path.display()
        )));
    };

    set_nested_value(&mut doc, key, parse_value(value))?;

    let toml_str = toml::to_string_pretty(&doc).map_err(|e| Error::config(e.to_string()))?;
    std::fs::write(&path, toml_str).map_err(|e| Error::io_with_path(e, &path))?;

    println!("Set {key} = {value} in {}", path.display());
    Ok(())
}

/// Create a default configuration file.
fn cmd_config_init(file: Option<&str>, force: bool) -> Result<()> {
    let path = match file {
        Some(p) => PathBuf::from(p),
        None => FabrykConfig::default_config_path()
            .ok_or_else(|| Error::config("Could not determine config directory"))?,
    };

    if path.exists() && !force {
        return Err(Error::config(format!(
            "Config file already exists at {}. Use --force to overwrite.",
            path.display()
        )));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io_with_path(e, parent))?;
    }

    let config = FabrykConfig::default();
    let toml_str = config.to_toml_string()?;
    std::fs::write(&path, &toml_str).map_err(|e| Error::io_with_path(e, &path))?;

    println!("Config file created at {}", path.display());
    Ok(())
}

/// Export configuration as environment variables.
fn cmd_config_export(config: &FabrykConfig, docker_env: bool) -> Result<()> {
    let vars = config.to_env_vars()?;
    for (key, value) in &vars {
        if docker_env {
            println!("--env {key}={value}");
        } else {
            println!("{key}={value}");
        }
    }
    Ok(())
}

// ============================================================================
// TOML dotted-key helpers
// ============================================================================

/// Navigate a dotted key path in a TOML value tree.
fn get_nested_value<'a>(value: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = value;
    for part in &parts {
        current = current.as_table()?.get(*part)?;
    }
    Some(current)
}

/// Set a value at a dotted key path, creating intermediate tables as needed.
fn set_nested_value(root: &mut toml::Value, key: &str, value: toml::Value) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            let table = current
                .as_table_mut()
                .ok_or_else(|| Error::config("Cannot set key on a non-table value"))?;
            table.insert(part.to_string(), value);
            return Ok(());
        }

        let table = current
            .as_table_mut()
            .ok_or_else(|| Error::config("Cannot navigate into a non-table value"))?;
        if !table.contains_key(*part) {
            table.insert(part.to_string(), toml::Value::Table(toml::map::Map::new()));
        }
        current = table.get_mut(*part).unwrap();
    }

    Err(Error::config("Empty key path"))
}

/// Parse a string value into a TOML value, auto-detecting the type.
///
/// Priority: bool → integer → float → string.
fn parse_value(s: &str) -> toml::Value {
    if s == "true" {
        return toml::Value::Boolean(true);
    }
    if s == "false" {
        return toml::Value::Boolean(false);
    }
    if let Ok(i) = s.parse::<i64>() {
        return toml::Value::Integer(i);
    }
    if let Ok(f) = s.parse::<f64>() {
        return toml::Value::Float(f);
    }
    toml::Value::String(s.to_string())
}

/// Format a TOML value for display on stdout.
fn format_toml_value(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Datetime(dt) => dt.to_string(),
        toml::Value::Array(_) | toml::Value::Table(_) => {
            toml::to_string_pretty(value).unwrap_or_else(|_| format!("{value:?}"))
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // cmd_config_path tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_cmd_config_path_default() {
        let result = cmd_config_path(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_config_path_explicit() {
        let result = cmd_config_path(Some("/explicit/config.toml"));
        assert!(result.is_ok());
    }

    // ------------------------------------------------------------------------
    // cmd_config_get tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_cmd_config_get_simple_key() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let config = FabrykConfig::default();
        std::fs::write(&path, config.to_toml_string().unwrap()).unwrap();

        let result = cmd_config_get(Some(path.to_str().unwrap()), "project_name");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_config_get_nested_key() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let config = FabrykConfig::default();
        std::fs::write(&path, config.to_toml_string().unwrap()).unwrap();

        let result = cmd_config_get(Some(path.to_str().unwrap()), "server.port");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_config_get_missing_key() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let config = FabrykConfig::default();
        std::fs::write(&path, config.to_toml_string().unwrap()).unwrap();

        let result = cmd_config_get(Some(path.to_str().unwrap()), "nonexistent.key");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // ------------------------------------------------------------------------
    // cmd_config_set tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_cmd_config_set_simple_key() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let config = FabrykConfig::default();
        std::fs::write(&path, config.to_toml_string().unwrap()).unwrap();

        let result = cmd_config_set(Some(path.to_str().unwrap()), "project_name", "new-name");
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("new-name"));
    }

    #[test]
    fn test_cmd_config_set_nested_key() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let config = FabrykConfig::default();
        std::fs::write(&path, config.to_toml_string().unwrap()).unwrap();

        let result = cmd_config_set(Some(path.to_str().unwrap()), "server.port", "8080");
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("8080"));
    }

    #[test]
    fn test_cmd_config_set_missing_file() {
        let result = cmd_config_set(Some("/nonexistent/config.toml"), "key", "value");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    // ------------------------------------------------------------------------
    // cmd_config_init tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_cmd_config_init_creates_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("fabryk").join("config.toml");

        let result = cmd_config_init(Some(path.to_str().unwrap()), false);
        assert!(result.is_ok());
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("project_name"));
        assert!(content.contains("[server]"));
    }

    #[test]
    fn test_cmd_config_init_no_overwrite() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "existing").unwrap();

        let result = cmd_config_init(Some(path.to_str().unwrap()), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_cmd_config_init_force_overwrites() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "old content").unwrap();

        let result = cmd_config_init(Some(path.to_str().unwrap()), true);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("project_name"));
    }

    // ------------------------------------------------------------------------
    // cmd_config_export tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_cmd_config_export_env_vars() {
        let config = FabrykConfig::default();
        let result = cmd_config_export(&config, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_config_export_docker_env() {
        let config = FabrykConfig::default();
        let result = cmd_config_export(&config, true);
        assert!(result.is_ok());
    }

    // ------------------------------------------------------------------------
    // get_nested_value tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_get_nested_value_top_level() {
        let val: toml::Value = toml::from_str("port = 8080").unwrap();
        let result = get_nested_value(&val, "port");
        assert_eq!(result, Some(&toml::Value::Integer(8080)));
    }

    #[test]
    fn test_get_nested_value_nested() {
        let val: toml::Value = toml::from_str("[server]\nport = 3000").unwrap();
        let result = get_nested_value(&val, "server.port");
        assert_eq!(result, Some(&toml::Value::Integer(3000)));
    }

    #[test]
    fn test_get_nested_value_missing() {
        let val: toml::Value = toml::from_str("port = 8080").unwrap();
        assert!(get_nested_value(&val, "nonexistent").is_none());
    }

    #[test]
    fn test_get_nested_value_deep_missing() {
        let val: toml::Value = toml::from_str("[server]\nport = 3000").unwrap();
        assert!(get_nested_value(&val, "server.nonexistent").is_none());
    }

    // ------------------------------------------------------------------------
    // set_nested_value tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_set_nested_value_top_level() {
        let mut val: toml::Value = toml::from_str("port = 8080").unwrap();
        set_nested_value(&mut val, "port", toml::Value::Integer(9090)).unwrap();
        assert_eq!(
            get_nested_value(&val, "port"),
            Some(&toml::Value::Integer(9090))
        );
    }

    #[test]
    fn test_set_nested_value_creates_section() {
        let mut val = toml::Value::Table(toml::map::Map::new());
        set_nested_value(&mut val, "server.port", toml::Value::Integer(3000)).unwrap();
        assert_eq!(
            get_nested_value(&val, "server.port"),
            Some(&toml::Value::Integer(3000))
        );
    }

    #[test]
    fn test_set_nested_value_overwrites() {
        let mut val: toml::Value = toml::from_str("[server]\nport = 3000").unwrap();
        set_nested_value(&mut val, "server.port", toml::Value::Integer(8080)).unwrap();
        assert_eq!(
            get_nested_value(&val, "server.port"),
            Some(&toml::Value::Integer(8080))
        );
    }

    // ------------------------------------------------------------------------
    // parse_value tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_parse_value_types() {
        assert_eq!(parse_value("true"), toml::Value::Boolean(true));
        assert_eq!(parse_value("false"), toml::Value::Boolean(false));
        assert_eq!(parse_value("42"), toml::Value::Integer(42));
        assert_eq!(parse_value("-7"), toml::Value::Integer(-7));
        assert_eq!(parse_value("3.14"), toml::Value::Float(3.14));
        assert_eq!(
            parse_value("hello world"),
            toml::Value::String("hello world".to_string())
        );
    }

    // ------------------------------------------------------------------------
    // format_toml_value tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_format_toml_value() {
        assert_eq!(
            format_toml_value(&toml::Value::String("hello".into())),
            "hello"
        );
        assert_eq!(format_toml_value(&toml::Value::Integer(42)), "42");
        assert_eq!(format_toml_value(&toml::Value::Float(3.14)), "3.14");
        assert_eq!(format_toml_value(&toml::Value::Boolean(true)), "true");
    }
}
