use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use wayland_client::{
    protocol::{wl_surface, WlSurface},
    Proxy,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use crate::connection::WaylandConnection;
use crate::error::{PlatformError, Result};

static XDG_WINDOW_ID: AtomicU32 = AtomicU32::new(0);

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
    pub fn to_wl(self) -> u32 {
        match self {
            WindowState::Activated => xdg_toplevel::State::Activated.bits(),
            WindowState::Maximized => xdg_toplevel::State::Maximized.bits(),
            WindowState::Minimized => xdg_toplevel::State::Minimized.bits(),
            WindowState::Fullscreen => xdg_toplevel::State::Fullscreen.bits(),
            WindowState::Resizing => xdg_toplevel::State::Resizing.bits(),
            WindowState::Focused => xdg_toplevel::State::Activated.bits(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAnchor {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

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

    pub fn with_decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
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
    pub surface: Proxy<wl_surface::WlSurface>,
    pub xdg_surface: Proxy<xdg_surface::XdgSurface>,
    pub toplevel: Proxy<xdg_toplevel::XdgToplevel>,
    config: XdgWindowConfig,
    committed: bool,
    width: u32,
    height: u32,
    states: Vec<WindowState>,
}

impl XdgWindow {
    pub fn new(connection: &WaylandConnection, config: XdgWindowConfig) -> Result<Self> {
        let state = connection.state();
        let wm_base = state
            .lock()
            .xdg_wm_base
            .clone()
            .ok_or_else(|| PlatformError::ProtocolNotSupported("xdg_wm_base".to_string()))?;

        let surface = connection.get_surface()?;
        let mut qh = connection.queue();

        let xdg_surface = wm_base
            .get_xdg_surface(&surface, &mut qh)
            .map_err(|e| PlatformError::Surface(format!("Failed to get xdg surface: {}", e)))?;

        let toplevel = xdg_surface
            .get_toplevel(&mut qh)
            .map_err(|e| PlatformError::Surface(format!("Failed to get toplevel: {}", e)))?;

        toplevel
            .set_title(config.title.clone())
            .map_err(|e| PlatformError::Surface(format!("Failed to set title: {}", e)))?;

        if let Some(app_id) = &config.app_id {
            toplevel
                .set_app_id(app_id.clone())
                .map_err(|e| PlatformError::Surface(format!("Failed to set app_id: {}", e)))?;
        }

        if config.min_width > 0 || config.min_height > 0 {
            toplevel
                .set_min_size(config.min_width as i32, config.min_height as i32)
                .map_err(|e| PlatformError::Surface(format!("Failed to set min size: {}", e)))?;
        }

        if config.max_width > 0 || config.max_height > 0 {
            toplevel
                .set_max_size(config.max_width as i32, config.max_height as i32)
                .map_err(|e| PlatformError::Surface(format!("Failed to set max size: {}", e)))?;
        }

        toplevel
            .set_resizable(config.resizable)
            .map_err(|e| PlatformError::Surface(format!("Failed to set resizable: {}", e)))?;

        if config.decorations {
            toplevel
                .set_decorations(xdg_toplevel::Decorations::Request)
                .map_err(|e| PlatformError::Surface(format!("Failed to set decorations: {}", e)))?;
        } else {
            toplevel
                .set_decorations(xdg_toplevel::Decorations::None)
                .map_err(|e| PlatformError::Surface(format!("Failed to set decorations: {}", e)))?;
        }

        let id = XDG_WINDOW_ID.fetch_add(1, Ordering::SeqCst);

        Ok(Self {
            id,
            surface,
            xdg_surface,
            toplevel,
            config,
            committed: false,
            width: config.width,
            height: config.height,
            states: Vec::new(),
        })
    }

    pub fn set_size(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width;
        self.height = height;
        self.surface.set_buffer_scale(1);
        self.toplevel
            .set_size(width as i32, height as i32)
            .map_err(|e| PlatformError::Surface(format!("Failed to set size: {}", e)))
    }

    pub fn set_title(&mut self, title: &str) -> Result<()> {
        self.toplevel
            .set_title(title.to_string())
            .map_err(|e| PlatformError::Surface(format!("Failed to set title: {}", e)))
    }

    pub fn set_app_id(&mut self, app_id: &str) -> Result<()> {
        self.toplevel
            .set_app_id(app_id.to_string())
            .map_err(|e| PlatformError::Surface(format!("Failed to set app_id: {}", e)))
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        if fullscreen {
            self.toplevel
                .set_fullscreen(None)
                .map_err(|e| PlatformError::Surface(format!("Failed to set fullscreen: {}", e)))?;
        } else {
            self.toplevel.unset_fullscreen().map_err(|e| {
                PlatformError::Surface(format!("Failed to unset fullscreen: {}", e))
            })?;
        }
        Ok(())
    }

    pub fn set_maximized(&mut self, maximized: bool) -> Result<()> {
        if maximized {
            self.toplevel
                .set_maximized()
                .map_err(|e| PlatformError::Surface(format!("Failed to set maximized: {}", e)))?;
        } else {
            self.toplevel
                .unset_maximized()
                .map_err(|e| PlatformError::Surface(format!("Failed to unset maximized: {}", e)))?;
        }
        Ok(())
    }

    pub fn set_minimized(&mut self) -> Result<()> {
        self.toplevel
            .set_minimized()
            .map_err(|e| PlatformError::Surface(format!("Failed to set minimized: {}", e)))
    }

    pub fn start_resize(&mut self, edge: WindowResizeEdge) -> Result<()> {
        self.toplevel
            .resize(None, i32::from(edge))
            .map_err(|e| PlatformError::Surface(format!("Failed to start resize: {}", e)))
    }

    pub fn move_(self: &mut XdgWindow) -> Result<()> {
        self.toplevel
            .move_(None, 0)
            .map_err(|e| PlatformError::Surface(format!("Failed to start move: {}", e)))
    }

    pub fn commit(&mut self) -> Result<()> {
        self.surface
            .commit()
            .map_err(|e| PlatformError::Surface(format!("Failed to commit surface: {}", e)))?;
        self.committed = true;
        Ok(())
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn add_state(&mut self, state: WindowState) {
        if !self.states.contains(&state) {
            self.states.push(state);
        }
    }

    pub fn remove_state(&mut self, state: WindowState) {
        self.states.retain(|s| *s != state);
    }

    pub fn get_states(&self) -> &[WindowState] {
        &self.states
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
    ) -> Result<u32> {
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

    pub fn windows_mut(&mut self) -> impl Iterator<Item = &mut XdgWindow> {
        self.windows.values_mut()
    }
}

impl Default for XdgWindowManager {
    fn default() -> Self {
        Self::new()
    }
}
