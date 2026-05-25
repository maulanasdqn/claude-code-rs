# stynx-code

An interactive AI coding assistant for the terminal — multi-provider, tool-using, fast, and yours.

```
  ____ _____ __   ___   __ __
 / ___|_   _\ \ / / \ | \ \ / /
 \___ \ | |  \ V /|  \| |\ V /
  ___) || |   | | | |\  | | |
 |____/ |_|   |_| |_| \_| |_|
               c o d e
```

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## What it is

`stynx-code` is a self-contained TUI for getting work done with an LLM. It speaks Anthropic's Claude API natively and any OpenAI-compatible endpoint (DeepSeek, OpenAI, OpenRouter, Together, local Ollama, …) for cheap delegation. It runs an autonomous tool-using loop with bash / file edit / glob / grep / web fetch and a permission system that prompts you only when you want it to.

Highlights:

- **Modern terminal UI** — sidebar, command palette, model picker, session list, theme switcher (rose-pine, catppuccin, tokyo-night, gruvbox), mouse + bracketed paste, `@`-file mentions, multi-line input, slash-command popover, toast notifications, diff renderer for file edits.
- **Intern mode** — Claude as the senior, DeepSeek (or any OpenAI-compat model) as the intern. Senior delegates focused subtasks via the `delegate_to_intern` tool or `/intern <task>` slash command.
- **Permission-first** — Normal / Auto-accept / Plan / Bypass modes. Per-tool allow rules. Inline confirmation modal — no terminal mode-switching.
- **First-class skills** — drop a markdown file under `.claude/skills/` and it shows up as a slash command.
- **Sessions** — automatic per-project persistence at `~/.stynx-code/projects/<slug>/session.json`.
- **Hooks** — `session-start`, `pre-tool-use`, `post-tool-use` shell hooks for integrations.

## Install

```bash
# from source, with cargo
cargo install --path stynx-code

# or with nix
nix develop
cargo build --release --workspace
```

The binary is `stynx-code`.

## Quickstart

```bash
# interactive TUI (the main mode)
stynx-code

# one-shot prompt
stynx-code -p "find every TODO comment under src/ and group by file"

# pipe mode — read stdin as the prompt
echo "explain this file" | stynx-code

# JSON output for scripting
stynx-code --json -p "list files modified in the last commit"
```

## Configuration

**Anthropic credentials** — uses Claude Code's OAuth token (`~/.claude/`) if present, falls back to `ANTHROPIC_API_KEY`. Run `/login` for instructions.

**Intern (OpenAI-compatible) provider** — set in `.env` at the project root or as shell env:

```bash
DEEPSEEK_API_KEY=sk-...
# optional overrides:
DEEPSEEK_MODEL=deepseek-chat                  # default
DEEPSEEK_BASE_URL=https://api.deepseek.com/v1
```

Add `.env` to your `.gitignore`.

**Settings file** — `~/.config/stynx-code/settings.json` (or project-local `.stynx/settings.json`) for permission rules, model defaults, hooks. Run `/config` to see the merged view.

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
| `@`            | File-mention picker                          |
| `/`            | Slash-command popover                        |
| `Ctrl+C`       | Quit                                         |

Mouse scroll wheel works in any terminal with mouse capture.

## Slash commands

`/help`, `/status`, `/version`, `/quit`, `/exit`
`/model [name]`, `/fast`, `/effort low|medium|high|max`, `/think`
`/mode`, `/plan [task]`
`/compact`, `/cost`, `/usage`
`/diff`, `/status`, `/review`, `/commit`
`/memory`, `/init`, `/add <path>`, `/files`, `/skills`
`/session`, `/rewind [n]`, `/export`, `/copy`, `/undo [n]`
`/config`, `/permissions`
`/intern <task>` — hand work to the intern model directly
`!<command>` — run a shell command without leaving the TUI

## Architecture

Every crate follows Clean Architecture with `domain/`, `application/`, and `infrastructure/` layers.

```
stynx-code/                main binary, app loop, CLI surface
stynx-code-types/          shared types: Provider, Tool, Message, Conversation
stynx-code-errors/         AppError + AppResult
stynx-code-config/         settings loader, theme/keybind config
stynx-code-auth/           credential resolution (Anthropic OAuth / env keys)
stynx-code-provider/       AnthropicProvider + OpenAiProvider (DeepSeek etc.)
stynx-code-engine/         streaming tool-use loop, max-turns, compact
stynx-code-tools/          bash, read, write, edit, glob, grep, web_fetch, …
stynx-code-permission/     allow/deny rules + interactive prompts (bridge to TUI)
stynx-code-memory/         per-project session persistence
stynx-code-commands/       slash-command handlers
stynx-code-compact/        conversation summarization
stynx-code-coordinator/    parallel sub-agent orchestration
stynx-code-skills/         user-defined skill loading
stynx-code-plugins/        plugin host (skills + MCP)
stynx-code-bridge/         server bridge for IDE/web frontends
stynx-code-server/         optional HTTP/SSE server surface
stynx-code-services/       cross-cutting services (tips, telemetry, …)
stynx-code-tui/            ratatui-based terminal UI
```

## Intern mode in 30 seconds

```bash
# .env
DEEPSEEK_API_KEY=sk-...

# in stynx-code
> refactor every println! in src/foo.rs to tracing::info!; hand it to the intern
```

Claude calls `delegate_to_intern` with a focused task description. The intern (DeepSeek) runs with a restricted toolset (bash/read/write/edit/glob/grep — no recursive sub-agents) and returns `Summary / Files changed / Output`. Claude reviews and integrates.

You can also call it directly: `/intern list every public function in stynx-code-tui`.

## Hooks

Drop scripts referenced by `settings.json::hooks`:

- `session-start` — run when a session begins; output is shown to the user
- `pre-tool-use` — run before each tool execution; can inject context
- `post-tool-use` — run after each tool execution; can post-process

Each hook receives the relevant JSON on stdin and prints text to stdout.

## License

MIT
