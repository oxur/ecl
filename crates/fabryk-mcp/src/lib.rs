//! Core MCP server infrastructure for Fabryk.
//!
//! This crate provides the foundational MCP server setup, tool
//! registration, and the health check tool.
//!
//! # Key Abstractions
//!
//! - `FabrykMcpServer<C>`: Generic MCP server parameterized over config
//! - `ToolRegistry` trait: Domain implementations register their tools

#![doc = include_str!("../README.md")]

// Modules to be added during extraction:
// pub mod server;
// pub mod registry;
// pub mod tools;
