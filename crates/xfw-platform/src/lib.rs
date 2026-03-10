use anyhow::Result;

pub struct PlatformSurface {}

impl PlatformSurface {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn dispatch_loop(&mut self) -> Result<()> {
        tracing::info!("msg" = "entering event loop");
        // Placeholder: block on Wayland/IPC events.
        Ok(())
    }
}
