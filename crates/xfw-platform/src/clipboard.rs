use parking_lot::Mutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use wayland_client::protocol::{wl_data_device, wl_data_offer, wl_data_source, wl_seat};
use wayland_client::{Dispatch, QueueHandle};

use crate::connection::WaylandState;
use crate::Result;

pub enum ClipboardContent {
    Text(String),
    Image(Vec<u8>),
}

pub struct Clipboard {
    offers: HashMap<u32, ClipboardContent>,
    current_offer: Option<wl_data_offer::WlDataOffer>,
    selection: Option<ClipboardContent>,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            offers: HashMap::new(),
            current_offer: None,
            selection: None,
        }
    }

    pub fn get_selection(&self) -> Option<&ClipboardContent> {
        self.selection.as_ref()
    }

    pub fn set_selection(&mut self, content: ClipboardContent) {
        self.selection = Some(content);
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn add_offer(&mut self, id: u32, content: ClipboardContent) {
        self.offers.insert(id, content);
    }

    pub fn get_offer(&self, id: u32) -> Option<&ClipboardContent> {
        self.offers.get(&id)
    }

    pub fn remove_offer(&mut self, id: u32) {
        self.offers.remove(&id);
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DataDeviceHandler {
    clipboard: Mutex<Clipboard>,
    seat: RefCell<Option<wl_seat::WlSeat>>,
}

impl DataDeviceHandler {
    pub fn new() -> Self {
        Self {
            clipboard: Mutex::new(Clipboard::new()),
            seat: RefCell::new(None),
        }
    }

    pub fn get_clipboard(&self) -> parking_lot::MutexGuard<'_, Clipboard> {
        self.clipboard.lock()
    }

    pub fn set_seat(&self, seat: wl_seat::WlSeat) {
        *self.seat.borrow_mut() = Some(seat);
    }

    pub fn copy_text(&self, text: String) -> Result<()> {
        let mut clipboard = self.clipboard.lock();
        clipboard.set_selection(ClipboardContent::Text(text.clone()));
        clipboard.set_selection(ClipboardContent::Text(text));
        Ok(())
    }

    pub fn paste_text(&self) -> Option<String> {
        let clipboard = self.clipboard.lock();
        match clipboard.get_selection() {
            Some(ClipboardContent::Text(text)) => Some(text.clone()),
            _ => None,
        }
    }
}

impl Default for DataDeviceHandler {
    fn default() -> Self {
        Self::new()
    }
}
