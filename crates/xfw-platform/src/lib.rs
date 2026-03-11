pub mod buffer;
pub mod clipboard;
pub mod connection;
pub mod error;
pub mod event_loop;
pub mod input;
pub mod surface;
pub mod xdg;

use std::collections::HashMap;
use std::os::fd::RawFd;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use buffer::{BufferConfig, BufferPool, ShmBuffer};
use connection::WaylandConnection;
use error::{buffer_error, Result};
use event_loop::{EventDispatcher, EventLoop, EventSource};
use input::{InputEvent, InputManager};
use mio::event::Event;
use surface::{
    Anchor, KeyboardInteractivity, Layer, LayerSurface, LayerSurfaceConfig, SurfaceManager,
};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceGeometry {
    pub x: f32,
    pub y: f32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    PointerEnter {
        surface_id: u32,
        x: f64,
        y: f64,
    },
    PointerLeave {
        surface_id: u32,
    },
    PointerMove {
        surface_id: u32,
        x: f64,
        y: f64,
    },
    PointerButton {
        surface_id: u32,
        button: u32,
        pressed: bool,
    },
    PointerAxis {
        surface_id: u32,
        horizontal: f64,
        vertical: f64,
    },
    Keyboard {
        surface_id: u32,
        key: u32,
        pressed: bool,
    },
    Keymap {
        fd: RawFd,
        size: u32,
    },
    ConfigChanged {
        width: u32,
        height: u32,
    },
    DataReceived {
        surface_id: u32,
        data: Vec<u8>,
    },
    Wake,
    Quit,
}

pub trait PlatformEventHandler: Send {
    fn handle_event(&mut self, event: PlatformEvent);
}

pub struct PlatformSurface {
    connection: WaylandConnection,
    event_loop: EventLoop,
    surface_manager: SurfaceManager,
    input_manager: InputManager,
    buffer_pools: HashMap<u32, BufferPool>,
    event_receiver: Option<Receiver<PlatformEvent>>,
    event_sender: Option<Sender<PlatformEvent>>,
    surface_geometries: HashMap<u32, SurfaceGeometry>,
}

impl PlatformSurface {
    pub fn new() -> Result<Self> {
        let connection = WaylandConnection::new()?;

        let shm = connection.state().lock().shm.clone();
        let default_config = BufferConfig::new(1920, 1080);
        let buffer_pool = BufferPool::new(shm, default_config);

        let mut event_loop = EventLoop::new()?;

        event_loop.register_fd(connection.fd(), PlatformDispatcher);

        let (event_sender, event_receiver) = channel();

        let mut surface = Self {
            connection,
            event_loop,
            surface_manager: SurfaceManager::new(),
            input_manager: InputManager::new(),
            buffer_pools: HashMap::new(),
            event_receiver: Some(event_receiver),
            event_sender: Some(event_sender),
            surface_geometries: HashMap::new(),
        };

        surface.buffer_pools.insert(0, buffer_pool);

        Ok(surface)
    }

    pub fn dispatch_loop(&mut self) -> Result<()> {
        tracing::info!("entering event loop");

        self.event_loop.run(&mut PlatformDispatcher, None)
    }

    pub fn create_layer_surface(&mut self, config: LayerSurfaceConfig) -> Result<u32> {
        let width = config.width;
        let height = config.height;
        let surface = LayerSurface::new(&self.connection, config)?;

        let surface_id = surface.get_surface_id();
        self.surface_manager.add_surface(surface);

        let shm = self.connection.state().lock().shm.clone();
        let buffer_config = BufferConfig::new(width.max(1), height.max(1));
        self.buffer_pools
            .insert(surface_id, BufferPool::new(shm, buffer_config));

        Ok(surface_id)
    }

    pub fn set_layer_surface_size(
        &mut self,
        surface_id: u32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        if let Some(surface) = self.surface_manager.get_surface_mut(surface_id) {
            surface.set_size(width, height)?;
            surface.commit()?;

            if let Some(pool) = self.buffer_pools.get_mut(&surface_id) {
                pool.resize(width, height, &mut self.connection.queue())?;
            }

            self.surface_geometries.insert(
                surface_id,
                SurfaceGeometry {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                },
            );
        }
        Ok(())
    }

    pub fn commit_layer_surface(&mut self, surface_id: u32) -> Result<()> {
        if let Some(surface) = self.surface_manager.get_surface_mut(surface_id) {
            surface.commit()?;
        }
        Ok(())
    }

    pub fn get_buffer(&mut self, surface_id: u32) -> Result<&mut ShmBuffer> {
        let pool = self
            .buffer_pools
            .get_mut(&surface_id)
            .ok_or_else(|| buffer_error(format!("No buffer pool for surface {}", surface_id)))?;

        pool.acquire(&mut self.connection.queue())
    }

    pub fn attach_buffer(&mut self, surface_id: u32, buffer: &ShmBuffer) -> Result<()> {
        if let Some(surface) = self.surface_manager.get_surface_mut(surface_id) {
            surface.surface.attach(Some(&buffer.buffer), 0, 0);
            surface
                .surface
                .damage(0, 0, buffer.width() as i32, buffer.height() as i32);
        }
        Ok(())
    }

    pub fn get_surface_geometry(&self, surface_id: u32) -> Option<SurfaceGeometry> {
        self.surface_geometries.get(&surface_id).cloned()
    }

    pub fn get_all_surfaces(&self) -> Vec<(u32, SurfaceGeometry)> {
        self.surface_geometries
            .iter()
            .map(|(&id, geo)| (id, geo.clone()))
            .collect()
    }

    pub fn poll_events(&self) -> Vec<PlatformEvent> {
        let mut events = Vec::new();
        if let Some(receiver) = &self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }
        events
    }

    pub fn fd(&self) -> RawFd {
        self.connection.fd()
    }

    pub fn roundtrip(&mut self) -> Result<()> {
        self.connection.roundtrip()
    }

    pub fn get_input_surface_under_cursor(&self, x: f64, y: f64) -> Option<u32> {
        let surfaces: Vec<_> = self
            .surface_geometries
            .iter()
            .map(|(&id, geo)| (id, geo.x, geo.y, geo.width as f32, geo.height as f32))
            .collect();
        self.input_manager.hit_test(x, y, &surfaces)
    }

    pub fn quit(&mut self) {
        self.event_loop.stop();
    }
}

struct PlatformDispatcher;

impl EventDispatcher for PlatformDispatcher {
    fn dispatch(&mut self, source: &dyn EventSource, event: &Event) {
        if event.is_readable() {
            tracing::trace!("event ready on fd");
        }
    }
}
