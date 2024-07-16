use core::ops::{BitAnd, BitOr};

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum Button {
    Select = 0x0001,
    LJoyBtn = 0x0002,
    RJoyBtn = 0x0004,
    Start = 0x0008,

    Up = 0x0010,
    Right = 0x0020,
    Down = 0x0040,
    Left = 0x0080,

    LTrigger = 0x0100,
    RTrigger = 0x0200,
    LButton = 0x0400,
    RButton = 0x0800,

    Triangle = 0x1000,
    Circle = 0x2000,
    Cross = 0x4000,
    Square = 0x8000,

    All = 0xFFFF,
    Arrows = 0x00F0,
}

impl BitOr<Button> for Button {
    type Output = u16;
    fn bitor(self, rhs: Self) -> Self::Output {
        (self as u16) | (rhs as u16)
    }
}
impl BitOr<u16> for Button {
    type Output = u16;
    fn bitor(self, rhs: u16) -> Self::Output {
        (self as u16) | rhs
    }
}
impl BitOr<Button> for u16 {
    type Output = u16;
    fn bitor(self, rhs: Button) -> Self::Output {
        self | (rhs as u16)
    }
}

impl BitAnd<Button> for Button {
    type Output = u16;
    fn bitand(self, _rhs: Button) -> Self::Output {
        0x0000
    }
}
impl BitAnd<u16> for Button {
    type Output = u16;
    fn bitand(self, rhs: u16) -> Self::Output {
        (self as u16) & rhs
    }
}
impl PartialEq<u16> for Button {
    fn eq(&self, rhs: &u16) -> bool {
        (*self as u16) == *rhs
    }
}
impl PartialEq<Button> for u16 {
    fn eq(&self, rhs: &Button) -> bool {
        (*self as u16) == *rhs as u16
    }
}
impl From<Button> for u16 {
    fn from(v: Button) -> u16 {
        v as u16
    }
}

#[allow(dead_code)]
#[repr(u16)]
#[derive(Copy, Clone)]
pub enum GuitarButton {
    UpStrum = 0x0010,
    RightStrum = 0x0020,
    DownStrum = 0x0040,
    LeftStrum = 0x0080,
    StartPower = 0x0100,
    GreenFret = 0x0200,
    YellowFret = 0x1000,
    RedFret = 0x2000,
    BlueFret = 0x4000,
    OrangeFret = 0x8000,
}

#[derive(Copy, Clone)]
pub struct AnalogSticks {
    pub lx: u8,
    pub ly: u8,
    pub rx: u8,
    pub ry: u8,
}
impl AnalogSticks {
    pub fn new(lx: u8, ly: u8, rx: u8, ry: u8) -> Self {
        Self { lx, ly, rx, ry }
    }
}
impl Default for AnalogSticks {
    fn default() -> Self {
        Self::new(128, 128, 128, 128)
    }
}
