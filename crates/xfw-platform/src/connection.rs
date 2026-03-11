use std::collections::HashMap;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::sync::Arc;

use parking_lot::Mutex;
use tracing;
use wayland_client::globals::{registry_queue_init, Global, GlobalList, GlobalListContents};
use wayland_client::protocol::{
    wl_buffer, wl_compositor, wl_display, wl_registry, wl_shm, wl_shm_pool, wl_subcompositor,
    wl_surface,
};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::xdg::shell::client::{
    xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

use crate::error::wayland_error;
use crate::Result;

pub struct WaylandState {
    pub display: wl_display::WlDisplay,
    pub compositor: wl_compositor::WlCompositor,
    pub subcompositor: wl_subcompositor::WlSubcompositor,
    pub shm: wl_shm::WlShm,
    pub layer_shell: Option<ZwlrLayerShellV1>,
    pub xdg_wm_base: Option<XdgWmBase>,
    pub registry: HashMap<String, Global>,
}

pub struct WaylandConnection {
    connection: Connection,
    event_queue: EventQueue<WaylandDispatcher>,
    globals: GlobalList,
    state: Arc<Mutex<WaylandState>>,
    pub(super) fd: RawFd,
}

impl WaylandConnection {
    pub fn new() -> Result<Self> {
        let connection = Connection::connect_to_env()
            .map_err(|e| wayland_error(format!("Failed to connect to Wayland: {}", e)))?;

        let display = connection.display();

        let (globals, event_queue) = registry_queue_init::<WaylandDispatcher>(&connection)
            .map_err(|e| wayland_error(format!("Failed to init registry: {}", e)))?;

        let compositor: wl_compositor::WlCompositor =
            globals.bind(&event_queue.handle(), 1..=6, ()).unwrap();
        let subcompositor: wl_subcompositor::WlSubcompositor =
            globals.bind(&event_queue.handle(), 1..=1, ()).unwrap();
        let shm: wl_shm::WlShm = globals.bind(&event_queue.handle(), 1..=1, ()).unwrap();

        let layer_shell: Option<ZwlrLayerShellV1> =
            globals.bind(&event_queue.handle(), 1..=4, ()).ok();
        let xdg_wm_base: Option<XdgWmBase> = globals.bind(&event_queue.handle(), 1..=1, ()).ok();

        let state = Arc::new(Mutex::new(WaylandState {
            display: display.clone(),
            compositor,
            subcompositor,
            shm,
            layer_shell,
            xdg_wm_base,
            registry: HashMap::new(),
        }));

        let fd = connection.as_fd().as_raw_fd();

        let conn = Self {
            connection,
            event_queue,
            globals,
            state,
            fd,
        };

        Ok(conn)
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn roundtrip(&mut self) -> Result<()> {
        self.connection
            .roundtrip()
            .map(|_: usize| ())
            .map_err(|e| wayland_error(format!("Roundtrip failed: {}", e)))
    }

    pub fn state(&self) -> Arc<Mutex<WaylandState>> {
        self.state.clone()
    }

    pub fn get_surface(&self) -> Result<wl_surface::WlSurface> {
        let mut qh = self.event_queue.handle();
        Ok(self.state.lock().compositor.create_surface(&mut qh, ()))
    }

    pub fn queue(&self) -> QueueHandle<WaylandDispatcher> {
        self.event_queue.handle()
    }
}

pub struct WaylandDispatcher;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                tracing::debug!("Global: {} {} v{}", name, interface, version);
            }
            wl_registry::Event::GlobalRemove { name: _ } => {}
            _ => {}
        }
    }
}

impl Dispatch<wl_display::WlDisplay, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &wl_display::WlDisplay,
        event: wl_display::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
        match event {
            wl_display::Event::DeleteId { .. } => {}
            _ => {}
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &wl_shm::WlShm,
        event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
        match event {
            wl_shm::Event::Format { .. } => {}
            _ => {}
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: <wl_shm_pool::WlShmPool as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &wl_buffer::WlBuffer,
        _event: <wl_buffer::WlBuffer as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &wl_compositor::WlCompositor,
        _event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<wl_subcompositor::WlSubcompositor, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &wl_subcompositor::WlSubcompositor,
        _event: <wl_subcompositor::WlSubcompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &wl_surface::WlSurface,
        _event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &ZwlrLayerShellV1,
        _event: <ZwlrLayerShellV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<XdgWmBase, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &XdgWmBase,
        _event: <XdgWmBase as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &ZwlrLayerSurfaceV1,
        _event: <ZwlrLayerSurfaceV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<XdgSurface, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &XdgSurface,
        _event: <XdgSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}

impl Dispatch<XdgToplevel, ()> for WaylandDispatcher {
    fn event(
        _dispatcher: &mut WaylandDispatcher,
        _proxy: &XdgToplevel,
        _event: <XdgToplevel as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandDispatcher>,
    ) {
    }
}
