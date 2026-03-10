# Lua Configuration Overview

xfw treats Lua files as the single configuration mechanism for layout, styling, and widget logic. Nothing meaningful happens without a Lua entrypoint.

## Entry Script
- Chosen via `--config path/to/file.lua` (handled by `xfw-cli`).
- Must `return` a root view table built with helpers from `lua/framework/ui.lua`.
- Can require as many Lua modules as desired for composability (`lua/widgets/`, `lua/ipc/`, etc.).
- Advanced setups can define their own style compilers (see `docs/examples/hardware_widgets.lua`) that translate ergonomic Lua syntax into the normalized tables Rust expects.

## Responsibilities of Lua Config
1. **Layout Definition:** Build a tree of `view`, `text`, `image`, etc., providing style tables that mirror CSS/Flexbox semantics.
2. **State Wiring:** Instantiate observables via `lua/framework/state.lua`, subscribe to IPC/timers, and bind results into nodes.
3. **Event Logic:** Assign callbacks for pointer events, custom commands, and background tasks.

## Styling Roadmap
- Today, style data is described inline within Lua tables for maximal performance and clarity.
- Future extensions (SCSS/CSS or theme description languages) will compile into Lua tables before hitting the runtime, allowing authors to pick high-level syntax without changing the engine.

## Live Reload
- Runtime watches Lua files; when they change, the previous state snapshot is provided to the reloaded chunk so widgets can maintain continuity.
- Only the nodes affected by the change are re-laid-out and redrawn thanks to the dirty-rect pipeline.
