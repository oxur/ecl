# ECL — Extract, Cogitate, Load

[![][build-badge]][build]
[![][crate-badge]][crate]
[![][tag-badge]][tag]
[![][docs-badge]][docs]

[![][logo]][logo-large]

*Far more than agent parallelism, we've found a deep need for "managed serialism" or agent workflow management.*

## What is ECL?

ECL is a Rust-based framework for building **durable AI agent workflows** with explicit control over sequencing, validation, and feedback loops — and a **persistent knowledge fabric** for storing, searching, and serving the results.

The project has two major components:

1. **ECL Workflows** — Managed serial execution of AI agent steps with feedback loops, journaling, and durable state
2. **Fabryk** — A modular knowledge fabric that ingests workflow outputs into a searchable, interconnected knowledge base spanning full-text search, knowledge graphs, and vector/semantic search

While most AI agent frameworks optimize for parallelism—running multiple tools or LLM calls concurrently—ECL addresses a different problem: **workflows that require deliberate, validated sequencing** where each step must complete successfully before the next begins, and downstream steps can request revisions from upstream. Fabryk then gives those workflow results a permanent, queryable home.

### Core Concepts

**Managed Serialism**: Steps execute in defined order with explicit handoffs. Each step validates its input, performs work (often involving LLM calls), and produces typed output for the next step.

**Feedback Loops**: Downstream steps can request revisions from upstream steps. Iteration is bounded—after N attempts, the workflow fails gracefully with full context.

**Durable Execution**: Every step is journaled. Workflows survive process crashes and resume exactly where they left off without re-executing completed steps.

**Knowledge Fabric**: Workflow outputs are persisted into a multi-modal knowledge store — a graph of relationships, a full-text search index, and a vector space — all exposed via MCP tools and a CLI.

---

## Why ECL?

### The Problem

Consider this workflow:

```
Step 1: Extract information from documents following specific instructions
Step 2: Review extraction, request revisions if criteria not met (max 3 attempts)
Step 3: Use validated extraction to produce final deliverables
Step 4: Store results so they can be searched, traversed, and reused
```

This pattern appears everywhere in AI-assisted decision making, planning, and document creation. But existing tools fall short:

- **Agent frameworks** (LangChain, etc.): Optimized for parallelism, not sequential validation
- **Workflow engines** (Airflow, etc.): Designed for data pipelines, not LLM interactions
- **Custom solutions**: Require extensive infrastructure code for durability and state management
- **Knowledge tools**: Siloed — you get a vector DB *or* a graph *or* full-text search, but not a unified fabric

### The Solution

ECL provides:

1. **Workflow primitives** built on Restate's durable execution engine
2. **Step abstractions** with built-in retry, feedback, and validation patterns
3. **Clean LLM integration** focused on Anthropic's Claude with provider abstraction
4. **Knowledge fabric** (Fabryk) with graph, full-text, and vector search — unified under one API

---

## Architecture Overview

```text
┌──────────────────────────────────────────────────────────────────┐
│                                                                  │
│  ECL                                                             │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                      Workflow Layer                        │  │
│  │               (Restate + Step Abstractions)                │  │
│  └────────────────────────────┬───────────────────────────────┘  │
│                               │                                  │
│  Fabryk                       v                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Knowledge Fabric                        │  │
│  │                                                            │  │
│  │   ┌──────────┐    ┌──────────┐    ┌──────────┐             │  │
│  │   │  Graph   │    │   FTS    │    │  Vector  │             │  │
│  │   │(petgraph)│    │(Tantivy) │    │(LanceDB) │             │  │
│  │   └──────────┘    └──────────┘    └──────────┘             │  │
│  │         │               │               │                  │  │
│  │         └───────────────┼───────────────┘                  │  │
│  │                         v                                  │  │
│  │              ┌────────────────────┐                        │  │
│  │              │    MCP Server      │                        │  │
│  │              │  (Tool Interface)  │                        │  │
│  │              └────────────────────┘                        │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │   LLM    │  │   Auth   │  │Resilience│  │       CLI        │  │
│  │ (Claude) │  │(OAuth2)  │  │ (backon) │  │     (clap)       │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### Fabryk Crate Map

The knowledge fabric is composed of modular, feature-gated crates:

| Tier | Crate | Purpose |
|------|-------|---------|
| Foundation | `fabryk-core` | Shared types, traits, error handling |
| Content | `fabryk-content` | Markdown parsing, frontmatter extraction |
| Search | `fabryk-fts` | Full-text search (Tantivy backend) |
| Search | `fabryk-graph` | Knowledge graph storage & traversal (petgraph) |
| Search | `fabryk-vector` | Vector/semantic search (LanceDB + fastembed) |
| Auth | `fabryk-auth` | Token validation & Tower middleware |
| Auth | `fabryk-auth-google` | Google OAuth2 / JWKS provider |
| Auth | `fabryk-auth-mcp` | RFC 9728 OAuth2 discovery endpoints |
| MCP | `fabryk-mcp` | Core MCP server infrastructure (rmcp) |
| MCP | `fabryk-mcp-content` | Content & source MCP tools |
| MCP | `fabryk-mcp-fts` | Full-text search MCP tools |
| MCP | `fabryk-mcp-graph` | Graph query MCP tools |
| CLI | `fabryk-cli` | CLI framework with graph commands |
| ACL | `fabryk-acl` | Access control (placeholder) |
| Umbrella | `fabryk` | Re-exports everything, feature-gated |

The `fabryk` umbrella crate lets you pull in exactly what you need:

```toml
# Just graph and FTS
fabryk = { version = "0.1", features = ["graph", "fts-tantivy"] }

