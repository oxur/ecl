//! CLI framework for Fabryk-based applications.
//!
//! This crate provides a generic CLI structure that domain applications
//! can extend with their own commands.
//!
//! # Key Abstractions
//!
//! - [`FabrykCli<C>`]: Generic CLI parameterized over config provider
//! - [`CliExtension`]: Trait for adding domain-specific subcommands
//! - Built-in graph commands (validate, stats, query)

pub mod app;
pub mod cli;
pub mod graph_handlers;

// Re-exports — CLI types
pub use cli::{BaseCommand, CliArgs, CliExtension, GraphCommand, GraphSubcommand};

// Re-exports — application
pub use app::FabrykCli;

// Re-exports — graph handler types
pub use graph_handlers::{BuildOptions, QueryOptions};
