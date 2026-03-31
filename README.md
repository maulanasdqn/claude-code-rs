# claude-rust

A minimal Rust reimplementation of [Claude Code](https://docs.anthropic.com/en/docs/claude-code) -- the agentic tool-use loop backed by the Anthropic streaming API.

The TypeScript original has ~1,900 files. This project distills it to its essence: a multi-crate Rust workspace that streams responses from the Anthropic Messages API, detects tool-use requests, executes tools locally, feeds results back, and loops until the model is done.

[![crates.io](https://img.shields.io/crates/v/claude-rust.svg)](https://crates.io/crates/claude-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Architecture

Every crate follows **Clean Architecture** with `domain/`, `application/`, and `infrastructure/` layers. All files are under 200 LOC.

```
claude-rust-auth        - Credential resolution (macOS Keychain OAuth + API key fallback)
claude-rust-errors      - AppError enum, axum IntoResponse impl
claude-rust-types       - Shared traits (Tool, Provider, PermissionChecker) and message types
claude-rust-tools       - Tool implementations (BashTool, ReadTool) and ToolRegistry
claude-rust-provider    - Anthropic HTTP + SSE streaming client
claude-rust-engine      - Agentic tool-use loop with streaming callbacks and retry
claude-rust-permission  - Interactive terminal permission checker (y/n prompts)
claude-rust-memory      - Session persistence (JSON files in ~/.claude-code-rs/sessions/)
claude-rust-commands    - Slash commands (/help, /clear, /model, /compact) and @file references
claude-rust             - Interactive terminal REPL (the main binary)
claude-rust-server      - axum HTTP server (POST /chat, GET /health)
```

### Dependency DAG

```
claude-rust-errors
  <- claude-rust-types
       <- claude-rust-tools
       <- claude-rust-permission
  <- claude-rust-auth
       <- claude-rust-provider
  <- claude-rust-memory
  <- claude-rust-commands
            <- claude-rust-engine
                 <- claude-rust (CLI)
                 <- claude-rust-server
```

### Core Loop

```
user input
  -> expand @file references
  -> build Conversation
  -> provider.stream()
  -> accumulate StreamEvents
  -> if stop_reason == ToolUse -> check permission -> execute tools -> append results -> loop
  -> if overloaded -> retry with exponential backoff (up to 3 attempts)
  -> else -> return final text -> auto-save session
```

Bounded by `max_turns` (default 20). Tool errors are sent back as `is_error: true` so the model can self-correct.

## Authentication

Credentials are resolved automatically in this order:

1. **`ANTHROPIC_API_KEY` environment variable** -- uses `api.anthropic.com` with `x-api-key` header.
2. **macOS Keychain** -- reads the OAuth token stored by Claude Code (service: `Claude Code-credentials`). Uses `api.claude.ai` with `Authorization: Bearer` header.

If you already have Claude Code installed and logged in, it just works -- no extra configuration needed.

## Features

### Interactive CLI

- Session persistence with resume on startup
- Slash commands: `/help`, `/clear`, `/compact`, `/model <name>`, `/quit`, `/exit`
- `@file` references: type `@Cargo.toml` to inject file contents into your message
- Interactive permission prompts for dangerous tool calls (bash)
- Streaming output with spinner animation
- Retry with exponential backoff on API overload

### HTTP Server

- `GET /health` -- health check
- `POST /chat` -- send messages and get responses with full conversation history

## Built-in Tools

| Tool | Permission | Description |
|------|-----------|-------------|
| `bash` | Dangerous | Execute a shell command and return stdout/stderr |
| `read` | ReadOnly | Read a file with line numbers (supports offset and limit) |

## Prerequisites

- Rust 1.85+ (2024 edition)
- One of:
  - An [Anthropic API key](https://console.anthropic.com/), or
  - An existing Claude Code installation (credentials are read from the macOS Keychain)

## Quick Start

### Install from crates.io

```bash
cargo install claude-rust
```

### Interactive CLI

```bash
# Using your existing Claude Code session (no env var needed)
claude-rust

# Or with an explicit API key
ANTHROPIC_API_KEY=sk-ant-... claude-rust
```

This starts an interactive REPL. Type a message and press Enter. The assistant streams its response to the terminal. Tool calls are displayed inline with their output.

Commands:
- `/help` -- list available commands
- `/model` -- show current model
- `/model claude-opus-4-6` -- switch model
- `/clear` -- clear conversation history
- `/compact` -- compact conversation to save context
- `/quit` or `/exit` -- end the session
- `Ctrl+C` -- abort

### @file References

Include file contents in your message by referencing them with `@`:

```
> What does @Cargo.toml configure?
> Explain @src/main.rs
```

Files over 100KB are skipped. Email addresses (user@example.com) are not expanded.

### HTTP Server

```bash
cargo run -p claude-rust-server
```

Starts an axum server on port 3000.

```bash
curl http://localhost:3000/health

curl -X POST http://localhost:3000/chat \
  -H 'Content-Type: application/json' \
  -d '{
    "messages": [{"role": "user", "content": "List files in the current directory"}],
    "system": "You are a helpful assistant."
  }'
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ANTHROPIC_API_KEY` | (auto-detected) | API key. Falls back to macOS Keychain if unset |
| `ANTHROPIC_BASE_URL` | auto (`api.anthropic.com` or `api.claude.ai`) | Override the API base URL |
| `MODEL` | `claude-sonnet-4-6` (OAuth) / `anthropic/claude-sonnet-4-20250514` (API key) | Model to use |
| `RUST_LOG` | `info,claude_rust_provider=debug` (server) / `warn` (cli) | Log level filter |

## Project Structure

```
.
├── Cargo.toml                    Workspace root
├── claude-rust/                  CLI binary
│   └── src/
│       ├── main.rs
│       └── infrastructure/
│           ├── terminal.rs       Terminal I/O helpers
│           └── event_renderer.rs Engine event display
├── claude-rust-auth/
│   └── src/
│       ├── domain/credential.rs
│       ├── application/resolve_credential.rs
│       └── infrastructure/keychain_provider.rs
├── claude-rust-errors/
│   └── src/lib.rs                AppError, IntoResponse
├── claude-rust-types/
│   └── src/domain/
│       ├── message.rs            Message, ContentBlock, Conversation
│       ├── tool.rs               Tool trait, PermissionLevel
│       ├── provider.rs           Provider trait, StreamEvent
│       └── permission.rs         PermissionChecker trait, AllowAll
├── claude-rust-tools/
│   └── src/
│       ├── application/registry.rs
│       └── infrastructure/
│           ├── bash_tool.rs
│           └── read_tool.rs
├── claude-rust-provider/
│   └── src/infrastructure/
│       ├── anthropic_provider.rs
│       ├── request_builder.rs
│       └── sse_parser.rs
├── claude-rust-engine/
│   └── src/
│       ├── domain/engine_event.rs
│       └── application/query_engine.rs
├── claude-rust-permission/
│   └── src/infrastructure/terminal_checker.rs
├── claude-rust-memory/
│   └── src/
│       ├── domain/session_repository.rs
│       ├── application/{save,load}_session.rs
│       └── infrastructure/file_session_repository.rs
├── claude-rust-commands/
│   └── src/
│       ├── domain/command.rs
│       ├── application/
│       │   ├── parse_command.rs
│       │   ├── execute_command.rs
│       │   └── expand_references.rs
│       ├── infrastructure/handlers/
│       └── tests/
└── claude-rust-server/
    └── src/
        ├── main.rs
        └── infrastructure/http/
            ├── routes.rs
            ├── handlers.rs
            └── dto.rs
```

## Key Traits

All major components are behind traits with `Arc<dyn Trait>` for dependency injection.

### Tool

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn permission_level(&self) -> PermissionLevel;
    async fn execute(&self, input: Value) -> AppResult<String>;
}
```

### Provider

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn stream(
        &self,
        conversation: &Conversation,
        tools: &[Value],
    ) -> AppResult<BoxStream<'static, StreamEvent>>;
}
```

### PermissionChecker

```rust
#[async_trait]
pub trait PermissionChecker: Send + Sync {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision>;
}
```

The CLI uses `InteractivePermissionChecker` which prompts y/n on stderr. The server uses `AllowAll`. Replace with your own logic to gate dangerous operations.

## Adding a New Tool

1. Create a struct implementing `claude_rust_types::Tool` in `claude-rust-tools/src/infrastructure/`.
2. Register it in the binary that needs it:

```rust
registry.register(Arc::new(MyNewTool));
```

The tool's `input_schema()` is sent to the API automatically, and the engine handles calling `execute()` when the model requests it.

## Adding a New Provider

Implement `claude_rust_types::Provider` to target a different LLM API. The engine is provider-agnostic -- it only cares about the `StreamEvent` stream.

## Building

```bash
cargo build --workspace
cargo build --workspace --release
cargo test --workspace
cargo check --workspace
```

## License

MIT
