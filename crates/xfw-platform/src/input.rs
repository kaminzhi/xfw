use std::collections::HashMap;
use std::sync::mpsc::Sender;

use wayland_client::protocol::wl_seat;
use wayland_client::{protocol::wl_pointer, Dispatch, Proxy, QueueHandle};

use crate::error::PlatformError;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
    Other(u32),
}

impl From<u32> for PointerButton {
    fn from(button: u32) -> Self {
        match button {
            wl_pointer::Button::ButtonLeft => PointerButton::Left,
            wl_pointer::Button::ButtonRight => PointerButton::Right,
            wl_pointer::Button::ButtonMiddle => PointerButton::Middle,
            wl_pointer::Button::ButtonWheelUp => PointerButton::WheelUp,
            wl_pointer::Button::ButtonWheelDown => PointerButton::WheelDown,
            other => PointerButton::Other(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone)]
pub struct PointerEvent {
    pub surface_id: u32,
    pub x: f64,
    pub y: f64,
    pub button: Option<PointerButton>,
    pub state: KeyState,
    pub axis: Option<(PointerAxis, f64)>,
}

#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub surface_id: u32,
    pub key: u32,
    pub state: KeyState,
    pub raw_key: u32,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    PointerEnter { surface_id: u32, x: f64, y: f64 },
    PointerLeave { surface_id: u32 },
}

pub struct InputState {
    pub pointer_x: f64,
    pub pointer_y: f64,
    pub focused_surface: Option<u32>,
    pub hovered_surface: Option<u32>,
    pub seat: Option<Proxy<wl_seat::WlSeat>>,
    pub pointer: Option<Proxy<wl_pointer::WlPointer>>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pointer_x: 0.0,
            pointer_y: 0.0,
            focused_surface: None,
            hovered_surface: None,
            seat: None,
            pointer: None,
        }
    }

    pub fn update_pointer_position(&mut self, x: f64, y: f64) {
        self.pointer_x = x;
        self.pointer_y = y;
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InputManager {
    state: InputState,
    event_sender: Option<Sender<InputEvent>>,
    surface_map: HashMap<u32, u32>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
            event_sender: None,
            surface_map: HashMap::new(),
        }
    }

    pub fn set_event_sender(&mut self, sender: Sender<InputEvent>) {
        self.event_sender = Some(sender);
    }

    pub fn register_surface(&mut self, surface_id: u32, wl_surface_id: u32) {
        self.surface_map.insert(wl_surface_id, surface_id);
    }

    pub fn get_surface_id(&self, wl_surface_id: u32) -> Option<u32> {
        self.surface_map.get(&wl_surface_id).copied()
    }

    pub fn get_hovered_surface(&self) -> Option<u32> {
        self.state.hovered_surface
    }

    fn send_event(&self, event: InputEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }

    pub fn handle_pointer_enter(&mut self, surface_id: u32, x: f64, y: f64) {
        self.state.hovered_surface = Some(surface_id);
        self.state.update_pointer_position(x, y);
        self.send_event(InputEvent::PointerEnter { surface_id, x, y });
    }

    pub fn handle_pointer_leave(&mut self, surface_id: u32) {
        if self.state.hovered_surface == Some(surface_id) {
            self.state.hovered_surface = None;
        }
        self.send_event(InputEvent::PointerLeave { surface_id });
    }

    pub fn handle_pointer_motion(&mut self, surface_id: u32, x: f64, y: f64) {
        self.state.update_pointer_position(x, y);
        self.send_event(InputEvent::Pointer(PointerEvent {
            surface_id,
            x,
            y,
            button: None,
            state: KeyState::Pressed,
            axis: None,
        }));
    }

    pub fn handle_pointer_button(
        &mut self,
        surface_id: u32,
        button: PointerButton,
        state: KeyState,
    ) {
        self.send_event(InputEvent::Pointer(PointerEvent {
            surface_id,
            x: self.state.pointer_x,
            y: self.state.pointer_y,
            button: Some(button),
            state,
            axis: None,
        }));
    }

    pub fn handle_pointer_axis(&mut self, surface_id: u32, axis: PointerAxis, value: f64) {
        self.send_event(InputEvent::Pointer(PointerEvent {
            surface_id,
            x: self.state.pointer_x,
            y: self.state.pointer_y,
            button: None,
            state: KeyState::Pressed,
            axis: Some((axis, value)),
        }));
    }

    pub fn hit_test(&self, x: f64, y: f64, surfaces: &[(u32, f32, f32, f32, f32)]) -> Option<u32> {
        for &(id, sx, sy, sw, sh) in surfaces.iter().rev() {
            if x >= sx as f64 && x < (sx + sw) as f64 && y >= sy as f64 && y < (sy + sh) as f64 {
                return Some(id);
            }
        }
        None
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
