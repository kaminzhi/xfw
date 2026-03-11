# xfw — Xiaoxigua Flash Widget

Xiaoxigua Flash Widget (xfw) is an ultra-lightweight Wayland bar/widget runtime that pairs a high-performance Rust core with hot-reloadable Lua declarative views. The framework targets Linux ricers that want AGS-level ergonomics with the RAM footprint of a small C binary.

## Highlights
- Declarative Lua DSL for describing views, layout metadata, and event bindings.
- Rust runtime with `taffy` flex/grid layout, `tiny-skia` + `cosmic-text` CPU rendering, and smithay layer-shell integration.
- Single RenderObject tree (Flutter-style) with both layout style and render style.
- Observable state graph that invalidates only dirty rectangles; zero redraws when nothing changes.
- Hot-reload Lua modules without restarting the compositor session.

## Architecture

### RenderObject Tree (Single Tree)
```
RenderObjectTree
├── RenderObject::Container { layout_style, render_style, rect, children }
├── RenderObject::Text { layout_style, render_style, rect, content }
└── RenderObject::Image { layout_style, render_style, rect, path }
```

### Style Separation
- **layout_style**: taffy::Style (flex_direction, justify_content, gap, padding, margin, etc.)
- **render_style**: RenderStyle (color, font_size, background_color, border_color, border_radius, opacity)

### Render Pipeline
```
Lua config → RenderObjectConverter → RenderObjectTree (with layout_style + render_style)
    → LayoutEngine.compute_layout() → rect computed
    → Renderer.render() → DrawCommand list → tiny-skia
```

## Repository Layout
```
docs/ # Architecture notes, roadmap, interface specs
lua/ # Lua DSL, widgets, IPC helpers
crates/
  xfw/ # Binary entrypoint, CLI + logging bootstrap
  xfw-cli/ # CLI parsing + Lua config entrypoint selection
  xfw-model/ # Core data structures (UiNode, StyleSource, StateField)
  xfw-layout/ # taffy-powered layout graph, RenderObjectTree, converter
  xfw-platform/ # Wayland + event loop glue (future smithay integration)
  xfw-render/ # tiny-skia + cosmic-text renderer, DrawCommand
  xfw-runtime/ # Scheduler, Lua bridge, dirty rect orchestration
Cargo.toml # Workspace manifest
README.md
```

## Quick Start
```bash
# install dependencies (Wayland dev libs, LuaJIT) then build
cargo build --workspace

# run with a sample widget set (Lua defines layout + styles + events)
cargo run -p xfw -- --config lua/widgets/status_bar.lua
```

## Testing
```bash
cargo test --workspace  # 27 tests across all crates
```

## Configuration Model
- **Lua-first:** Every widget layout, style, and event binding is authored in Lua (`lua/widgets/*.lua`). The runtime loads the Lua tree through `mlua`, diffs changes, and reacts without restarting.
- **State + Logic:** Lua modules describe observable stores, IPC handlers, and view trees in one place. Rust stays focused on layout math, rendering, and Wayland glue.
- **Future styling options:** Additional style descriptions (SCSS/CSS translators, theming DSLs) can compile down to the same Lua schema later, but raw Lua definitions remain the priority for now.

## Next Steps
1. Implement the Phase 1 "Canvas" milestone (Wayland surface + pixel buffer test pattern).
2. Finalize the Lua DSL schema (`docs/lua_dsl.md`) and bridge it inside `runtime::lua`.
3. Flesh out the dirty-rectangle renderer and benchmarking harness.

See `docs/configuration.md` for Lua config expectations, `docs/examples/hardware_widgets.lua` for an annotated speaker/battery widget script, and `docs/roadmap.md` for the detailed multi-phase plan.