# Everything including MCP server and CLI
fabryk = { version = "0.1", features = ["full"] }
```

### Key Dependencies

| Component | Library | Purpose |
|-----------|---------|---------|
| Orchestration | [Restate](https://restate.dev) | Durable workflow execution |
| LLM Integration | [llm](https://crates.io/crates/llm) | Claude API abstraction |
| Knowledge Graph | [petgraph](https://crates.io/crates/petgraph) | Graph data structures & algorithms |
| Full-Text Search | [Tantivy](https://crates.io/crates/tantivy) | Rust-native search engine |
| Vector Search | [LanceDB](https://lancedb.com/) | Embedded vector database |
| Embeddings | [fastembed](https://crates.io/crates/fastembed) | Local embedding generation |
| MCP Server | [rmcp](https://crates.io/crates/rmcp) | Model Context Protocol |
| Auth | [jsonwebtoken](https://crates.io/crates/jsonwebtoken) | JWT validation |
| CLI | [clap](https://crates.io/crates/clap) | Command-line parsing |
| Retry Logic | [backon](https://crates.io/crates/backon) | Exponential backoff |
| Configuration | [figment](https://crates.io/crates/figment) | Hierarchical config |
| Observability | [tracing](https://crates.io/crates/tracing) | Structured logging |

---

## Example: Document Review Pipeline

```rust
#[restate_sdk::workflow]
pub trait DocumentReviewWorkflow {
    /// Main workflow execution — runs exactly once per workflow instance
    async fn run(input: WorkflowInput) -> Result<WorkflowOutput, HandlerError>;

    /// Signal handler for feedback — can be called multiple times
    #[shared]
    async fn submit_feedback(feedback: ReviewFeedback) -> Result<(), HandlerError>;

    /// Query handler for status — can be called anytime
    #[shared]
    async fn get_status() -> Result<WorkflowStatus, HandlerError>;
}

impl DocumentReviewWorkflow for DocumentReviewWorkflowImpl {
    async fn run(
        &self,
        ctx: WorkflowContext<'_>,
        input: WorkflowInput,
    ) -> Result<WorkflowOutput, HandlerError> {
        // Step 1: Extract — durable, won't re-execute on recovery
        let extraction = ctx.run(|| {
            self.llm.extract(&input.files, &input.instructions)
        }).await?;

        ctx.set("status", "Extraction complete, awaiting review");

        // Step 2: Review with feedback loop
        let mut attempts = 0;
        let validated = loop {
            // Wait for review feedback (durable promise)
            let feedback = ctx.promise::<ReviewFeedback>("review").await?;

            if feedback.approved {
                break extraction.clone();
            }

            attempts += 1;
            if attempts >= input.max_iterations {
                return Err(HandlerError::terminal("Max iterations exceeded"));
            }

            // Revise based on feedback — also durable
            extraction = ctx.run(|| {
                self.llm.revise(&extraction, &feedback.comments)
            }).await?;

            ctx.set("status", format!("Revision {} complete", attempts));
        };

        // Step 3: Produce final output
        let output = ctx.run(|| {
            self.llm.produce(&validated, &input.output_instructions)
        }).await?;

        ctx.set("status", "Complete");
        Ok(output)
    }
}
```

---

## Project Status

**Active Development** — The knowledge fabric (Fabryk) is functional; workflow engine is in progress.

### Completed

- [x] Architecture design and library research
- [x] Knowledge graph with traversal algorithms (fabryk-graph)
- [x] Full-text search with Tantivy backend (fabryk-fts)
- [x] Vector/semantic search with LanceDB (fabryk-vector)
- [x] Markdown content parsing and frontmatter extraction (fabryk-content)
- [x] MCP server infrastructure and tool suites (fabryk-mcp-*)
- [x] OAuth2 authentication with Google provider (fabryk-auth-*)
- [x] CLI framework with graph commands (fabryk-cli)
- [x] Configuration infrastructure with TOML support

### In Progress

- [ ] ECL workflow primitives (Restate integration)
- [ ] Step abstraction layer with feedback loops
- [ ] LLM integration
- [ ] Connecting ECL workflows to Fabryk persistence

### Planned

- [ ] Access control layer (fabryk-acl)
- [ ] Additional MCP tool suites
- [ ] Example workflows
- [ ] Published documentation

---

## Getting Started

> **Note**: ECL is under active development. The Fabryk crates are functional; the workflow engine is still in progress.

### Prerequisites

- Rust 1.75+
- [Restate Server](https://docs.restate.dev/get_started/) (for workflow execution)
- Anthropic API key (for LLM steps)

### Building

```bash
git clone https://github.com/oxur/ecl
cd ecl
cargo build
```

---

## Documentation

- [Architecture Proposal](docs/01_architecture_proposal.md) — System design and conceptual model
- [Library Research](docs/02_library_research.md) — Detailed analysis of chosen libraries
- [Project Proposal](docs/03_proposal_and_justification.md) — Strategic rationale and implementation plan

---

## Contributing

We're not yet accepting external contributions, but will open the project once the core architecture stabilizes.

---

## License

TBD

---

[//]: ---Named-Links---

[logo]: assets/images/logo/v1-x250.png
[logo-large]: assets/images/logo/v1.png
[build]: https://github.com/oxur/ecl/actions/workflows/ci.yml
[build-badge]: https://github.com/oxur/ecl/actions/workflows/ci.yml/badge.svg
[crate]: https://crates.io/crates/ecl
[crate-badge]: https://img.shields.io/crates/v/ecl.svg
[docs]: https://docs.rs/ecl/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
[tag-badge]: https://img.shields.io/github/tag/oxur/ecl.svg
[tag]: https://github.com/oxur/ecl/tags
