use std::collections::VecDeque;
use std::fs::File;
use std::io::{Error as IoError, Write};
use std::os::fd::{FromRawFd, RawFd};
use std::sync::atomic::{AtomicU32, Ordering};

use memmap2::Mmap;
use wayland_client::protocol::{wl_buffer, wl_shm, wl_shm_pool::WlShmPool};
use wayland_client::QueueHandle;

use crate::connection::WaylandDispatcher;
use crate::error::buffer_error;
use crate::Result;

static BUFFER_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone)]
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
    pub buffer: wl_buffer::WlBuffer,
    pub config: BufferConfig,
    file: File,
    _mapping: Mmap,
    data: Vec<u8>,
    in_use: bool,
}

impl ShmBuffer {
    pub fn new(
        shm: &wl_shm::WlShm,
        qh: &mut QueueHandle<WaylandDispatcher>,
        config: BufferConfig,
    ) -> Result<Self> {
        let size = config.size();
        let fd = Self::create_shm_fd(size)?;

        let mut file = unsafe { File::from_raw_fd(fd) };

        file.write_all(&vec![0u8; size]).unwrap();

        let borrowed_fd = unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) };
        let pool: WlShmPool = shm.create_pool(borrowed_fd, size as i32, qh, ());

        let format = wl_shm::Format::Xrgb8888;
        let buffer: wl_buffer::WlBuffer = pool.create_buffer(
            0,
            config.width as i32,
            config.height as i32,
            config.stride as i32,
            format,
            qh,
            (),
        );

        let mapping = unsafe {
            Mmap::map(&file).map_err(|e| buffer_error(format!("Failed to mmap: {}", e)))?
        };

        let data = vec![0u8; size];

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
                return Err(buffer_error(format!(
                    "memfd_create failed: {}",
                    IoError::last_os_error()
                )));
            }

            if unsafe { libc::ftruncate(fd, size as libc::off_t) } < 0 {
                return Err(buffer_error(format!(
                    "ftruncate failed: {}",
                    IoError::last_os_error()
                )));
            }

            Ok(fd)
        }

        #[cfg(not(target_os = "linux"))]
        {
            use std::fs::OpenOptions;
            let temp_dir = std::env::temp_dir();
            let path = temp_dir.join(format!("xfw-shm-{}", std::process::id()));
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .map_err(|e| buffer_error(format!("Failed to create temp file: {}", e)))?;

            file.set_len(size as u64)
                .map_err(|e| buffer_error(format!("Failed to set file size: {}", e)))?;

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
    shm: wl_shm::WlShm,
}

impl BufferPool {
    pub fn new(shm: wl_shm::WlShm, config: BufferConfig) -> Self {
        Self {
            pool: VecDeque::new(),
            config,
            shm,
        }
    }

    pub fn acquire(&mut self, qh: &mut QueueHandle<WaylandDispatcher>) -> Result<&mut ShmBuffer> {
        let found = self.pool.iter_mut().find(|b| !b.is_in_use()).map(|b| {
            b.set_in_use(true);
            b.id
        });

        if let Some(id) = found {
            let idx = self.pool.iter().position(|b| b.id == id).unwrap();
            return Ok(&mut self.pool[idx]);
        }

        let buffer = ShmBuffer::new(&self.shm, qh, self.config.clone())?;
        self.pool.push_back(buffer);
        let buffer = self.pool.back_mut().unwrap();
        Ok(buffer)
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
        qh: &mut QueueHandle<WaylandDispatcher>,
    ) -> Result<()> {
        self.config = BufferConfig::new(width, height);
        self.pool.clear();
        self.acquire(qh)?;
        Ok(())
    }
}
