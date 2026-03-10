# Development Roadmap

## Phase 1 — The Canvas
- [ ] Bootstrap `platform::wayland` to create a layer-shell surface with transparent background.
- [ ] Render a static gradient rectangle via `tiny-skia` to validate CPU raster path.
- [ ] Add CLI flag validation + logging/tracing plumbing.

## Phase 2 — The Bridge & Layout
- [ ] Embed Lua runtime, parse a sample widget tree (`lua/widgets/status_bar.lua`).
- [ ] Convert Lua tables into `layout::graph` nodes and compute frames via `taffy`.
- [ ] Serialize layout output back to Lua for debugging (inspection overlay flag).

## Phase 3 — Reactivity & Input
- [ ] Implement observable stores in Lua and state diff channels in Rust.
- [ ] Add hit-testing + pointer event dispatch.
- [ ] Introduce dirty-rectangle tracking + partial rerendering.

## Phase 4 — Polish
- [ ] Integrate `cosmic-text` for multilingual + emoji typography.
- [ ] Wire IPC adapters (DBus scripts, shell commands, user pipes).
- [ ] Provide tween helpers + easing curves for lightweight animations.

## Stretch Goals
- Benchmark suite comparing RAM/CPU usage with Waybar + AGS.
- Packaging for major distros and `nix` flake recipe.
- Config schema validator + language-server-style autocomplete for the Lua DSL.
