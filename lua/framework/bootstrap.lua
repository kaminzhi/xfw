local ui = require("framework.ui")
local runtime = require("framework.runtime")
local style = require("framework.style")

_G.View = ui.View
_G.Text = ui.Text
_G.Image = ui.Image
_G.Window = ui.Window
_G.Button = ui.Button
_G.Row = ui.Row
_G.Column = ui.Column

_G.Style = style.Style
_G.UI = runtime.UI
_G.IPC = runtime.IPC
_G.System = runtime.System

return true
