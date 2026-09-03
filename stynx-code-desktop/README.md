# Stynx Desktop (cross-platform)

Tauri v2 + Svelte front-end for the stynx engine, targeting macOS, Windows,
and Linux. The Rust backend (`src-tauri/`) links `stynx-code-app` directly —
no uniffi layer — and exposes the same command surface the macOS SwiftUI app
(`stynx-code-mac/`) consumes over FFI. Engine events stream to the webview as
a single `engine-event` Tauri event, tagged by `type`.

## Layout

- `src-tauri/src/` — Rust backend (workspace member `stynx-code-desktop`),
  layered like the rest of the workspace:
  - `domain/` — UI event enum and serializable models; no Tauri types.
  - `application/` — session state and the engine dispatch loop; takes an
    event-sink closure, so it never touches Tauri directly.
  - `infrastructure/` — the Tauri adapters: event emitter, engine bridge
    drains, and the `#[tauri::command]` surface (session, messaging,
    settings, history).
- `src/` — Svelte UI: `lib/` holds the logic (invoke wrappers, engine-event
  reducer, messaging queue, session actions, markdown parser), `components/`
  holds presentation only; all files kebab-case and under 200 lines.

## Develop

```bash
cd stynx-code-desktop
npm install
npm run tauri dev
```

## Build installers

```bash
npm run tauri build   # .dmg / .msi + .exe / .deb + .AppImage per host OS
```

Regenerate the full icon set from the shared logo before shipping:

```bash
npm run tauri icon ../stynx-code-mac/Resources/AppIcon.icns
```

## Platform notes

- Linux needs WebKitGTK (`libwebkit2gtk-4.1-dev` and friends at build time).
- Windows needs the WebView2 runtime (preinstalled on Windows 11).
- The engine's bash tool assumes a POSIX shell; on Windows run under WSL or
  Git Bash until a PowerShell adapter lands in `stynx-code-tools`.
