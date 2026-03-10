use std::collections::HashMap;
use std::os::fd::RawFd;
use std::sync::Arc;

use parking_lot::Mutex;
use wayland_client::{
    protocol::{wl_compositor, wl_display, wl_shm, wl_subcompositor, wl_surface},
    Connection, Dispatch, EventQueue, Global, Proxy, QueueHandle, WaylandProxy,
};
use wayland_protocols::wlr::unstable_layer_shell::v1::client::zwlr_layer_shell_v1;
use wayland_protocols::xdg::shell::client::xdg_wm_base;

use crate::error::PlatformError;
use crate::Result;

pub struct WaylandState {
    pub display: Proxy<wl_display::WlDisplay>,
    pub compositor: Proxy<wl_compositor::WlCompositor>,
    pub subcompositor: Proxy<wl_subcompositor::WlSubcompositor>,
    pub shm: Proxy<wl_shm::WlShm>,
    pub layer_shell: Option<Proxy<zwlr_layer_shell_v1::ZwlrLayerShellV1>>,
    pub xdg_wm_base: Option<Proxy<xdg_wm_base::XdgWmBase>>,
    pub registry: HashMap<String, Global<usize>>,
}

pub struct WaylandConnection {
    connection: Connection,
    event_queue: EventQueue,
    state: Arc<Mutex<WaylandState>>,
    pub(super) fd: RawFd,
}

impl WaylandConnection {
    pub fn new() -> Result<Self> {
        let connection = Connection::connect_to_env()
            .map_err(|e| PlatformError::Wayland(format!("Failed to connect to Wayland: {}", e)))?;

        let display = connection.display();
        let mut event_queue = connection.new_event_queue();
        let state = Arc::new(Mutex::new(WaylandState {
            display: display.clone(),
            compositor: unsafe {
                display.interpret_announcement::<wl_compositor::WlCompositor>(
                    &mut event_queue.handle(),
                )
            }
            .map_err(|e| PlatformError::Wayland(format!("Failed to get compositor: {}", e)))?
            .unwrap_or_else(|| panic!("Compositor not available")),
            subcompositor: unsafe {
                display.interpret_announcement::<wl_subcompositor::WlSubcompositor>(
                    &mut event_queue.handle(),
                )
            }
            .map_err(|e| PlatformError::Wayland(format!("Failed to get subcompositor: {}", e)))?
            .unwrap_or_else(|| panic!("Subcompositor not available")),
            shm: unsafe {
                display.interpret_announcement::<wl_shm::WlShm>(&mut event_queue.handle())
            }
            .map_err(|e| PlatformError::Wayland(format!("Failed to get SHM: {}", e)))?
            .unwrap_or_else(|| panic!("SHM not available")),
            layer_shell: unsafe {
                display.interpret_announcement::<zwlr_layer_shell_v1::ZwlrLayerShellV1>(
                    &mut event_queue.handle(),
                )
            }
            .map_err(|e| PlatformError::Wayland(format!("Failed to get layer shell: {}", e)))?
            .ok(),
            xdg_wm_base: unsafe {
                display.interpret_announcement::<xdg_wm_base::XdgWmBase>(&mut event_queue.handle())
            }
            .map_err(|e| PlatformError::Wayland(format!("Failed to get xdg wm base: {}", e)))?
            .ok(),
            registry: HashMap::new(),
        }));

        let fd = connection.since();

        let conn = Self {
            connection,
            event_queue,
            state,
            fd,
        };

        conn.roundtrip()?;

        Ok(conn)
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn roundtrip(&self) -> Result<()> {
        self.event_queue
            .roundtrip(&mut WaylandDispatcher)
            .map_err(|e| PlatformError::Wayland(format!("Roundtrip failed: {}", e)))?;
        Ok(())
    }

    pub fn state(&self) -> Arc<Mutex<WaylandState>> {
        self.state.clone()
    }

    pub fn get_surface(&self) -> Result<Proxy<wl_surface::WlSurface>> {
        let state = self.state.lock();
        state
            .compositor
            .create_surface(&mut self.event_queue.handle())
            .map_err(|e| PlatformError::Surface(format!("Failed to create surface: {}", e)))
    }

    pub fn queue(&self) -> QueueHandle {
        self.event_queue.handle()
    }
}

struct WaylandDispatcher;

impl Dispatch<wl_display::WlDisplay, ()> for WaylandDispatcher {
    fn event(
        _client: &wayland_client::Client,
        _proxy: &wl_display::WlDisplay,
        event: wl_display::Event,
        _data: &mut (),
        _conn: &Connection,
        _qh: &QueueHandle,
    ) {
        match event {
            wl_display::Event::DeleteId { .. } => {}
            _ => {}
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for WaylandDispatcher {
    fn event(
        _client: &wayland_client::Client,
        _proxy: &wl_shm::WlShm,
        event: wl_shm::Event,
        _data: &mut (),
        _conn: &Connection,
        _qh: &QueueHandle,
    ) {
        match event {
            wl_shm::Event::Format { .. } => {}
            _ => {}
        }
    }
}
