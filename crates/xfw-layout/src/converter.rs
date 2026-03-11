use taffy::prelude::*;
use xfw_model::{ContentSource, NodeKind, StyleSource, StyleValue, UiNode};

use super::render_object_tree::{Color, RenderObject, RenderStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleAttr {
    FlexDirection,
    FlexWrap,
    FlexGrow,
    AlignItems,
    JustifyContent,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Gap,
    Unknown,
}

impl StyleAttr {
    pub fn from_str(s: &str) -> Self {
        match s {
            "flex_direction" => StyleAttr::FlexDirection,
            "flex_wrap" => StyleAttr::FlexWrap,
            "flex_grow" => StyleAttr::FlexGrow,
            "align_items" => StyleAttr::AlignItems,
            "justify_content" => StyleAttr::JustifyContent,
            "width" => StyleAttr::Width,
            "height" => StyleAttr::Height,
            "min_width" => StyleAttr::MinWidth,
            "min_height" => StyleAttr::MinHeight,
            "max_width" => StyleAttr::MaxWidth,
            "max_height" => StyleAttr::MaxHeight,
            "gap" => StyleAttr::Gap,
            "flex" => StyleAttr::FlexGrow,
            _ => StyleAttr::Unknown,
        }
    }
}

pub struct RenderObjectConverter;

impl RenderObjectConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn convert(&self, ui_node: &UiNode) -> RenderObject {
        let layout_style = self.convert_layout_style(&ui_node.props.style);
        let render_style = self.convert_render_style(&ui_node.props.style);
        let id = ui_node.id.clone();
        let children: Vec<RenderObject> = ui_node
            .children
            .iter()
            .map(|child| self.convert(child))
            .collect();

        match &ui_node.kind {
            NodeKind::View
            | NodeKind::Window
            | NodeKind::Row
            | NodeKind::Column
            | NodeKind::Button => RenderObject::container(id, layout_style, render_style, children),
            NodeKind::Text => {
                let content = ui_node
                    .props
                    .value
                    .as_ref()
                    .and_then(|c| match c {
                        ContentSource::StaticString(s) => Some(s.as_str()),
                        ContentSource::Dynamic => None,
                    })
                    .map(String::from)
                    .unwrap_or_default();
                RenderObject::text(id, layout_style, render_style, content)
            }
            NodeKind::Image => {
                let path = ui_node
                    .props
                    .value
                    .as_ref()
                    .and_then(|c| match c {
                        ContentSource::StaticString(s) => Some(s.as_str()),
                        ContentSource::Dynamic => None,
                    })
                    .map(String::from)
                    .unwrap_or_default();
                RenderObject::image(id, layout_style, render_style, path)
            }
            NodeKind::Custom(_) => {
                RenderObject::container(id, layout_style, render_style, children)
            }
        }
    }

    fn convert_render_style(&self, style_source: &StyleSource) -> RenderStyle {
        let mut style = RenderStyle::default();
        if let StyleSource::Static(entries) = style_source {
            for attr in entries {
                match attr.name.as_str() {
                    "color" => {
                        style.color =
                            Self::parse_string(&attr.value).and_then(|s| Color::from_hex(&s));
                    }
                    "font_size" | "font-size" => {
                        style.font_size = Self::parse_single_number(&attr.value);
                    }
                    "bg_color" | "background" | "background_color" => {
                        style.background_color =
                            Self::parse_string(&attr.value).and_then(|s| Color::from_hex(&s));
                    }
                    "border_color" | "border-color" => {
                        style.border_color =
                            Self::parse_string(&attr.value).and_then(|s| Color::from_hex(&s));
                    }
                    "border_radius" | "border-radius" => {
                        style.border_radius = Self::parse_single_number(&attr.value);
                    }
                    "opacity" => {
                        style.opacity = Self::parse_single_number(&attr.value);
                    }
                    _ => {}
                }
            }
        }
        style
    }

    fn parse_string(value: &StyleValue) -> Option<String> {
        match value {
            StyleValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn convert_layout_style(&self, style_source: &StyleSource) -> Style {
        let mut style = Style::default();

        if let StyleSource::Static(entries) = style_source {
            for attr in entries {
                match StyleAttr::from_str(&attr.name) {
                    StyleAttr::FlexDirection => {
                        style.flex_direction = Self::parse_flex_direction(&attr.value);
                    }
                    StyleAttr::FlexWrap => {
                        style.flex_wrap = Self::parse_flex_wrap(&attr.value);
                    }
                    StyleAttr::FlexGrow => {
                        style.flex_grow = Self::parse_single_number(&attr.value).unwrap_or(0.0);
                    }
                    StyleAttr::AlignItems => {
                        style.align_items = Self::parse_align_items(&attr.value);
                    }
                    StyleAttr::JustifyContent => {
                        style.justify_content = Self::parse_justify_content(&attr.value);
                    }
                    StyleAttr::Width => {
                        style.size.width = Self::parse_dimension(&attr.value);
                    }
                    StyleAttr::Height => {
                        style.size.height = Self::parse_dimension(&attr.value);
                    }
                    StyleAttr::MinWidth => {
                        style.min_size.width = Self::parse_dimension(&attr.value);
                    }
                    StyleAttr::MinHeight => {
                        style.min_size.height = Self::parse_dimension(&attr.value);
                    }
                    StyleAttr::MaxWidth => {
                        style.max_size.width = Self::parse_dimension(&attr.value);
                    }
                    StyleAttr::MaxHeight => {
                        style.max_size.height = Self::parse_dimension(&attr.value);
                    }
                    StyleAttr::Gap => {
                        let n = Self::parse_single_number(&attr.value).unwrap_or(0.0);
                        style.gap = Size {
                            width: LengthPercentage::length(n),
                            height: LengthPercentage::length(n),
                        };
                    }
                    StyleAttr::Unknown => {}
                }
            }
        }

        style
    }

    fn parse_flex_direction(value: &StyleValue) -> FlexDirection {
        match value {
            StyleValue::String(s) => match s.as_str() {
                "row" => FlexDirection::Row,
                "row-reverse" => FlexDirection::RowReverse,
                "column" => FlexDirection::Column,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => FlexDirection::Row,
            },
            _ => FlexDirection::Row,
        }
    }

    fn parse_align_items(value: &StyleValue) -> Option<AlignItems> {
        match value {
            StyleValue::String(s) => match s.as_str() {
                "start" | "flex-start" => Some(AlignItems::FlexStart),
                "center" => Some(AlignItems::Center),
                "end" | "flex-end" => Some(AlignItems::FlexEnd),
                "stretch" => Some(AlignItems::Stretch),
                "baseline" => Some(AlignItems::Baseline),
                _ => Some(AlignItems::FlexStart),
            },
            _ => None,
        }
    }

    fn parse_justify_content(value: &StyleValue) -> Option<JustifyContent> {
        match value {
            StyleValue::String(s) => match s.as_str() {
                "start" | "flex-start" => Some(JustifyContent::FlexStart),
                "center" => Some(JustifyContent::Center),
                "end" | "flex-end" => Some(JustifyContent::FlexEnd),
                "space-between" => Some(JustifyContent::SpaceBetween),
                "space-around" => Some(JustifyContent::SpaceAround),
                "space-evenly" => Some(JustifyContent::SpaceEvenly),
                _ => Some(JustifyContent::FlexStart),
            },
            _ => None,
        }
    }

    fn parse_flex_wrap(value: &StyleValue) -> FlexWrap {
        match value {
            StyleValue::String(s) => match s.as_str() {
                "wrap" => FlexWrap::Wrap,
                "nowrap" => FlexWrap::NoWrap,
                "wrap-reverse" => FlexWrap::WrapReverse,
                _ => FlexWrap::NoWrap,
            },
            _ => FlexWrap::NoWrap,
        }
    }

    fn parse_align_content(value: &StyleValue) -> Option<AlignContent> {
        match value {
            StyleValue::String(s) => match s.as_str() {
                "start" | "flex-start" => Some(AlignContent::FlexStart),
                "center" => Some(AlignContent::Center),
                "end" | "flex-end" => Some(AlignContent::FlexEnd),
                "space-between" => Some(AlignContent::SpaceBetween),
                "space-around" => Some(AlignContent::SpaceAround),
                "stretch" => Some(AlignContent::Stretch),
                _ => Some(AlignContent::FlexStart),
            },
            _ => None,
        }
    }

    fn parse_rect(value: &StyleValue) -> Rect<LengthPercentage> {
        if let Some(n) = Self::parse_single_number(value) {
            Rect {
                left: LengthPercentage::length(n),
                right: LengthPercentage::length(n),
                top: LengthPercentage::length(n),
                bottom: LengthPercentage::length(n),
            }
        } else {
            Rect::zero()
        }
    }

    fn parse_rect_auto(value: &StyleValue) -> Rect<LengthPercentageAuto> {
        if let Some(n) = Self::parse_single_number(value) {
            Rect {
                left: LengthPercentageAuto::length(n),
                right: LengthPercentageAuto::length(n),
                top: LengthPercentageAuto::length(n),
                bottom: LengthPercentageAuto::length(n),
            }
        } else {
            Rect::zero()
        }
    }

    fn parse_size(value: &StyleValue) -> Size<LengthPercentage> {
        if let Some(n) = Self::parse_single_number(value) {
            Size {
                width: LengthPercentage::length(n),
                height: LengthPercentage::length(n),
            }
        } else {
            Size::zero()
        }
    }

    fn parse_display(value: &StyleValue) -> Display {
        match value {
            StyleValue::String(s) => match s.as_str() {
                "none" => Display::None,
                "flex" => Display::Flex,
                "grid" => Display::Grid,
                "block" => Display::Block,
                _ => Display::Flex,
            },
            _ => Display::Flex,
        }
    }

    fn parse_position(value: &StyleValue) -> Position {
        match value {
            StyleValue::String(s) => match s.as_str() {
                "absolute" => Position::Absolute,
                "relative" => Position::Relative,
                _ => Position::Relative,
            },
            _ => Position::Relative,
        }
    }

    fn parse_dimension(value: &StyleValue) -> Dimension {
        match Self::parse_single_number(value) {
            Some(n) => Dimension::length(n),
            None => Dimension::auto(),
        }
    }

    fn parse_single_number(value: &StyleValue) -> Option<f32> {
        match value {
            StyleValue::Number(n) => Some(*n),
            StyleValue::Integer(i) => Some(*i as f32),
            StyleValue::String(s) => Self::parse_number_string(s),
            StyleValue::Array(items) if !items.is_empty() => Self::parse_single_number(&items[0]),
            _ => None,
        }
    }

    fn parse_number_string(s: &str) -> Option<f32> {
        let token = s.replace("px", "").trim().to_string();
        token.parse::<f32>().ok()
    }
}

impl Default for RenderObjectConverter {
    fn default() -> Self {
        Self::new()
    }
}
