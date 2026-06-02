# stynx-code

An interactive AI coding assistant for the terminal. Multi-provider, tool-using, fast.

```
 ____   _____ __   __ _   _ __  __
/ ___| |_   _|\ \ / /| \ | |\ \/ /
\___ \   | |   \ V / |  \| | \  /
 ___) |  | |    | |  | |\  | /  \
|____/   |_|    |_|  |_| \_|/_/\_\
            c o d e
```

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

The product is `stynx-code`. The command you'll actually type is `stynx`.

## What it is

A self-contained terminal app for getting work done with an LLM. Speaks Anthropic's Claude API natively and any OpenAI-compatible endpoint (DeepSeek, OpenAI, OpenRouter, Together, local Ollama, …) for cheap delegation. Runs an autonomous tool-using loop with bash / file edit / glob / grep / web fetch behind a permission system that prompts you only when you ask it to.

The engine runs concurrent-safe tools in parallel via `tokio::spawn`, sequential otherwise. Context is compacted automatically at 60% token threshold through a 4-stage pipeline (Auto → Micro → Session Memory → Full). File mutations are tracked on an undo stack. Provider overload triggers exponential backoff with up to 3 retries.

Highlights:

- **Modern terminal UI** — sidebar, command palette, model picker, session list, runtime theme switcher (rose-pine, catppuccin, tokyo-night, gruvbox), mouse + bracketed paste, `@`-file mentions, multi-line input, slash-command popover with descriptions, toast notifications, colorized diff renderer for file edits.
- **Intern mode** — Claude as the senior, one or more cheaper OpenAI-compat models (DeepSeek, OpenRouter, OpenAI, custom) as interns. Each intern shows up as its own `delegate_to_<name>` tool the senior can pick from, or call directly via `/intern <name> <task>`.
- **Permission-first** — Normal / Auto-accept / Plan / Bypass modes, per-tool allow rules, in-TUI confirmation modal (no terminal mode-switching).
- **Skills as slash commands** — drop a markdown file under `.claude/skills/` and it appears as `/your-skill`.
- **Sessions** — automatic per-project persistence at `~/.stynx-code/projects/<slug>/session.json`.
- **Hooks** — `session-start`, `pre-tool-use`, `post-tool-use`, `stop` shell hooks for integrations.

> **On `.claude/` vs `.stynx/` directories:** stynx-code reuses `.claude/` paths (`.claude/skills/`, `.claude/.credentials.json`) for Claude Code compatibility — you can run both tools side by side without conflict. stynx-specific data (settings, logs, sessions, benchmark results) lives under `.stynx/`.

## Install

```bash
# from source
git clone https://github.com/maulanasdqn/stynx-code
cd stynx-code
cargo install --path stynx-code

# with nix
nix develop
cargo build --release --workspace
./target/release/stynx
```

The installed binary is **`stynx`**.

## Quickstart

```bash
# interactive TUI (the main mode)
stynx

# one-shot prompt
stynx -p "find every TODO comment under src/ and group by file"

# pipe mode — read stdin as the prompt
echo "explain this file" | stynx

# JSON output for scripting
stynx --json -p "list files modified in the last commit"
```

## Configuration

### Anthropic credentials

Resolved in order:

1. macOS Keychain / Linux libsecret (the Claude Code OAuth token at `Claude Code-credentials`)
2. `~/.stynx/.credentials.json` (also checks `~/.claude/.credentials.json` as fallback)
3. `~/.claude/settings.json` → `auth_token`
4. `ANTHROPIC_API_KEY` env var

Run `/login` inside `stynx` for instructions.

### Interns (OpenAI-compatible providers)

You can configure any number of interns concurrently — DeepSeek and OpenRouter together, mixed providers, whatever you want. Each becomes its own `delegate_to_<name>` tool the senior model can pick from.

**Option 1 — `.env` shorthand** (zero settings.json edits):

```bash
# DeepSeek (legacy single-intern shortcut)
DEEPSEEK_API_KEY=sk-...
INTERN_MODEL=deepseek-chat                    # optional (DEEPSEEK_MODEL also accepted)

# OpenRouter — declare multiple interns via name:model pairs
OPENROUTER_API_KEY=sk-or-...
OPENROUTER_INTERNS=qwen-coder:qwen/qwen3-coder,haiku:anthropic/claude-haiku-4.5,kimi:moonshotai/kimi-k2,gemini-flash:google/gemini-2.5-flash,mimo:mimo-v2.5-pro

# Qwen / Alibaba DashScope (auto-registers a "qwen" intern; multi via QWEN_INTERNS)
QWEN_API_KEY=sk-...
QWEN_MODEL=qwen-plus                         # optional; qwen-max / qwen-turbo / qwen3-coder-plus
# QWEN_INTERNS=qwen-max:qwen-max,qwen-coder:qwen3-coder-plus

# Xiaomi MiMo (standalone provider)
MIMO_API_KEY=sk-...
MIMO_BASE_URL=https://api.xiaomimimo.com/v1
MIMO_MODEL=mimo-v2.5-pro
```

