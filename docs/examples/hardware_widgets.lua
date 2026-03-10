-- ============================================================================
-- TYPE DEFINITIONS & API STUBS (For Editor Autocomplete / LuaLS)
-- Project: Ultra-Lightweight Wayland Widget Framework
-- ============================================================================

---@alias FlexDirection "row" | "column"
---@alias AlignItems "flex-start" | "center" | "flex-end" | "stretch"
---@alias JustifyContent "flex-start" | "center" | "flex-end" | "space-between" | "space-around"
---@alias DynamicString string | fun(): string
---@alias DynamicStyle table<integer, any> | fun(): table<integer, any>

---@class StyleProps
---@field flex_direction? FlexDirection
---@field align_items? AlignItems
---@field justify_content? JustifyContent
---@field padding? string | number
---@field margin? string | number
---@field gap? number
---@field bg_color? string           -- e.g., "#1e1e2e" or "rgba(30, 30, 46, 0.9)"
---@field color? string              -- Text color
---@field font_size? number
---@field border_radius? number
---@field flex_grow? number          -- Used for Spacer elements
---@field transition? string         -- e.g., "all 0.2s ease" (Handled by Rust tweening)

--- Utility function to wrap style properties.
---@param props StyleProps
---@return table<integer, any>
function S(props) end

-- ============================================================================
-- SEMANTIC UI COMPONENTS (Instantiated and calculated by Rust/Taffy)
-- ============================================================================

---@class ComponentProps
---@field id? string                 -- Unique identifier for targeted partial repaints
---@field style? DynamicStyle        -- Table or a reactive function returning a Table
---@field children? table[]          -- Nested UI nodes
---@field on_click? fun()            -- Callback for mouse click events
---@field on_scroll? fun(direction: "up" | "down") -- Callback for mouse wheel events
---@field on_hover? fun(is_hovered: boolean)       -- Callback for hover state changes
---@field value? DynamicString       -- Static string or reactive function returning a string (for Text)

--- A generic container node.
---@param props ComponentProps
---@return table
function View(props) end

--- A container that aligns children horizontally (flex_direction = "row").
---@param props ComponentProps
---@return table
function Row(props) end

--- A container that aligns children vertically (flex_direction = "column").
---@param props ComponentProps
---@return table
function Column(props) end

--- An interactive container that provides click/hover/scroll feedback.
---@param props ComponentProps
---@return table
function Button(props) end

--- A flexible space that expands to fill available room (flex_grow = 1).
---@param props ComponentProps|nil
---@return table
function Spacer(props) end

--- A node for rendering text strings and emojis via cosmic-text.
---@param props ComponentProps
---@return table
function Text(props) end

-- ============================================================================
-- WINDOW COMPONENT (Wayland Layer Shell Root)
-- ============================================================================

---@alias Anchor "top" | "bottom" | "left" | "right" | "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center"
---@alias Layer "background" | "bottom" | "top" | "overlay"

---@class WindowProps
---@field anchor? Anchor             -- Position on the screen
---@field layer? Layer               -- Wayland Layer Shell z-index equivalent
---@field exclusive_zone? integer | "auto" -- Reserves screen space to prevent window overlap
---@field monitor? string | integer  -- Target monitor (e.g., "DP-1" or 1)
---@field margin? string | number    -- Distance from the screen edge
---@field style? DynamicStyle        -- Root node styling
---@field children? table[]          -- The internal DOM tree

--- The Root Window container mapping directly to a Layer Shell surface.
---@param props WindowProps
---@return table
function Window(props) end

-- ============================================================================
-- UI API (State Management & Renderer Bridge)
-- ============================================================================

---@class UIAPI
UI = {}

--- Creates a Deep Reactive Proxy from a standard Lua table.
--- Intercepts reads/writes to automatically track dependencies and trigger
--- dirty-rectangle repaints in Rust when accessed data changes.
---@generic T : table
---@param initial_state T
---@return T
function UI.state(initial_state) end

--- Mounts the Virtual DOM root node to the Wayland Layer Shell window.
--- Triggers Rust to calculate the initial Taffy layout and perform the first render.
---@param root_node table
function UI.render(root_node) end

-- ============================================================================
-- IPC API (Inter-Process Communication / The Nervous System)
-- ============================================================================

---@class IPCAPI
IPC = {}

