use crate::{
    buttons::{AnalogSticks, Button /*GuitarButton*/},
    commands::{CResult, DeviceCInfo, DeviceMode, DeviceState, DeviceType},
    device::PsxDevice,
    driver::{Driver, PsxDriver},
    Gamepad,
};
use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
};

/// Creating an instance [`Controller`] to work with a device connected to ordinary digital pins
pub fn create_psx_controller<Dat, Cmd, Att, Clk, D>(
    dat: Dat,
    cmd: Cmd,
    att: Att,
    clk: Clk,
    delay: D,
) -> Controller<PsxDriver<PsxDevice<Dat, Cmd, Att, Clk, D>>>
where
    Dat: InputPin,
    Cmd: OutputPin,
    Att: OutputPin,
    Clk: OutputPin,
    D: DelayNs,
{
    let dev = PsxDevice::new(dat, cmd, att, clk, delay);
    let driver = PsxDriver::new(dev);
    Controller::new(driver)
}

const NO_BUTTONS: u16 = 0xFFFF;

/// Device management controller
pub struct Controller<D> {
    pub ctype: DeviceType,
    pub state: DeviceState,
    pub cmode: DeviceMode,
    pub is_analog_led: bool,
    pub info: DeviceCInfo,
    analog: AnalogSticks,
    buttons: u16,
    buttons_prev: u16,
    // enable_rumble: bool,
    // enable_pressures: bool,
    is_first_connect: bool,
    driver: D,
}

impl<D: Driver> Controller<D> {
    /// Create new instance of ps2 device
    pub fn new(driver: D) -> Self {
        Self {
            ctype: DeviceType::Unknown,
            state: DeviceState::ConnectionError,
            cmode: DeviceMode::Unknown,
            is_analog_led: false,
            info: DeviceCInfo::default(),
            analog: AnalogSticks::default(),
            buttons: NO_BUTTONS,
            buttons_prev: NO_BUTTONS,
            is_first_connect: true,
            driver,
        }
    }
    /// Connect to device and detect his type
    pub fn connect(&mut self) {
        self.state = self.driver.initialize().into();
        self.state = match self.state {
            DeviceState::Connected => match self.driver.query_model_and_mode() {
                Ok((ctype, is_led)) => {
                    self.ctype = ctype.into();
                    self.cmode = self.driver.current_mode();
                    self.is_analog_led = 0x01 == is_led;
                    Ok(())
                }
                Err(e) => Err(e),
            }
            .into(),
            _ => self.state,
        }
    }
    /// Simple reconnect
    pub fn reconnect(&mut self) {
        self.is_first_connect = false;
        self.state = self.driver.initialize().into();
    }
    /// Polling device buttons and sticks
    pub fn poll(&mut self) {
        match self.state {
            DeviceState::Connected => {
                self.state = self._poll().into();
            }
            _ => self.reconnect(),
        };
    }

    fn _poll(&mut self) -> CResult<()> {
        match self.driver.poll() {
            Ok(buttons) => {
                self.cmode = self.driver.current_mode();
                self.buttons_prev = self.buttons;
                self.buttons = buttons;
                self.analog = self.driver.analog_sticks();
                Ok(())
            }
            Err(e) => {
                self.buttons_prev = NO_BUTTONS;
                self.buttons = NO_BUTTONS;
                self.analog = AnalogSticks::default();
                Err(e)
            }
        }
    }
}

// implementation Gamepad trait for Device
impl<T> Gamepad for Controller<T> {
    fn is_analog(&self) -> bool {
        match self.cmode {
            DeviceMode::Analog => true,
            _ => false,
        }
    }
    /// Any button is pressed
    fn is_active(&self) -> bool {
        self.buttons != NO_BUTTONS
    }
    /// Check button change self state
    /// True if button state changed from up to down or down to up
    fn is_changed(&self, btn: Button) -> bool {
        ((self.buttons_prev ^ self.buttons) & btn as u16) > 0
    }
    /// Check button is pressed
    /// True if button state is down
    fn is_pressed(&self, btn: Button) -> bool {
        (!self.buttons & btn as u16) > 0
    }
    /// Check buttons by mask is pressed
    /// True if all button state is down
    fn is_pressed_all(&self, mask: u16) -> bool {
        (!self.buttons & mask) == mask
    }
    /// Check buttons by mask is pressed
    /// True if any button state is down
    fn is_pressed_any(&self, mask: u16) -> bool {
        (!self.buttons & mask) > 0
    }
    /// Check button is down
    /// True if button state changed from up to down
    fn is_down(&self, btn: Button) -> bool {
        self.is_changed(btn) && self.is_pressed(btn)
    }
    /// Check button is released
    /// True if button state changed from down to up
    fn is_up(&self, btn: Button) -> bool {
        self.is_changed(btn) && (!self.buttons_prev & btn as u16) > 0
    }
    /// List all pressed buttons
    /// Return buttons only state down
    fn pressed_buttons(&self, filter: u16) -> u16 {
        !self.buttons & filter
    }
    /// Analog stricks values
    fn analog_sticks(&self) -> AnalogSticks {
        self.analog
    }
    /// Analog value for Guitar Herro device
    fn whammy_bar(&self) -> u8 {
        self.analog.ly
    }
}
