local Style = {
    FlexDirection = "flex_direction",
    AlignItems = "align_items",
    Padding = "padding",
    Margin = "margin",
    Gap = "gap",
    BgColor = "bg_color",
    Color = "color",
    FontSize = "font_size",
    BorderRadius = "border_radius",
}

local function normalize(props)
    local normalized = {}
    for key, value in pairs(props or {}) do
        normalized[key] = value
    end
    return normalized
end

return {
    Style = Style,
    normalize = normalize,
}
