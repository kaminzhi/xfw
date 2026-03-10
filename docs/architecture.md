# Architecture Overview

## High-Level Flow
1. **Lua DSL** (`lua/widgets/*.lua`) declares the widget tree using nested tables that resemble HTML; these scripts are the sole configuration surface.
2. **Runtime Bridge** (`runtime::lua`) loads the tree via `mlua`, assigns stable node IDs, and exports observable state handles.
3. **Layout Engine** (`layout::graph`) maps the tree into `taffy` nodes and computes absolute frames.
4. **Renderer** (`render::compositor`) walks dirty nodes, rasterizes them via `tiny-skia` + `cosmic-text`, and uploads buffers to the Wayland surface managed by `platform::wayland`.
5. **Event Loop** blocks on `epoll`/Wayland file descriptors; external IPC injects state changes that invalidates targeted nodes only.

```
Lua DSL ──IPC────┐
                 │  Hot reload / state patches
runtime::lua─────┴──▶layout::graph──▶render::compositor──▶Wayland Layer Surface
          ▲                                            ▲
          └────────────── dirty rect metadata ◀────────┘
```

## Module Responsibilities

| Module | Purpose |
| --- | --- |
| `runtime` | Owns process lifecycle, hot reload controller, scheduler, and dirty-rect tracking. |
| `runtime::lua` | Embeds LuaJIT, exposes DSL helpers, serializes node graphs, and maps observable state to Rust channels. |
| `layout` | Wraps `taffy` to compute flex/grid layouts, caches node styles, and reports frame diffs. |
| `render` | CPU renderer built on `tiny-skia` for primitives and `cosmic-text` for glyph/emoji shaping. |
| `platform` | Wayland + layer-shell setup, buffer management, input dispatch, epoll-driven wakeups. |
| `cli` | Parses CLI arguments/environment and points the runtime at a Lua config entrypoint. |

## Data Contracts
- **Node Tree Schema:** Each node carries `id`, `kind`, `props`, `style` tables; IDs stay stable for diffing.
- **Invalidation Messages:** `runtime::lua` sends `{ node_id, dirty_rect }` events; renderer batches by surface region.
- **Input Events:** Platform module normalizes pointer/button events and forwards them back to Lua with hit-test metadata.

## Performance Principles
- Prefer stack allocation and small structs; no dynamic trait objects on the hot path.
- Track dirty rectangles per node and collapse overlapping regions before rasterization.
- Sleep the event loop via OS blocking primitives; wake on Wayland fd or IPC pipes only.
- Keep Lua↔Rust FFI chatter coarse-grained (diff bundles, not per-pixel instructions).
