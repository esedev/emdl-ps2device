pub const DATA_HEADER_SIZE: u8 = 3;
pub const DATA_PAYLOAD_SIZE: u8 = 18;
pub const DATA_SIZE: usize = (DATA_HEADER_SIZE + DATA_PAYLOAD_SIZE) as usize;
pub const TX_PSX: u8 = 0x00;
pub const TX_PS2: u8 = 0x5A;

/// Enum of device commands
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Command {
    QueryDS2AnalogMode = 0x41,
    MainPoll = 0x42,
    Config = 0x43,
    SetModeAndLock = 0x44,    // config mode only
    QueryModelAndMode = 0x45, // config mode only
    GetConst1 = 0x46,         // config mode only
    GetConst2 = 0x47,         // config mode only
    GetConst3 = 0x4C,         // config mode only
    MapMotors = 0x4D,         // config mode only
    SetupPoll = 0x4F,         // config mode only
}

/// enum mode of ps2 device
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum DeviceMode {
    Unknown = 0x00,
    Digital = 0x40,
    Analog = 0x70,
    DualShock2Native = 0xF0,
}

impl From<u8> for DeviceMode {
    fn from(v: u8) -> Self {
        match v & 0xF0 {
            0x40 => Self::Digital,
            0x70 => Self::Analog,
            0xF0 => Self::DualShock2Native,
            _ => Self::Unknown,
        }
    }
}

/// enum type of ps2 device
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum DeviceType {
    Unknown = 0x00,
    DualShock1 = 0x01,
    DualShock2 = 0x03,
    //GuitarHero,
}

impl From<u8> for DeviceType {
    fn from(v: u8) -> Self {
        match v {
            0x01 => Self::DualShock1,
            0x03 => Self::DualShock2,
            _ => Self::Unknown,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct DeviceCInfo {
    pub unknown2: [u8; 5],
    pub unknown1: [u8; 10],
    pub unknown3: [u8; 10],
}

/// enum state of device connection
#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum DeviceState {
    ConnectionError,
    Connected,
}

impl<T> From<CResult<T>> for DeviceState {
    fn from(r: CResult<T>) -> Self {
        match r {
            Ok(_) => Self::Connected,
            _ => Self::ConnectionError,
        }
    }
}

/// Result of operation device
pub type CResult<T> = Result<T, ControllerError>;

///
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum ControllerError {
    NoResponse = 0,
    BadHeader,
    LogicError,
}
