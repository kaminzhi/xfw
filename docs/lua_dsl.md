# Lua DSL Cheatsheet

```lua
local ui = require("framework.ui")

return ui.view({
  id = "status-bar",
  direction = "row",
  spacing = 8,
  padding = { 12, 16 },
  style = {
    background = "#0f1117dd",
    radius = 12,
  },
  children = {
    ui.text({ id = "clock", bind = "time.now" }),
    ui.spacer(),
    ui.icon({ source = "assets/battery.png" }),
    ui.text({ id = "battery", bind = "system.battery" }),
  },
})
```

## Core Concepts
- **Elements:** `view`, `text`, `icon`, `image`, `spacer`, `stack`, etc. Each returns a Lua table with metadata.
- **Styles:** Mirror CSS-like properties (`direction`, `justify`, `align`, `gap`, `padding`, `margin`, `border`, `background`). Values map one-to-one with `taffy` nodes.
- **Theming Futures:** Additional authoring layers (SCSS/CSS translators, theme compilers) can later emit the same Lua tables, but native Lua definitions are the source of truth for now.
- **Bindings:** `bind = "store.path"` wires node props to observable stores. Runtime maps them to node IDs for precise invalidation.
- **Actions:** `on_click`, `on_scroll`, `on_hover` callbacks reference Lua functions; runtime carries pointer metadata.
- **Animations:** Declarative `tween` table with target props + easing curve; runtime schedules them through the dirty-rect system.

## File Layout
- `lua/framework/*.lua` — Helpers, DSL constructors, state primitives.
- `lua/widgets/*.lua` — User-authored bars/widgets (status bars, side panels, notification lists).
- `lua/ipc/*.lua` — Optional modules bridging to shell commands or DBus.

## Hot Reload Flow
1. Runtime watches Lua files for changes (inotify).
2. On change, current state snapshot is serialized and passed into the reloaded chunk.
3. New widget tree diff is computed; stable IDs prevent UI flicker.
