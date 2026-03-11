use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use wayland_client::protocol::wl_surface;
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::connection::WaylandConnection;
use crate::error::protocol_not_supported;
use crate::Result;

static LAYER_SURFACE_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

impl From<Layer> for zwlr_layer_shell_v1::Layer {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::Background => zwlr_layer_shell_v1::Layer::Background,
            Layer::Bottom => zwlr_layer_shell_v1::Layer::Bottom,
            Layer::Top => zwlr_layer_shell_v1::Layer::Top,
            Layer::Overlay => zwlr_layer_shell_v1::Layer::Overlay,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    All,
}

impl Anchor {
    pub fn to_wl(self) -> zwlr_layer_surface_v1::Anchor {
        match self {
            Anchor::Top => zwlr_layer_surface_v1::Anchor::Top,
            Anchor::Bottom => zwlr_layer_surface_v1::Anchor::Bottom,
            Anchor::Left => zwlr_layer_surface_v1::Anchor::Left,
            Anchor::Right => zwlr_layer_surface_v1::Anchor::Right,
            Anchor::TopLeft => {
                zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Left
            }
            Anchor::TopRight => {
                zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Right
            }
            Anchor::BottomLeft => {
                zwlr_layer_surface_v1::Anchor::Bottom | zwlr_layer_surface_v1::Anchor::Left
            }
            Anchor::BottomRight => {
                zwlr_layer_surface_v1::Anchor::Bottom | zwlr_layer_surface_v1::Anchor::Right
            }
            Anchor::All => {
                zwlr_layer_surface_v1::Anchor::Top
                    | zwlr_layer_surface_v1::Anchor::Bottom
                    | zwlr_layer_surface_v1::Anchor::Left
                    | zwlr_layer_surface_v1::Anchor::Right
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardInteractivity {
    None,
    Exclusive,
    OnDemand,
}

impl From<KeyboardInteractivity> for zwlr_layer_surface_v1::KeyboardInteractivity {
    fn from(interactivity: KeyboardInteractivity) -> Self {
        match interactivity {
            KeyboardInteractivity::None => zwlr_layer_surface_v1::KeyboardInteractivity::None,
            KeyboardInteractivity::Exclusive => {
                zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive
            }
            KeyboardInteractivity::OnDemand => {
                zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand
            }
        }
    }
}

#[derive(Clone)]
pub struct LayerSurfaceConfig {
    pub anchor: Anchor,
    pub layer: Layer,
    pub keyboard_interactivity: KeyboardInteractivity,
    pub margin: (i32, i32, i32, i32),
    pub namespace: String,
    pub width: u32,
    pub height: u32,
}

impl Default for LayerSurfaceConfig {
    fn default() -> Self {
        Self {
            anchor: Anchor::Top,
            layer: Layer::Top,
            keyboard_interactivity: KeyboardInteractivity::None,
            margin: (0, 0, 0, 0),
            namespace: "xfw".to_string(),
            width: 0,
            height: 0,
        }
    }
}

#[derive(Clone)]
pub struct LayerSurface {
    pub id: u32,
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    config: LayerSurfaceConfig,
    committed: bool,
    width: u32,
    height: u32,
}

impl LayerSurface {
    pub fn new(connection: &WaylandConnection, config: LayerSurfaceConfig) -> Result<Self> {
        let config_clone = config.clone();
        let state = connection.state();
        let layer_shell = state
            .lock()
            .layer_shell
            .clone()
            .ok_or_else(|| protocol_not_supported("layer shell"))?;

        let surface = connection.get_surface()?;
        let mut qh = connection.queue();

        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            None,
            zwlr_layer_shell_v1::Layer::from(config.layer),
            config.namespace.clone(),
            &mut qh,
            (),
        );

        layer_surface.set_anchor(config.anchor.to_wl());

        layer_surface.set_margin(
            config.margin.0,
            config.margin.1,
            config.margin.2,
            config.margin.3,
        );

        layer_surface.set_keyboard_interactivity(
            zwlr_layer_surface_v1::KeyboardInteractivity::from(config.keyboard_interactivity),
        );

        if config_clone.width > 0 && config_clone.height > 0 {
            layer_surface.set_size(config_clone.width, config_clone.height);
        }

        let id = LAYER_SURFACE_ID.fetch_add(1, Ordering::SeqCst);
        let width = config_clone.width;
        let height = config_clone.height;

        Ok(Self {
            id,
            surface,
            layer_surface,
            config: config_clone,
            committed: false,
            width,
            height,
        })
    }

    pub fn set_size(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width;
        self.height = height;
        self.layer_surface.set_size(width, height);
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        self.surface.commit();
        self.committed = true;
        Ok(())
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_surface_id(&self) -> u32 {
        self.id
    }
}

pub struct SurfaceManager {
    surfaces: HashMap<u32, LayerSurface>,
}

impl SurfaceManager {
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
        }
    }

    pub fn add_surface(&mut self, surface: LayerSurface) -> u32 {
        let id = surface.get_surface_id();
        self.surfaces.insert(id, surface);
        id
    }

    pub fn get_surface(&self, id: u32) -> Option<&LayerSurface> {
        self.surfaces.get(&id)
    }

    pub fn get_surface_mut(&mut self, id: u32) -> Option<&mut LayerSurface> {
        self.surfaces.get_mut(&id)
    }

    pub fn remove_surface(&mut self, id: u32) -> Option<LayerSurface> {
        self.surfaces.remove(&id)
    }

    pub fn surfaces(&self) -> impl Iterator<Item = &LayerSurface> {
        self.surfaces.values()
    }
}

impl Default for SurfaceManager {
    fn default() -> Self {
        Self::new()
    }
}