**Option 2 — `interns` array in `settings.json`** (full control):

```json
{
  "interns": [
    {
      "name": "deepseek",
      "provider": "deepseek",
      "model": "deepseek-chat",
      "description": "Cheap general-purpose intern; good for boilerplate and mechanical refactors."
    },
    {
      "name": "qwen-coder",
      "provider": "openrouter",
      "model": "qwen/qwen3-coder",
      "description": "Coding specialist; strong at multi-file edits."
    },
    {
      "name": "qwen-max",
      "provider": "qwen",
      "model": "qwen-max",
      "description": "Complex features, architecture, hard bugs."
    },
    {
      "name": "haiku",
      "provider": "openrouter",
      "model": "anthropic/claude-haiku-4.5",
      "description": "Fast, well-balanced general intern."
    },
    {
      "name": "kimi",
      "provider": "openrouter",
      "model": "moonshotai/kimi-k2",
      "description": "Strong reasoning and long-context tasks."
    },
    {
      "name": "gemini-flash",
      "provider": "openrouter",
      "model": "google/gemini-2.5-flash",
      "description": "Boilerplate, simple edits, formatting."
    },
    {
      "name": "mimo",
      "provider": "openrouter",
      "model": "mimo-v2.5-pro",
      "description": "Mechanical edits, grep/glob grunt work."
    },
    {
      "name": "local",
      "provider": "custom",
      "base_url": "http://localhost:11434/v1",
      "api_key_env": "OLLAMA_API_KEY",
      "model": "qwen2.5-coder:32b",
      "description": "Local Ollama intern; free but slower."
    }
  ]
}
```

`provider` shorthand resolves to:

| provider     | base_url                                                       | default api key env     |
| ------------ | -------------------------------------------------------------- | ----------------------- |
| `deepseek`   | `https://api.deepseek.com/v1`                                  | `DEEPSEEK_API_KEY`      |
| `openrouter` | `https://openrouter.ai/api/v1`                                 | `OPENROUTER_API_KEY`    |
| `openai`     | `https://api.openai.com/v1`                                    | `OPENAI_API_KEY`        |
| `qwen`       | `https://dashscope-intl.aliyuncs.com/compatible-mode/v1`       | `QWEN_API_KEY`          |
| `mimo`       | `https://api.xiaomimimo.com/v1`                                | `MIMO_API_KEY`          |
| `custom`     | (set via `base_url`)                                           | (set via `api_key_env`) |

Copy `.env.example` to `.env` and fill in your API keys. Add `.env` to your `.gitignore` — `stynx` autoloads it on startup.

Base URLs default to the values above. Override via `<PROVIDER>_BASE_URL` env vars — `DEEPSEEK_BASE_URL`, `QWEN_BASE_URL`, `MIMO_BASE_URL`, etc.

At launch you'll see one `· intern ready: <name> (<provider> / <model>)` line per intern that successfully resolved. Interns missing their API key are silently skipped (a `WARN` is emitted to logs).

### Settings file

`~/.stynx/settings.json` (global) and `.stynx/settings.json` (project-local) get merged. `~/.claude/settings.json` is only read for the `auth_token` field (Claude Code OAuth compatibility). Schema:

```json
{
  "model": "claude-sonnet-4-6",
  "max_turns": 200,
  "max_tokens": 8192,
  "effort": "medium",
  "commit_attribution": false,
  "interns": [
    { "name": "deepseek", "provider": "deepseek", "model": "deepseek-chat" }
  ],
  "permissions": {
    "allow": ["bash:cargo *", "read:*"],
    "deny":  ["bash:rm -rf*"]
  },
  "hooks": {
    "SessionStart":  [{ "command": "scripts/session-start.sh" }],
    "PreToolUse":    [{ "matcher": "file_write", "command": "scripts/check.sh" }],
    "PostToolUse":   [{ "command": "scripts/audit.sh" }]
  }
}
```

`commit_attribution: false` (the default) tells the assistant to omit AI/assistant attribution from commits — no `Co-Authored-By:` trailers, no `🤖 Generated with …`. Set it to `true` if you want attribution back.

Run `/config` inside the session to see the merged view.

## Daily flow

