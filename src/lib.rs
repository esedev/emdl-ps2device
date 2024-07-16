#![no_std]

mod buttons;
mod commands;
mod controller;
mod device;
mod driver;

/// PS2 Gamepad interface
pub trait Gamepad {
    fn is_analog(&self) -> bool;
    fn is_active(&self) -> bool;
    fn is_changed(&self, btn: Ps2Button) -> bool;
    fn is_pressed(&self, btn: Ps2Button) -> bool;
    fn is_pressed_all(&self, mask: u16) -> bool;
    fn is_pressed_any(&self, mask: u16) -> bool;
    fn is_down(&self, btn: Ps2Button) -> bool;
    fn is_up(&self, btn: Ps2Button) -> bool;
    fn pressed_buttons(&self, filter: u16) -> u16;
    fn analog_sticks(&self) -> Ps2AnalogSticks;
    fn whammy_bar(&self) -> u8;
}

pub mod prelude {
    pub use super::buttons::{AnalogSticks as Ps2AnalogSticks, Button as Ps2Button};
    pub use super::commands::{DeviceState as Ps2DeviceState, DeviceType as Ps2DeviceType};
    pub use super::controller::create_psx_controller;
    pub use super::controller::Controller as Ps2Controller;
    pub use super::Gamepad as Ps2Gamepad;
}

pub use self::prelude::*;
