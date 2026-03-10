local STYLE_MAP = {
    flex_direction = Style.FlexDirection,
    align_items = Style.AlignItems,
    padding = Style.Padding,
    gap = Style.Gap,
    bg_color = Style.BgColor,
    color = Style.Color,
    font_size = Style.FontSize,
    border_radius = Style.BorderRadius,
}

local function S(props)
    local normalized = {}
    for key, token in pairs(STYLE_MAP) do
        if props[key] ~= nil then
            normalized[token] = props[key]
        end
    end
    return normalized
end

local function ClockWidget()
    return View {
        id = "clock_widget",
        style = S {
            flex_direction = "row",
            gap = 6,
            align_items = "center",
        },
        children = {
            Text {
                value = function()
                    return os.date("%H:%M")
                end,
                style = S { color = "#cdd6f4", font_size = 14 },
            },
        },
    }
end

UI.render(Window {
    id = "status_bar_root",
    anchor = "top",
    style = S {
        flex_direction = "row",
        gap = 12,
        padding = "8px 16px",
        bg_color = "rgba(16, 19, 27, 0.8)",
        border_radius = 10,
    },
    children = {
        ClockWidget(),
        Spacer { flex = 1 },
        Text {
            value = function()
                return "XFw"
            end,
            style = S { color = "#89b4fa", font_size = 14 },
        },
    },
})