--- Listens to a UNIX Domain Socket (e.g., Hyprland/Sway IPC, mpd).
--- Awakens Lua from epoll only when data is received.
---@param socket_path string
---@param callback fun(payload: table|string)
function IPC.listen_socket(socket_path, callback) end

--- Watches a file descriptor for changes (e.g., sysfs battery files).
---@param file_path string
---@param event "on_change" | "on_read"
---@param callback fun(value: string)
function IPC.watch_file(file_path, event, callback) end

--- Spawns an external command that yields continuous output (e.g., `pactl subscribe`)
--- and attaches its stdout to the Rust epoll loop.
---@param command string
---@param callback fun(line: string)
function IPC.spawn_and_watch(command, callback) end

-- ============================================================================
-- EXAMPLE IMPLEMENTATION
-- ============================================================================

-- 1. GLOBAL STATE (Single Source of Truth)
local Store = UI.state({
	workspaces = { active = 1 },
	speaker = { vol = 50, is_muted = false },
	battery = { level = 100, is_charging = false },
})

-- 2. STATELESS COMPONENTS
local function WorkspacesWidget()
	return Row({
		style = S({ gap = 8, align_items = "center" }),
		children = {
			Text({ value = "WS:", style = S({ color = "#a6adc8", font_size = 14 }) }),
			Text({
				-- Reactive value: Re-evaluated only when Store.workspaces.active changes
				value = function()
					return tostring(Store.workspaces.active)
				end,
				style = S({ color = "#89b4fa", font_size = 16 }),
			}),
		},
	})
end

local function SpeakerWidget()
	return Button({
		id = "widget_speaker",
		style = S({ align_items = "center", gap = 6, padding = "4px 8px", border_radius = 8 }),

		on_scroll = function(direction)
			if direction == "up" then
				Store.speaker.vol = math.min(100, Store.speaker.vol + 5)
			else
				Store.speaker.vol = math.max(0, Store.speaker.vol - 5)
			end
		end,

		on_click = function()
			Store.speaker.is_muted = not Store.speaker.is_muted
		end,

		children = {
			Text({
				value = function()
					if Store.speaker.is_muted then
						return "󰖁"
					end
					if Store.speaker.vol > 50 then
						return "󰕾"
					end
					return "󰕿"
				end,
				-- Reactive style based on state
				style = function()
					return S({ color = Store.speaker.is_muted and "#f38ba8" or "#89b4fa" })
				end,
			}),
			Text({
				value = function()
					return Store.speaker.is_muted and "Muted" or (Store.speaker.vol .. "%")
				end,
				style = S({ color = "#cdd6f4", font_size = 14 }),
			}),
		},
	})
end

local function BatteryWidget()
	return Row({
		id = "widget_battery",
		style = S({ align_items = "center", gap = 6, padding = "4px 8px", border_radius = 8 }),

		children = {
			Text({
				value = function()
					return Store.battery.is_charging and "󰂄" or "󰁹"
				end,
				style = function()
					local is_critical = (not Store.battery.is_charging) and (Store.battery.level <= 20)
					return S({ color = is_critical and "#f38ba8" or "#a6e3a1" })
				end,
			}),
			Text({
				value = function()
					return Store.battery.level .. "%"
				end,
				style = S({ color = "#cdd6f4", font_size = 14 }),
			}),
		},
	})
end

-- 3. ROOT RENDERER
UI.render(Window({
	anchor = "top",
	style = S({
		margin = "10px",
		bg_color = "rgba(30, 30, 46, 0.9)",
		border_radius = 12,
		padding = "5px 15px",
		gap = 10,
		align_items = "center",
	}),
	children = {
		WorkspacesWidget(),

		-- The Spacer pushes widgets to opposite ends
		Spacer(),

		Row({
			style = S({ gap = 15, align_items = "center" }),
			children = {
				SpeakerWidget(),
				BatteryWidget(),
			},
		}),
	},
}))

-- 4. IPC EVENT LISTENERS (Hardware / System integration)
IPC.listen_socket("/tmp/hypr/my_socket.sock", function(payload)
	-- Assuming a JSON decoded payload table
	if payload.event == "workspace" then
		Store.workspaces.active = payload.data
	end
end)

IPC.watch_file("/sys/class/power_supply/BAT0/capacity", "on_change", function(value)
	Store.battery.level = tonumber(value) or 100
end)

IPC.watch_file("/sys/class/power_supply/BAT0/status", "on_change", function(value)
	Store.battery.is_charging = (value:match("Charging") ~= nil)
end)
