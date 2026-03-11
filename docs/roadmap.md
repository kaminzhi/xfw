# Development Roadmap

> Last updated: 2026-03-12
> Branch: `wtf`
> Tests: 20 passing

## Phase 1 — The Canvas
- [x] Bootstrap `platform::wayland` to create a layer-shell surface with transparent background.
  - Implemented Wayland connection via wayland-client
  - Event loop with mio/epoll for zero CPU idle
  - FD monitoring infrastructure
  - Headless mode support when no Wayland available
- [ ] Render a static gradient rectangle via `tiny-skia` to validate CPU raster path.
- [x] Add CLI flag validation + logging/tracing plumbing.
  - xfw-cli crate with clap argument parsing
  - tracing for structured logging

## Phase 2 — The Bridge & Layout
- [x] Embed Lua runtime, parse a sample widget tree (`lua/widgets/status_bar.lua`).
  - xfw-runtime crate with mlua integration
  - Lua state management with reactivity
- [x] Convert Lua tables into `layout::graph` nodes and compute frames via `taffy`.
  - xfw-layout crate wrapping taffy
  - RenderObjectTree with ID indexing
- [ ] Serialize layout output back to Lua for debugging (inspection overlay flag).

## Phase 3 — Reactivity & Input
- [x] Implement observable stores in Lua and state diff channels in Rust.
  - StateRegistry with path-based tracking
  - Dirty flagging on state changes
- [ ] Add hit-testing + pointer event dispatch.
- [ ] Introduce dirty-rectangle tracking + partial rerendering.
- [x] Wire IPC adapters (inotify file watching).
  - FileWatcher for monitoring system files
  - Support for watch_file and watch_directory

## Phase 4 — Polish
- [ ] Integrate `cosmic-text` for multilingual + emoji typography.
- [ ] Wire IPC adapters (DBus scripts, shell commands, user pipes).
- [ ] Provide tween helpers + easing curves for lightweight animations.

## Stretch Goals
- Benchmark suite comparing RAM/CPU usage with Waybar + AGS.
- Packaging for major distros and `nix` flake recipe.
- Config schema validator + language-server-style autocomplete for the Lua DSL.
