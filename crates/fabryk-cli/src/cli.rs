//! CLI argument parsing and command definitions.
//!
//! Provides the common CLI structure that all Fabryk-based applications share:
//! configuration, verbosity, and base commands (serve, index, version, health, graph).
//!
//! Domain applications extend this via the [`CliExtension`] trait.

use clap::{Parser, Subcommand};

// ============================================================================
// CLI argument types
// ============================================================================

/// Top-level CLI arguments for Fabryk applications.
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
pub struct CliArgs {
    /// Path to configuration file.
    #[arg(short, long, env = "FABRYK_CONFIG")]
    pub config: Option<String>,

    /// Enable verbose output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress non-essential output.
    #[arg(short, long)]
    pub quiet: bool,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Option<BaseCommand>,
}

/// Built-in commands shared by all Fabryk applications.
#[derive(Subcommand, Debug)]
pub enum BaseCommand {
    /// Start the MCP server.
    Serve {
        /// Port to listen on.
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// Build or refresh the content index.
    Index {
        /// Force full re-index.
        #[arg(short, long)]
        force: bool,

        /// Check index freshness without rebuilding.
        #[arg(long)]
        check: bool,
    },

    /// Print version information.
    Version,

    /// Check system health.
    Health,

    /// Graph operations.
    Graph(GraphCommand),
}

/// Graph-specific subcommands.
#[derive(Parser, Debug)]
pub struct GraphCommand {
    /// Graph subcommand to execute.
    #[command(subcommand)]
    pub command: GraphSubcommand,
}

/// Available graph subcommands.
#[derive(Subcommand, Debug)]
pub enum GraphSubcommand {
    /// Build the knowledge graph from content.
    Build {
        /// Output file path for the graph.
        #[arg(short, long)]
        output: Option<String>,

        /// Show what would be built without writing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate graph integrity.
    Validate,

    /// Show graph statistics.
    Stats,

    /// Query the graph.
    Query {
        /// Node ID to query.
        #[arg(short, long)]
        id: String,

        /// Type of query: related, prerequisites, path.
        #[arg(short = 't', long, default_value = "related")]
        query_type: String,

        /// Target node ID (for path queries).
        #[arg(long)]
        to: Option<String>,
    },
}

// ============================================================================
// CliExtension trait
// ============================================================================

/// Extension point for domain-specific CLI commands.
///
/// Domain applications implement this trait to add custom subcommands
/// beyond the built-in base commands.
pub trait CliExtension: Send + Sync {
    /// The domain-specific command type.
    type Command: Send + Sync;

    /// Handle a domain-specific command.
    fn handle_command(
        &self,
        command: Self::Command,
    ) -> impl std::future::Future<Output = fabryk_core::Result<()>> + Send;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_args_default() {
        let args = CliArgs::parse_from(["test"]);
        assert!(args.config.is_none());
        assert!(!args.verbose);
        assert!(!args.quiet);
        assert!(args.command.is_none());
    }

    #[test]
    fn test_cli_args_verbose() {
        let args = CliArgs::parse_from(["test", "--verbose"]);
        assert!(args.verbose);
        assert!(!args.quiet);
    }

    #[test]
    fn test_cli_args_quiet() {
        let args = CliArgs::parse_from(["test", "--quiet"]);
        assert!(!args.verbose);
        assert!(args.quiet);
    }

    #[test]
    fn test_cli_args_config() {
        let args = CliArgs::parse_from(["test", "--config", "/path/to/config.toml"]);
        assert_eq!(args.config, Some("/path/to/config.toml".to_string()));
    }

    #[test]
    fn test_serve_command() {
        let args = CliArgs::parse_from(["test", "serve"]);
        match args.command {
            Some(BaseCommand::Serve { port }) => assert_eq!(port, 3000),
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_serve_command_custom_port() {
        let args = CliArgs::parse_from(["test", "serve", "--port", "8080"]);
        match args.command {
            Some(BaseCommand::Serve { port }) => assert_eq!(port, 8080),
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_index_command() {
        let args = CliArgs::parse_from(["test", "index"]);
        match args.command {
            Some(BaseCommand::Index { force, check }) => {
                assert!(!force);
                assert!(!check);
            }
            _ => panic!("Expected Index command"),
        }
    }

    #[test]
    fn test_index_command_force() {
        let args = CliArgs::parse_from(["test", "index", "--force"]);
        match args.command {
            Some(BaseCommand::Index { force, check }) => {
                assert!(force);
                assert!(!check);
            }
            _ => panic!("Expected Index command with force"),
        }
    }

    #[test]
    fn test_version_command() {
        let args = CliArgs::parse_from(["test", "version"]);
        assert!(matches!(args.command, Some(BaseCommand::Version)));
    }

    #[test]
    fn test_health_command() {
        let args = CliArgs::parse_from(["test", "health"]);
        assert!(matches!(args.command, Some(BaseCommand::Health)));
    }

    #[test]
    fn test_graph_build_command() {
        let args = CliArgs::parse_from(["test", "graph", "build"]);
        match args.command {
            Some(BaseCommand::Graph(GraphCommand {
                command: GraphSubcommand::Build { output, dry_run },
            })) => {
                assert!(output.is_none());
                assert!(!dry_run);
            }
            _ => panic!("Expected Graph Build command"),
        }
    }

    #[test]
    fn test_graph_build_dry_run() {
        let args = CliArgs::parse_from(["test", "graph", "build", "--dry-run"]);
        match args.command {
            Some(BaseCommand::Graph(GraphCommand {
                command: GraphSubcommand::Build { dry_run, .. },
            })) => {
                assert!(dry_run);
            }
            _ => panic!("Expected Graph Build command with dry_run"),
        }
    }

    #[test]
    fn test_graph_validate_command() {
        let args = CliArgs::parse_from(["test", "graph", "validate"]);
        match args.command {
            Some(BaseCommand::Graph(GraphCommand {
                command: GraphSubcommand::Validate,
            })) => {}
            _ => panic!("Expected Graph Validate command"),
        }
    }

    #[test]
    fn test_graph_stats_command() {
        let args = CliArgs::parse_from(["test", "graph", "stats"]);
        match args.command {
            Some(BaseCommand::Graph(GraphCommand {
                command: GraphSubcommand::Stats,
            })) => {}
            _ => panic!("Expected Graph Stats command"),
        }
    }

    #[test]
    fn test_graph_query_command() {
        let args = CliArgs::parse_from(["test", "graph", "query", "--id", "node-1"]);
        match args.command {
            Some(BaseCommand::Graph(GraphCommand {
                command: GraphSubcommand::Query { id, query_type, to },
            })) => {
                assert_eq!(id, "node-1");
                assert_eq!(query_type, "related");
                assert!(to.is_none());
            }
            _ => panic!("Expected Graph Query command"),
        }
    }

    #[test]
    fn test_graph_query_path() {
        let args = CliArgs::parse_from([
            "test",
            "graph",
            "query",
            "--id",
            "a",
            "--query-type",
            "path",
            "--to",
            "b",
        ]);
        match args.command {
            Some(BaseCommand::Graph(GraphCommand {
                command: GraphSubcommand::Query { id, query_type, to },
            })) => {
                assert_eq!(id, "a");
                assert_eq!(query_type, "path");
                assert_eq!(to, Some("b".to_string()));
            }
            _ => panic!("Expected Graph Query path command"),
        }
    }
}
