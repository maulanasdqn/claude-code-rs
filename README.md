# stynx-code

An interactive AI coding assistant for the terminal. Multi-provider, tool-using, fast.

```
  ____ _____ __   ___   __ __
 / ___|_   _\ \ / / \ | \ \ / /
 \___ \ | |  \ V /|  \| |\ V /
  ___) || |   | | | |\  | | |
 |____/ |_|   |_| |_| \_| |_|
               c o d e
```

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

The product is `stynx-code`. The command you'll actually type is `stynx`.

## What it is

A self-contained terminal app for getting work done with an LLM. Speaks Anthropic's Claude API natively and any OpenAI-compatible endpoint (DeepSeek, OpenAI, OpenRouter, Together, local Ollama, …) for cheap delegation. Runs an autonomous tool-using loop with bash / file edit / glob / grep / web fetch behind a permission system that prompts you only when you ask it to.

Highlights:

- **Modern terminal UI** — sidebar, command palette, model picker, session list, runtime theme switcher (rose-pine, catppuccin, tokyo-night, gruvbox), mouse + bracketed paste, `@`-file mentions, multi-line input, slash-command popover with descriptions, toast notifications, colorized diff renderer for file edits.
- **Intern mode** — Claude as the senior, DeepSeek (or any OpenAI-compat model) as the intern. Senior delegates focused subtasks via the `delegate_to_intern` tool or `/intern <task>` slash command.
- **Permission-first** — Normal / Auto-accept / Plan / Bypass modes, per-tool allow rules, in-TUI confirmation modal (no terminal mode-switching).
- **Skills as slash commands** — drop a markdown file under `.claude/skills/` and it appears as `/your-skill`.
- **Sessions** — automatic per-project persistence at `~/.stynx-code/projects/<slug>/session.json`.
- **Hooks** — `session-start`, `pre-tool-use`, `post-tool-use`, `stop` shell hooks for integrations.

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
2. `~/.claude/.credentials.json`
3. `~/.claude/settings.json` → `auth_token`
4. `ANTHROPIC_API_KEY` env var

Run `/login` inside `stynx` for instructions.

### Intern (OpenAI-compatible) provider

Set in a project-local `.env` or shell env:

```bash
DEEPSEEK_API_KEY=sk-...
# optional overrides:
DEEPSEEK_MODEL=deepseek-chat                  # or deepseek-reasoner
DEEPSEEK_BASE_URL=https://api.deepseek.com/v1
```

Add `.env` to your `.gitignore` — `stynx` autoloads it on startup.

At launch you'll see `· intern ready (deepseek-chat)` in the banner if the key was picked up. If not, the `/intern` command and the `delegate_to_intern` tool are simply not registered.

### Settings file

`~/.claude/settings.json` (global) and project-local settings get merged. Schema:

```json
{
  "model": "claude-sonnet-4-6",
  "max_turns": 30,
  "max_tokens": 8192,
  "effort": "medium",
  "commit_attribution": false,
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

/add <path>           pin a file into every message
/files                list pinned files
/skills               list available skills
/intern <task>        hand work to the intern model

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
# .env
DEEPSEEK_API_KEY=sk-...
```

```
> refactor every println! in src/foo.rs to tracing::info!; hand it to the intern
```

The senior (Claude) calls `delegate_to_intern` with a focused task description. The intern (DeepSeek) runs with a restricted toolset — bash, read, file_write, file_edit, glob, grep — and cannot spawn further sub-agents. It returns `Summary / Files changed / Output`. The senior reviews and integrates.

Direct invocation:

```
/intern list every public function in stynx-code-tui and one-line what each does
```

The intern's transcript is shown as a system message in the conversation, including any tool calls it made.

## Hooks

Each hook command receives the relevant JSON on stdin and writes text to stdout (mixed into the conversation as a system message). Configure under `hooks` in `settings.json`:

- `SessionStart` — runs once when a session begins
- `PreToolUse` — runs before each tool invocation; can short-circuit by exiting non-zero
- `PostToolUse` — runs after each tool invocation; can post-process the result
- `Stop` — runs when the assistant finishes a turn

Each can have an optional `matcher` (substring against tool name) and a required `command`.

## Architecture

Every crate follows Clean Architecture with `domain/`, `application/`, and `infrastructure/` layers.

```
stynx-code/                main binary, app loop, CLI surface
stynx-code-types/          shared types: Provider, Tool, Message, Conversation
stynx-code-errors/         AppError + AppResult
stynx-code-config/         settings loader
stynx-code-auth/           credential resolution
stynx-code-provider/       AnthropicProvider + OpenAiProvider
stynx-code-engine/         streaming tool-use loop
stynx-code-tools/          built-in tools + MCP loader
stynx-code-permission/     allow/deny + prompt bridge to TUI
stynx-code-memory/         per-project session persistence
stynx-code-commands/       slash-command handlers
stynx-code-compact/        conversation summarization
stynx-code-coordinator/    parallel sub-agent orchestration
stynx-code-skills/         user-defined skill loading
stynx-code-plugins/        plugin host (skills + MCP)
stynx-code-bridge/         server bridge for frontends
stynx-code-server/         HTTP/SSE server
stynx-code-services/       cross-cutting services (tips, telemetry, …)
stynx-code-tui/            ratatui-based terminal UI
```

## License

MIT
