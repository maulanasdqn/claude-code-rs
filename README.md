# claude-rust

A Rust reimplementation of [Claude Code](https://docs.anthropic.com/en/docs/claude-code) -- the agentic tool-use loop backed by the Anthropic streaming API.

The TypeScript original has ~1,900 files. This project distills it to its essence: a multi-crate Rust workspace that streams responses from the Anthropic Messages API, detects tool-use requests, executes tools locally, feeds results back, and loops until the model is done.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Architecture

Every crate follows **Clean Architecture** with `domain/`, `application/`, and `infrastructure/` layers.

```
claude-rust-auth        - Credential resolution (macOS Keychain / Linux keyring OAuth + API key fallback)
claude-rust-config      - Settings loading and merging (global + project)
claude-rust-errors      - AppError enum, axum IntoResponse impl
claude-rust-types       - Shared traits (Tool, Provider, PermissionChecker) and message types
claude-rust-tools       - Tool implementations and ToolRegistry (+ MCP support)
claude-rust-provider    - Anthropic HTTP + SSE streaming client with thinking support
claude-rust-engine      - Agentic tool-use loop with streaming callbacks, retry, and sub-agents
claude-rust-permission  - Config-aware permission checker with interactive prompts and skill scoping
claude-rust-memory      - Session persistence (JSON files)
claude-rust-commands    - Slash commands and @file references
claude-rust             - Interactive terminal REPL (the main binary)
claude-rust-server      - axum HTTP server (POST /chat, GET /health)
```

### Core Loop

```
user input
  -> expand @file references + image attachments
  -> build Conversation
  -> provider.stream() (with extended thinking if enabled)
  -> accumulate StreamEvents
  -> if stop_reason == ToolUse -> check permission -> execute tools -> append results -> loop
  -> if overloaded -> retry with exponential backoff (up to 3 attempts)
  -> else -> return final text -> auto-save session
```

Bounded by `max_turns` (default 20). Tool errors are sent back as `is_error: true` so the model can self-correct.

## Authentication

Credentials are resolved automatically in this order:

1. **`ANTHROPIC_API_KEY` environment variable** -- uses `api.anthropic.com` with `x-api-key` header.
2. **System Keychain** -- reads the OAuth token stored by Claude Code. Uses `api.claude.ai` with `Authorization: Bearer` header. Works on macOS (Keychain) and Linux (secret-tool/keyring).

If you already have Claude Code installed and logged in, it just works -- no extra configuration needed.

## Features

### Interactive CLI

- Rich terminal UI with bordered input box, permission mode badges, and git branch display
- Session persistence with resume on startup
- Streaming output with `indicatif` progress spinners and task progress display
- Vi mode editing (toggle with `/vim`)
- Command autocomplete with dropdown suggestions
- History search (`Ctrl+R`)
- Image paste from clipboard (`Ctrl+V`) with `[Image #N]` markers
- Background task execution (`Ctrl+B`)
- Stash/restore input buffer (`Ctrl+S`)
- `@file` references: type `@Cargo.toml` to inject file contents
- Interactive permission prompts for dangerous tool calls
- Retry with exponential backoff on API overload
- Extended thinking support (`/think`)

### Slash Commands

| Command | Description |
|---------|-------------|
| `/help` | List available commands |
| `/model [name]` | Show or switch model (interactive picker if no arg) |
| `/fast` | Toggle fast mode (Haiku) |
| `/mode` | Cycle permission mode (Normal / Auto-accept / Plan / Bypass) |
| `/plan [task]` | Enter plan mode for read-only exploration |
| `/think` | Toggle extended thinking |
| `/effort [level]` | Set effort level (low / medium / high / max) |
| `/clear` | Clear conversation history |
| `/compact` | Compact conversation to save context |
| `/diff` | Show git diff |
| `/status` | Show git status |
| `/review` | Review uncommitted changes |
| `/commit` | Commit staged changes |
| `/memory` | Show CLAUDE.md files |
| `/export` | Export conversation to markdown |
| `/rewind [n]` | Rewind conversation by N messages |
| `/add <path>` | Pin a file to all future messages |
| `/files` | List pinned files |
| `/config` | Show current settings |
| `/permissions` | Show permission rules |
| `/usage` | Show plan usage (OAuth only) |
| `/doctor` | Diagnostic checks |
| `/vim` | Toggle vi mode |
| `/copy` | Copy last response to clipboard |
| `/login` / `/logout` | Manage authentication |
| `/version` | Show version |
| `/quit` or `/exit` | End the session |

### Custom Skills & Commands

Load custom slash commands from markdown files with YAML frontmatter:

**Skills** (`~/.claude/skills/` or `.claude/skills/`):
```
~/.claude/skills/
  my-skill/
    SKILL.md       # Directory format (preferred)
```

**Legacy Commands** (`~/.claude/commands/` or `.claude/commands/`):
```
~/.claude/commands/
  smart-push.md    # Standalone .md format
  my-tool/
    SKILL.md       # Directory format also supported
```

#### Frontmatter Format

```yaml
---
name: smart-push
description: Intelligent git workflow with grouped commits
argument-hint: [optional commit message]
allowed-tools: Bash(git add:*), Bash(git status:*), Bash(git commit:*), Bash(git push:*)
when_to_use: Use when the user wants to commit and push changes
user-invocable: true
---

Your prompt template here. Use $ARGUMENTS for user-provided arguments.
```

- **`allowed-tools`**: Auto-approve matching tool calls during skill execution
- **`when_to_use`**: Included in system prompt so the model can suggest the skill
- **`user-invocable`**: Set to `false` to hide from autocomplete (default: `true`)

### Built-in Tools

| Tool | Permission | Description |
|------|-----------|-------------|
| `bash` | Dangerous | Execute a shell command |
| `read` | ReadOnly | Read a file with line numbers |
| `file_write` | Dangerous | Create or overwrite a file |
| `file_edit` | Dangerous | Find-and-replace edit in a file |
| `glob` | ReadOnly | Find files matching a glob pattern |
| `grep` | ReadOnly | Search file contents with regex |
| `web_fetch` | Dangerous | Fetch a URL and extract content |
| `web_search` | Dangerous | Search the web |
| `ask_user_question` | ReadOnly | Ask the user a question |
| `todo_write` | ReadOnly | Create/update task list with progress display |
| `todo_read` | ReadOnly | Read current task list |
| `enter_plan_mode` / `exit_plan_mode` | ReadOnly | Plan mode transitions |
| `agent` | Dangerous | Spawn an autonomous sub-agent |
| `explore` | ReadOnly | Spawn a read-only exploration sub-agent |
| MCP tools | Varies | External tools via Model Context Protocol |

### Configuration

Settings are loaded from `~/.claude/settings.json` (global) and `.claude/settings.json` (project), with project overriding global:

```json
{
  "model": "claude-sonnet-4-6",
  "max_turns": 20,
  "max_tokens": 16384,
  "permissions": {
    "allow": ["read", "glob", "grep", "bash(git *)"],
    "deny": ["bash(rm -rf *)"]
  },
  "hooks": {
    "PreToolUse": [{ "matcher": "bash", "command": "echo $CLAUDE_TOOL_INPUT" }],
    "PostToolUse": [{ "command": "notify-send 'Tool done'" }],
    "SessionStart": [{ "command": "echo 'Session started'" }],
    "Stop": [{ "command": "echo 'Session ended'" }]
  }
}
```

### CLAUDE.md

Project instructions are loaded hierarchically from `CLAUDE.md` and `.claude/CLAUDE.md` files, walking up from the current directory to `~/.claude/CLAUDE.md`. All found files are concatenated into the system prompt.

### MCP (Model Context Protocol)

External tool servers are configured via `.mcp.json`, `.claude/mcp.json`, or `~/.claude/mcp.json`:

```json
{
  "mcpServers": {
    "my-server": {
      "command": "npx",
      "args": ["-y", "my-mcp-server"],
      "env": { "API_KEY": "..." }
    }
  }
}
```

### HTTP Server

- `GET /health` -- health check
- `POST /chat` -- send messages and get responses

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ANTHROPIC_API_KEY` | (auto-detected) | API key. Falls back to system keychain if unset |
| `ANTHROPIC_BASE_URL` | auto | Override the API base URL |
| `MODEL` | `claude-sonnet-4-6` | Model to use |
| `RUST_LOG` | `warn` (cli) / `info` (server) | Log level filter |

## Prerequisites

- Rust 1.85+ (2024 edition)
- One of:
  - An [Anthropic API key](https://console.anthropic.com/), or
  - An existing Claude Code installation (credentials are read from the system keychain)

## Quick Start

### Install from source

```bash
git clone https://github.com/maulanasdqn/claude-rust.git
cd claude-rust
cargo install --path claude-rust
```

### Run

```bash
# Using your existing Claude Code session (no env var needed)
claude-rust

# Or with an explicit API key
ANTHROPIC_API_KEY=sk-ant-... claude-rust
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Submit message |
| `Shift+Enter` | New line |
| `Ctrl+C` | Cancel current operation |
| `Ctrl+R` | Search history |
| `Ctrl+V` | Paste image from clipboard |
| `Ctrl+B` | Run current input as background task |
| `Ctrl+S` | Stash/restore input buffer |
| `Up/Down` | Navigate history / autocomplete suggestions |
| `Tab` | Accept autocomplete suggestion |
| `Esc` | Dismiss suggestions |

## Building

```bash
cargo build --workspace
cargo build --workspace --release
cargo test --workspace
```

## License

MIT
