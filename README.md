# claude-code-rs

A minimal Rust reimplementation of [Claude Code](https://docs.anthropic.com/en/docs/claude-code) -- the agentic tool-use loop backed by the Anthropic streaming API.

The TypeScript original has ~1,900 files. This project distills it to its essence: a multi-crate Rust workspace that streams responses from the Anthropic Messages API, detects tool-use requests, executes tools locally, feeds results back, and loops until the model is done.

## Architecture

```
cc-auth      - Credential resolution (macOS Keychain OAuth + API key fallback)
cc-errors    - AppError enum, axum IntoResponse impl
cc-types     - Shared traits (Tool, Provider, PermissionChecker) and message types
cc-tools     - Tool implementations (BashTool, ReadTool) and ToolRegistry
cc-provider  - Anthropic HTTP + SSE streaming client
cc-engine    - Agentic tool-use loop with streaming callbacks
cc-cli       - Interactive terminal REPL (like Claude Code)
cc-server    - axum HTTP server (POST /chat, GET /health)
```

### Dependency DAG

```
cc-errors
  <- cc-types
       <- cc-tools
  <- cc-auth
       <- cc-provider
            <- cc-engine
                 <- cc-cli
                 <- cc-server
```

### Core Loop

```
user input
  -> build Conversation
  -> provider.stream()
  -> accumulate StreamEvents
  -> if stop_reason == ToolUse -> execute tools via registry -> append results -> loop
  -> else -> return final text
```

Bounded by `max_turns` (default 20). Tool errors are sent back as `is_error: true` so the model can self-correct.

## Authentication

Credentials are resolved automatically in this order:

1. **`ANTHROPIC_API_KEY` environment variable** -- uses `api.anthropic.com` with `x-api-key` header.
2. **macOS Keychain** -- reads the OAuth token stored by Claude Code (service: `Claude Code-credentials`). Uses `api.claude.ai` with `Authorization: Bearer` header.

If you already have Claude Code installed and logged in, it just works -- no extra configuration needed.

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

The default `AllowAll` implementation permits every tool call. Replace it with your own logic to gate dangerous operations.

## Built-in Tools

| Tool | Permission | Description |
|------|-----------|-------------|
| `bash` | Dangerous | Execute a shell command and return stdout/stderr |
| `read` | ReadOnly | Read a file with line numbers (supports offset and limit) |

## Prerequisites

- Rust 1.75+ (2024 edition)
- One of:
  - An [Anthropic API key](https://console.anthropic.com/), or
  - An existing Claude Code installation (credentials are read from the macOS Keychain)

## Quick Start

### Interactive CLI

```bash
# Using your existing Claude Code session (no env var needed)
cargo run -p cc-cli

# Or with an explicit API key
ANTHROPIC_API_KEY=sk-ant-... cargo run -p cc-cli
```

This starts an interactive REPL. Type a message and press Enter. The assistant streams its response to the terminal. Tool calls are displayed inline with their output.

Commands:
- `/quit` or `/exit` -- end the session
- `Ctrl+C` -- abort

### HTTP Server

```bash
cargo run -p cc-server
```

Starts an axum server on port 3000.

```bash
# Health check
curl http://localhost:3000/health

# Chat
curl -X POST http://localhost:3000/chat \
  -H 'Content-Type: application/json' \
  -d '{
    "messages": [{"role": "user", "content": "List files in the current directory"}],
    "system": "You are a helpful assistant."
  }'
```

Response format:

```json
{
  "response": "The final assistant text",
  "messages": [ ... full conversation history including tool calls ... ]
}
```

## Configuration

All configuration is via environment variables.

| Variable | Default | Description |
|----------|---------|-------------|
| `ANTHROPIC_API_KEY` | (auto-detected) | API key. Falls back to macOS Keychain if unset |
| `ANTHROPIC_BASE_URL` | auto (`api.anthropic.com` or `api.claude.ai`) | Override the API base URL |
| `ANTHROPIC_MODEL` | `claude-sonnet-4-20250514` | Model to use |
| `RUST_LOG` | `info,cc_provider=debug` (server) / `warn` (cli) | Log level filter |

## Building

```bash
# Build all crates
cargo build --workspace

# Build in release mode
cargo build --workspace --release

# Run tests
cargo test --workspace

# Check without building
cargo check --workspace
```

## Adding a New Tool

1. Create a struct implementing `cc_types::Tool` in `cc-tools/src/`.
2. Register it in the binary that needs it:

```rust
registry.register(Arc::new(MyNewTool));
```

That's it. The tool's `input_schema()` is sent to the API automatically, and the engine handles calling `execute()` when the model requests it.

## Adding a New Provider

Implement `cc_types::Provider` to target a different LLM API. The engine is provider-agnostic -- it only cares about the `StreamEvent` stream.

## Project Structure

```
.
├── Cargo.toml           Workspace root
├── cc-auth/
│   └── src/lib.rs       Credential resolution (Keychain + env)
├── cc-errors/
│   └── src/lib.rs       AppError, IntoResponse
├── cc-types/
│   └── src/
│       ├── lib.rs       Re-exports
│       ├── message.rs   Message, ContentBlock, Conversation
│       ├── tool.rs      Tool trait, PermissionLevel
│       ├── provider.rs  Provider trait, StreamEvent, StopReason
│       └── permission.rs PermissionChecker trait, AllowAll
├── cc-tools/
│   └── src/
│       ├── lib.rs       Re-exports
│       ├── bash.rs      BashTool
│       ├── read.rs      ReadTool
│       └── registry.rs  ToolRegistry
├── cc-provider/
│   └── src/
│       ├── lib.rs       Re-exports
│       ├── anthropic.rs AnthropicProvider
│       └── stream.rs    SSE parser
├── cc-engine/
│   └── src/lib.rs       QueryEngine agentic loop
├── cc-cli/
│   └── src/main.rs      Interactive terminal REPL
└── cc-server/
    └── src/main.rs      axum HTTP server
```

## Design Decisions

- **Max 200 LOC per file.** Keeps each unit small and reviewable.
- **Trait-based DI with `Arc<dyn Trait>`.** Provider, tools, and permission checking are all swappable at runtime.
- **Streaming callbacks.** The engine emits `EngineEvent`s during execution so the CLI can print text as it arrives and show tool activity in real time.
- **No framework magic.** Direct use of reqwest for HTTP, manual SSE parsing, explicit wiring. Easy to follow and debug.
- **Tool errors as messages.** When a tool fails, the error is sent back to the model as a tool result with `is_error: true`, allowing the model to recover or try a different approach.
- **Automatic credential resolution.** Reuses your existing Claude Code OAuth session from the macOS Keychain, so no manual API key setup is needed.

## License

MIT