| Key            | Action                                       |
|----------------|----------------------------------------------|
| `Enter`        | Submit message                               |
| `Shift+Enter`  | Newline (also `Alt+Enter`, `Ctrl+J`)         |
| `Esc`          | Interrupt streaming · or vim normal mode     |
| `Ctrl+P`       | Command palette                              |
| `Ctrl+S`       | Session list                                 |
| `Ctrl+M`       | Switch model                                 |
| `Ctrl+B`       | Toggle sidebar                               |
| `Ctrl+T`       | Toggle tool block details                    |
| `Shift+Tab`    | Cycle permission mode                        |
| `Shift+↑/↓`    | Scroll messages by line (works while typing) |
| `PgUp/PgDn`    | Scroll by page                               |
| `Ctrl+Alt+U/D` | Scroll messages by half page                 |
| `@`            | File-mention picker                          |
| `/`            | Slash-command popover                        |
| `Ctrl+C`       | Quit                                         |

Mouse scroll wheel works in any terminal with mouse capture.

## Slash commands

```
/help                 show this help
/status               git status
/version              show stynx version
/quit, /exit          exit

/model [name]         switch model
/fast                 toggle fast (haiku) mode
/effort low|medium|high|max    set effort
/think                toggle extended thinking

/mode                 cycle permission mode
/plan [task]          toggle plan mode (read-only tools only)

/compact              summarize older turns to free up context
/cost                 show token usage and cost
/usage                show plan limits (OAuth only)

/diff                 show git diff
/review               review the current diff
/commit               generate a commit message
/init                 create/update CLAUDE.md
/memory               show CLAUDE.md

> `CLAUDE.md` is used (not `STYNX.md`) for interoperability with Claude Code's project memory convention — both tools share the same file.

/add <path>           pin a file into every message
/files                list pinned files
/skills               list available skills
/intern <task>             hand work to the first intern
/intern <name> <task>      hand work to a specific intern by name
/intern-bench [name]       benchmark & rank interns by capability

/session              list / load sessions
/rewind [n]           remove last n exchanges
/undo [n]             restore last n file edits
/export               write transcript to disk
/copy                 copy last response to clipboard

/config               show merged config
/permissions          show allow / deny rules
/login, /logout       credential management

!<command>            run a shell command without leaving the TUI
```

## Intern mode in 30 seconds

```bash
# .env — quick mix of providers
DEEPSEEK_API_KEY=sk-...
OPENROUTER_API_KEY=sk-or-...
OPENROUTER_INTERNS=qwen-coder:qwen/qwen3-coder,haiku:anthropic/claude-haiku-4.5,kimi:moonshotai/kimi-k2,gemini-flash:google/gemini-2.5-flash,mimo:mimo-v2.5-pro
```

```
> refactor every println! in src/foo.rs to tracing::info!; hand it to the qwen-coder intern
```

The senior (Claude) calls `delegate_to_qwen_coder` (or any other registered intern) with a focused task description. The intern runs with a restricted toolset — bash, read, file_write, file_edit, glob, grep — and cannot spawn further sub-agents. It returns `Summary / Files changed / Output`. The senior reviews and integrates.

Direct invocation:

```
/intern                                                # show available interns
/intern list every public function and one-line them    # picks the first intern
/intern qwen-coder write a unit test for util::strip_ansi
```

When multiple interns are configured, the senior picks based on each tool's `description` — so write descriptions that say what each intern is good at (speed, cost, specialty). The intern's transcript is shown as a system message in the conversation, including any tool calls it made.

### Benchmarking interns

Not sure which intern is actually pulling its weight? Run the built-in benchmark:

```
/intern-bench                 # benchmark every configured intern
/intern-bench qwen-coder      # benchmark just one
```

It runs a fixed suite of self-contained tasks (coding, debugging, an algorithm, reasoning, instruction-following, a refactor) against each intern, then has the **main model grade every answer** against a per-task rubric on a 0–10 scale (LLM-as-judge). Each run is timed. You get a ranked leaderboard plus a per-task score matrix:

```
🏁 Intern Benchmark — 3 intern(s) × 6 tasks

Leaderboard (avg score, then speed):
  🥇 #1  deepseek    8.5/10   pass 6/6     42.3s
  🥈 #2  qwen-coder  7.2/10   pass 5/6     55.1s
  🥉 #3  mimo        5.0/10   pass 4/6     88.0s
```

The full report — including the judge's one-line note for every task — is written to `.stynx/intern-bench.md`. The benchmark forces Auto-accept while it runs (the UI is blocked, so a permission prompt would otherwise deadlock it) and restores your previous permission mode afterward.

## Terminal sessions

`bash` is a **persistent shell**, not a series of disposable subshells. The first call boots a long-lived `bash --norc --noprofile` process; every subsequent call runs through the same shell, so `cd`, `export`, sourced files, and function definitions survive across calls.

For long-running processes (dev servers, watchers, log tails), pass `background: true`:

