use anyhow::Result;
use xfw_layout::{Color, Rect, RenderObject, RenderObjectTree};

#[derive(Debug, Clone)]
pub enum DrawCommand {
    FillRect {
        rect: Rect,
        color: (f32, f32, f32, f32),
    },
    StrokeRect {
        rect: Rect,
        color: (f32, f32, f32, f32),
        width: f32,
    },
    DrawText {
        text: String,
        x: f32,
        y: f32,
        color: (f32, f32, f32, f32),
        font_size: f32,
    },
    DrawImage {
        path: String,
        rect: Rect,
    },
}

pub struct Renderer {
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn with_default_size() -> Self {
        Self::new(1920, 1080)
    }

    pub fn prepare(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn render(
        &mut self,
        tree: &RenderObjectTree,
        _dirty_rect: Option<Rect>,
    ) -> Result<Vec<DrawCommand>> {
        let mut commands = Vec::new();
        self.process_node(tree.root(), &mut commands);
        Ok(commands)
    }

    fn process_node(&self, node: &RenderObject, commands: &mut Vec<DrawCommand>) {
        let rect = node.rect();
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        let render_style = node.render_style();

        match node {
            RenderObject::Container { children, .. } => {
                if let Some(bg) = render_style.background_color {
                    commands.push(DrawCommand::FillRect {
                        rect: *rect,
                        color: (bg.r, bg.g, bg.b, bg.a),
                    });
                }

                if let Some(border_color) = render_style.border_color {
                    let border_width = render_style.border_radius.unwrap_or(1.0);
                    commands.push(DrawCommand::StrokeRect {
                        rect: *rect,
                        color: (
                            border_color.r,
                            border_color.g,
                            border_color.b,
                            border_color.a,
                        ),
                        width: border_width,
                    });
                }

                for child in children {
                    self.process_node(child, commands);
                }
            }
            RenderObject::Text { content, .. } => {
                let color = render_style
                    .color
                    .map(|c| (c.r, c.g, c.b, c.a))
                    .unwrap_or((0.0, 0.0, 0.0, 1.0));
                let font_size = render_style.font_size.unwrap_or(14.0);

                commands.push(DrawCommand::DrawText {
                    text: content.clone(),
                    x: rect.x,
                    y: rect.y,
                    color,
                    font_size,
                });
            }
            RenderObject::Image { path, .. } => {
                commands.push(DrawCommand::DrawImage {
                    path: path.clone(),
                    rect: *rect,
                });
            }
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}
