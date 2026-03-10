use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{Error as IoError, Map, MmapOptions, Write};
use std::os::fd::{FromRawFd, IntoRawFd, RawFd};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use wayland_client::protocol::{wl_buffer, wl_shm, wl_shm_pool};
use wayland_client::Proxy;

use crate::error::PlatformError;
use crate::Result;
use anyhow::anyhow;

static BUFFER_ID: AtomicU32 = AtomicU32::new(0);

pub struct BufferConfig {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: u32,
}

impl BufferConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            stride: width * 4,
            format: wl_shm::Format::Xrgb8888 as u32,
        }
    }

    pub fn with_stride(mut self, stride: u32) -> Self {
        self.stride = stride;
        self
    }

    pub fn with_format(mut self, format: u32) -> Self {
        self.format = format;
        self
    }

    pub fn size(&self) -> usize {
        (self.stride * self.height) as usize
    }
}

pub struct ShmBuffer {
    pub id: u32,
    pub buffer: Proxy<wl_buffer::WlBuffer>,
    pub config: BufferConfig,
    file: File,
    _mapping: Map,
    data: Vec<u8>,
    in_use: bool,
}

impl ShmBuffer {
    pub fn new(
        shm: &Proxy<wl_shm::WlShm>,
        qh: &mut wayland_client::QueueHandle,
        config: BufferConfig,
    ) -> Result<Self> {
        let size = config.size();
        let fd = Self::create_shm_fd(size)?;

        let mut file = unsafe { File::from_raw_fd(fd) };

        file.write_all(&vec![0u8; size])
            .map_err(|e| PlatformError::Buffer(format!("Failed to initialize SHM: {}", e)))?;

        let pool = shm
            .create_pool(fd, size as i32, qh)
            .map_err(|e| PlatformError::Buffer(format!("Failed to create pool: {}", e)))?;

        let buffer = pool
            .create_buffer(
                0,
                config.width as i32,
                config.height as i32,
                config.stride,
                config.format,
                qh,
            )
            .map_err(|e| PlatformError::Buffer(format!("Failed to create buffer: {}", e)))?;

        let mapping = unsafe {
            MmapOptions::new()
                .len(size)
                .map(&file)
                .map_err(|e| PlatformError::Buffer(format!("Failed to mmap: {}", e)))?
        };

        let data = unsafe { std::slice::from_raw_parts_mut(mapping.as_mut_ptr(), size).to_vec() };

        let id = BUFFER_ID.fetch_add(1, Ordering::SeqCst);

        Ok(Self {
            id,
            buffer,
            config,
            file,
            _mapping: mapping,
            data,
            in_use: false,
        })
    }

    fn create_shm_fd(size: usize) -> Result<RawFd> {
        #[cfg(target_os = "linux")]
        {
            use libc::memfd_create;
            use std::ffi::CString;

            let name = CString::new("xfw-shm").unwrap();
            let fd = unsafe { memfd_create(name.as_ptr(), 0) };
            if fd < 0 {
                return Err(PlatformError::Buffer(format!(
                    "memfd_create failed: {}",
                    IoError::last_os_error()
                )));
            }

            if unsafe { libc::ftruncate(fd, size as libc::off_t) } < 0 {
                return Err(PlatformError::Buffer(format!(
                    "ftruncate failed: {}",
                    IoError::last_os_error()
                )));
            }

            Ok(fd)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let temp_dir = std::env::temp_dir();
            let path = temp_dir.join(format!("xfw-shm-{}", std::process::id()));
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .map_err(|e| PlatformError::Buffer(format!("Failed to create temp file: {}", e)))?;

            file.set_len(size as u64)
                .map_err(|e| PlatformError::Buffer(format!("Failed to set file size: {}", e)))?;

            Ok(file.into_raw_fd())
        }
    }

    pub fn data(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn set_in_use(&mut self, in_use: bool) {
        self.in_use = in_use;
    }

    pub fn is_in_use(&self) -> bool {
        self.in_use
    }

    pub fn width(&self) -> u32 {
        self.config.width
    }

    pub fn height(&self) -> u32 {
        self.config.height
    }

    pub fn stride(&self) -> u32 {
        self.config.stride
    }
}

impl Drop for ShmBuffer {
    fn drop(&mut self) {
        self.buffer.destroy();
    }
}

pub struct BufferPool {
    pool: VecDeque<ShmBuffer>,
    config: BufferConfig,
    shm: Proxy<wl_shm::WlShm>,
}

impl BufferPool {
    pub fn new(shm: Proxy<wl_shm::WlShm>, config: BufferConfig) -> Self {
        Self {
            pool: VecDeque::new(),
            config,
            shm,
        }
    }

    pub fn acquire(&mut self, qh: &mut wayland_client::QueueHandle) -> Result<&mut ShmBuffer> {
        if let Some(buffer) = self.pool.iter_mut().find(|b| !b.is_in_use()) {
            buffer.set_in_use(true);
            return Ok(buffer);
        }

        let mut buffer = ShmBuffer::new(&self.shm, qh, self.config.clone())?;
        buffer.set_in_use(true);
        self.pool.push_back(buffer);

        self.pool
            .back_mut()
            .ok_or_else(|| anyhow!("Failed to acquire buffer"))
    }

    pub fn release(&mut self, buffer_id: u32) {
        if let Some(buffer) = self.pool.iter_mut().find(|b| b.id == buffer_id) {
            buffer.set_in_use(false);
        }
    }

    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        qh: &mut wayland_client::QueueHandle,
    ) -> Result<()> {
        self.config = BufferConfig::new(width, height);
        self.pool.clear();
        self.acquire(qh)?;
        Ok(())
    }
}
