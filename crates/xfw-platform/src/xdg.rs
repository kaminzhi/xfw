use std::collections::HashMap;

use wayland_client::protocol::wl_surface;
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel};

use crate::error::PlatformResult;
use crate::wayland::WaylandConnection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Activated,
    Maximized,
    Minimized,
    Fullscreen,
    Resizing,
    Focused,
}

impl WindowState {
    pub fn to_wl(self) -> xdg_toplevel::State {
        match self {
            WindowState::Activated => xdg_toplevel::State::Activated,
            WindowState::Maximized => xdg_toplevel::State::Maximized,
            WindowState::Minimized => xdg_toplevel::State::Maximized,
            WindowState::Fullscreen => xdg_toplevel::State::Fullscreen,
            WindowState::Resizing => xdg_toplevel::State::Resizing,
            WindowState::Focused => xdg_toplevel::State::Activated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowResizeEdge {
    None,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl From<WindowResizeEdge> for xdg_toplevel::ResizeEdge {
    fn from(edge: WindowResizeEdge) -> Self {
        match edge {
            WindowResizeEdge::None => xdg_toplevel::ResizeEdge::None,
            WindowResizeEdge::Top => xdg_toplevel::ResizeEdge::Top,
            WindowResizeEdge::Bottom => xdg_toplevel::ResizeEdge::Bottom,
            WindowResizeEdge::Left => xdg_toplevel::ResizeEdge::Left,
            WindowResizeEdge::Right => xdg_toplevel::ResizeEdge::Right,
            WindowResizeEdge::TopLeft => xdg_toplevel::ResizeEdge::TopLeft,
            WindowResizeEdge::TopRight => xdg_toplevel::ResizeEdge::TopRight,
            WindowResizeEdge::BottomLeft => xdg_toplevel::ResizeEdge::BottomLeft,
            WindowResizeEdge::BottomRight => xdg_toplevel::ResizeEdge::BottomRight,
        }
    }
}

#[derive(Clone)]
pub struct XdgWindowConfig {
    pub title: String,
    pub app_id: Option<String>,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub decorations: bool,
    pub resizable: bool,
    pub fullscreen: bool,
    pub maximized: bool,
    pub minimized: bool,
    pub focus: bool,
}

impl Default for XdgWindowConfig {
    fn default() -> Self {
        Self {
            title: "xfw".to_string(),
            app_id: Some("xfw".to_string()),
            width: 800,
            height: 600,
            min_width: 0,
            min_height: 0,
            max_width: 0,
            max_height: 0,
            decorations: true,
            resizable: true,
            fullscreen: false,
            maximized: false,
            minimized: false,
            focus: true,
        }
    }
}

impl XdgWindowConfig {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            title: title.to_string(),
            app_id: Some("xfw".to_string()),
            width,
            height,
            ..Default::default()
        }
    }

    pub fn with_app_id(mut self, app_id: &str) -> Self {
        self.app_id = Some(app_id.to_string());
        self
    }

    pub fn with_min_size(mut self, width: u32, height: u32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    pub fn with_max_size(mut self, width: u32, height: u32) -> Self {
        self.max_width = width;
        self.max_height = height;
        self
    }

    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }
}

pub struct XdgWindow {
    pub id: u32,
    pub surface: wl_surface::WlSurface,
    pub xdg_surface: xdg_surface::XdgSurface,
    pub toplevel: xdg_toplevel::XdgToplevel,
    #[allow(dead_code)]
    config: XdgWindowConfig,
    committed: bool,
    width: u32,
    height: u32,
    #[allow(dead_code)]
    states: Vec<WindowState>,
}

impl XdgWindow {
    pub fn new(_connection: &WaylandConnection, _config: XdgWindowConfig) -> PlatformResult<Self> {
        todo!("XDG window creation requires deeper WaylandConnection integration");
    }

    pub fn set_size(&mut self, width: u32, height: u32) -> PlatformResult<()> {
        self.width = width;
        self.height = height;
        self.surface.set_buffer_scale(1);
        self.toplevel.set_max_size(width as i32, height as i32);
        Ok(())
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> PlatformResult<()> {
        if fullscreen {
            self.toplevel.set_fullscreen(None);
        } else {
            self.toplevel.unset_fullscreen();
        }
        Ok(())
    }

    pub fn set_maximized(&mut self, maximized: bool) -> PlatformResult<()> {
        if maximized {
            self.toplevel.set_maximized();
        } else {
            self.toplevel.unset_maximized();
        }
        Ok(())
    }

    pub fn commit(&mut self) -> PlatformResult<()> {
        self.surface.commit();
        self.committed = true;
        Ok(())
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

pub struct XdgWindowManager {
    windows: HashMap<u32, XdgWindow>,
    focused_window: Option<u32>,
}

impl XdgWindowManager {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            focused_window: None,
        }
    }

    pub fn create_window(
        &mut self,
        connection: &WaylandConnection,
        config: XdgWindowConfig,
    ) -> PlatformResult<u32> {
        let window = XdgWindow::new(connection, config)?;
        let id = window.get_id();
        self.windows.insert(id, window);
        Ok(id)
    }

    pub fn get_window(&self, id: u32) -> Option<&XdgWindow> {
        self.windows.get(&id)
    }

    pub fn get_window_mut(&mut self, id: u32) -> Option<&mut XdgWindow> {
        self.windows.get_mut(&id)
    }

    pub fn remove_window(&mut self, id: u32) -> Option<XdgWindow> {
        if self.focused_window == Some(id) {
            self.focused_window = None;
        }
        self.windows.remove(&id)
    }

    pub fn set_focused(&mut self, id: u32) {
        self.focused_window = Some(id);
    }

    pub fn get_focused(&self) -> Option<u32> {
        self.focused_window
    }

    pub fn windows(&self) -> impl Iterator<Item = &XdgWindow> {
        self.windows.values()
    }
}

impl Default for XdgWindowManager {
    fn default() -> Self {
        Self::new()
    }
}
