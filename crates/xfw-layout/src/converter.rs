use taffy::prelude::*;
use xfw_model::{ContentSource, NodeKind, StyleSource, StyleValue, UiNode};

use super::render_object_tree::RenderObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleAttr {
    FlexDirection,
    AlignItems,
    JustifyContent,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Flex,
    Unknown,
}

impl StyleAttr {
    pub fn from_str(s: &str) -> Self {
        match s {
            "flex_direction" => StyleAttr::FlexDirection,
            "align_items" => StyleAttr::AlignItems,
            "justify_content" => StyleAttr::JustifyContent,
            "width" => StyleAttr::Width,
            "height" => StyleAttr::Height,
            "min_width" => StyleAttr::MinWidth,
            "min_height" => StyleAttr::MinHeight,
            "max_width" => StyleAttr::MaxWidth,
            "max_height" => StyleAttr::MaxHeight,
            "flex" => StyleAttr::Flex,
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
        let style = self.convert_style(&ui_node.props.style);
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
            | NodeKind::Button => RenderObject::container(id, style, children),
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
                RenderObject::text(id, style, content)
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
                RenderObject::image(id, style, path)
            }
            NodeKind::Custom(_) => RenderObject::container(id, style, children),
        }
    }

    fn convert_style(&self, style_source: &StyleSource) -> Style {
        let mut style = Style::default();

        if let StyleSource::Static(entries) = style_source {
            for attr in entries {
                match StyleAttr::from_str(&attr.name) {
                    StyleAttr::FlexDirection => {
                        style.flex_direction = Self::parse_flex_direction(&attr.value);
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
                    StyleAttr::Flex => {
                        style.flex_grow = Self::parse_single_number(&attr.value).unwrap_or(0.0);
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
