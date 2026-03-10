use anyhow::Result;

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn prepare(&mut self) -> Result<()> {
        // Initialize fonts, atlases, etc.
        Ok(())
    }
}