```jsonc
// start a dev server
{"command": "bun run dev", "background": true}        // → "started background process 'bg1'"

// check on it (only new output since last read)
{"status": "bg1"}                                     // → "[running, 12s]\n..."

// read everything it has emitted
{"status": "bg1", "full": true}

// see all background processes
{"list": true}

// stop it
{"kill": "bg1"}
```

Foreground commands have a 120s default timeout (override with `"timeout": <secs>`). If you need longer, use `background: true` instead — a timed-out foreground command leaves the persistent shell in an unknown state.

## Hooks

Each hook command receives the relevant JSON on stdin and writes text to stdout (mixed into the conversation as a system message). Configure under `hooks` in `settings.json`:

- `SessionStart` — runs once when a session begins
- `PreToolUse` — runs before each tool invocation; can short-circuit by exiting non-zero
- `PostToolUse` — runs after each tool invocation; can post-process the result
- `Stop` — runs when the assistant finishes a turn

Each can have an optional `matcher` (substring against tool name) and a required `command`.

## Available Tools

The engine provides a rich set of tools categorized as follows:

- **File I/O**: `read`, `file_write`, `file_edit`
- **Discovery**: `glob`, `grep`, `tool_search`
- **Shell**: `bash` (persistent session), `powershell`
- **Web**: `web_fetch`, `web_search`
- **Tasks**: `todo_read`, `todo_write`
- **Background tasks**: `task_create`, `task_get`, `task_list`, `task_stop`, `task_update`, `task_output`
- **Scheduling**: `cron_create`, `cron_delete`
- **Interaction**: `ask_user_question`
- **Integration**: MCP (dynamic tools loaded per server), `lsp`, `remote_trigger`
- **Skills**: `skill` invocation
- **Teams**: `team_create`, `team_delete`
- **Workspace**: `enter_worktree`, `exit_worktree`
- **Misc**: `notebook_edit`, `repl`, `send_message`, `sleep`, `synthetic_output`, `plan_mode`, `brief`, `config`

## How it works

### Engine loop

The QueryEngine runs up to N turns (default 200, configurable via `max_turns`). Each turn:

1. Sends the conversation to the provider (with read-only tools only in plan mode)
2. Streams the response and extracts tool calls
3. Runs pre-tool hooks for each tool
4. Executes concurrent-safe tools in parallel (via tokio::spawn); non-concurrent tools sequentially
5. Runs post-tool hooks and collects results
6. Adds results back to conversation
7. Repeats if the model calls tools, or returns if the model stopped

At 60% token threshold, the engine automatically invokes the compactor to free up context.

### Context compaction

When tokens exceed 60% of the limit, the engine builds a text summary of the conversation so far and sends it to the provider for summarization. If the provider call fails, it falls back to keeping the last 6 messages intact. The `stynx-code-compact` crate provides a deeper 4-stage pipeline (Auto → Micro → Session Memory → Full) available for additional compaction strategies.

### File edit undo stack

Every file write/edit is tracked on an undo stack. Use `/undo [n]` to restore the last n file mutations.

## Architecture

The project is a 17-crate Rust workspace. Most crates follow Clean Architecture with `domain/`, `application/`, and `infrastructure/` layers where it adds value.

- `stynx-code-errors`: Common application error types and results.
- `stynx-code-types`: Core traits and structs for tools, providers, messages, and permissions.
- `stynx-code-tools`: Implements 39 tools — 36 built-in (file I/O, shell, web, tasks, scheduling, LSP) plus 3 MCP dynamic tool wrappers.
- `stynx-code-provider`: Anthropic SSE streaming and OpenAI-compatible provider integrations.
- `stynx-code-engine`: Core query engine — multi-turn agentic loop, tool execution, context compaction, hooks, undo stack.
- `stynx-code-server`: Axum HTTP API for headless operation.
- `stynx-code`: Main executable — CLI parsing, TUI orchestration, agent/intern delegation, skill loading, system prompt assembly.
- `stynx-code-auth`: OAuth PKCE and API key credential resolution.
- `stynx-code-permission`: Interactive and configuration-driven permission gating.
- `stynx-code-commands`: Slash-command expansion and file reference resolution in the TUI.
- `stynx-code-memory`: Per-project session persistence.
- `stynx-code-config`: Global and project-specific settings, hooks, intern config.
- `stynx-code-services`: Analytics, LSP bridge, rate limiting, token estimation, diagnostics, notifications.
- `stynx-code-compact`: Conversation summarization pipeline (Auto → Micro → Session Memory → Full).
- `stynx-code-bridge`: External integration bridge (stdio, WebSocket, JWT).
- `stynx-code-plugins`: Plugin lifecycle — subprocess plugins, builtins, config.
- `stynx-code-tui`: `ratatui` terminal UI — state management, rendering, themes, widgets.

## License

MIT
