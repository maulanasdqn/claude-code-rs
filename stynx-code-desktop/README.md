# Stynx Desktop (cross-platform)

Tauri v2 + Svelte front-end for the stynx engine, targeting macOS, Windows,
and Linux. The Rust backend (`src-tauri/`) links `stynx-code-app` directly —
no uniffi layer — and exposes the same command surface the macOS SwiftUI app
(`stynx-code-mac/`) consumes over FFI. Engine events stream to the webview as
a single `engine-event` Tauri event, tagged by `type`.

## Layout

- `src-tauri/` — Rust backend: session bootstrap (`init_session`), messaging,
  sessions, settings, and the permission / ask-user / workspace bridges as
  Tauri commands (workspace member `stynx-code-desktop`).
- `src/` — Svelte UI: `lib/` (invoke wrappers, engine-event reducer, stores),
  `components/` (sidebar, chat, tool cards, permission and question cards).

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
