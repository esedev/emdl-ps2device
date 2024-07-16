use crate::buttons::*;
use crate::commands::*;
use crate::device::PsxTrasferData;
// use embedded_hal::spi::{Operation, SpiDevice};

// Command to query device analog mode state
// const CMD_QUERY_DS2_ANALOG_MODE: &[u8] = &[0x01, 0x41, 0x00];
// Command to poll buttons
// const CMD_MAIN_POLL: &[u8] = &[0x01, 0x42, 0x00];

/// Trait implementing a protocol for exchanging commands and data with a device
pub trait Driver {
    /// Initialize controller
    fn initialize(&mut self) -> CResult<()>;
    /// Polling controller state
    fn poll(&mut self) -> CResult<u16>;
    /// Polling controller state and vibrate
    fn poll_vibrate(&mut self, m1: u8, m2: u8) -> CResult<u16>;
    /// Polling controller state and vibrate
    fn poll_vibrate_ex(&mut self, m1: u8, m2: u8) -> CResult<u16>;
    /// Get device status (mode, led)
    fn query_model_and_mode(&mut self) -> CResult<(u8, u8)>;
    /// Read device info
    /// return 3 unknown consts, 10b, 5b and 10b sizes
    fn read_device_info(&mut self) -> CResult<DeviceCInfo>;
    /// Return analog sticks values
    fn analog_sticks(&self) -> AnalogSticks;
    /// Read current device mode
    fn current_mode(&self) -> DeviceMode;
}

/// Implementation [`Driver`] trait for [`PsxDevice`] type
pub struct PsxDriver<Dev> {
    buf: DeviceBuffer,
    dev: Dev,
    cursor: usize,
}

impl<Dev: PsxTrasferData> PsxDriver<Dev> {
    pub fn new(dev: Dev) -> Self {
        let buf = DeviceBuffer::default();
        Self {
            buf,
            dev,
            cursor: 0,
        }
    }

    // Wait first successs answer form device
    fn wait_response(&mut self, retry: u8) -> CResult<()> {
        for _ in 0..retry {
            self.dev.sleep();
            match self.send_query_ds2() {
                Ok(_) => return Ok(()),
                _ => {}
            }
        }
        Err(ControllerError::NoResponse)
    }
    // Simple command
    fn send_query_ds2(&mut self) -> CResult<()> {
        self.send_command(Command::QueryDS2AnalogMode, |me| {
            me.send_bytes(TX_PS2, me.rx_data_rest_len())
        })
    }
    /// Enter/Escape in config mode, closure using for send commands in config mode
    fn configure<F, T>(&mut self, f_cfg: F) -> CResult<T>
    where
        F: Fn(&mut Self) -> CResult<T>,
    {
        self.dev.sleep();
        self.send_command(Command::Config, |me| {
            me.send_byte(0x01);
            me.send_bytes(TX_PSX, me.rx_data_rest_len());
        })?;
        self.dev.sleep();

        let result = (f_cfg)(self);

        self.dev.sleep();
        self.send_command(Command::Config, |me| {
            me.send_byte(0x00);
            me.send_bytes(TX_PSX, me.rx_data_rest_len());
        })?;
        self.dev.sleep();

        result
    }
    /// Send command sequence of bytes, closure will call for send command payload
    fn send_command<T>(&mut self, ncmd: Command, f_send_data: T) -> CResult<()>
    where
        T: Fn(&mut Self),
    {
        self.dev.start_cmd();
        self.send_header(ncmd);
        if self.buf.rx_is_header_success() {
            (f_send_data)(self);
            self.dev.stop_cmd();
            // self.dbg_cmd_drop(ncmd);
            Ok(())
        } else {
            self.dev.stop_cmd();
            // self.dbg_cmd_drop(ncmd);
            Err(ControllerError::BadHeader)
        }
    }
    fn send_header(&mut self, ncmd: Command) {
        self.cursor = 0;
        self.__tx_rx__(0x01);
        self.__tx_rx__(ncmd as u8);
        self.__tx_rx__(0x00);
    }
    fn send_byte(&mut self, byte: u8) {
        self.__tx_rx__(byte);
    }
    fn send_bytes(&mut self, byte: u8, repeat: u8) {
        for _ in 0..repeat {
            self.__tx_rx__(byte);
        }
    }
    fn __tx_rx__(&mut self, byte: u8) {
        if self.cursor >= DATA_SIZE {
            panic!("Firmware error");
        }
        //self.cbuf[self.data_cursor] = byte;
        self.buf.data[self.cursor] = self.dev.send_8bit(byte);
        self.cursor += 1;
    }
    fn rx_data_rest_len(&self) -> u8 {
        let len = self.buf.rx_data_len();
        let len_sended = self.cursor as u8 - DATA_HEADER_SIZE;
        if len_sended >= len {
            return 0;
        }
        len - len_sended
    }
}

impl<Dev> Driver for PsxDriver<Dev>
where
    Dev: PsxTrasferData,
{
    /// Initialize controller
    fn initialize(&mut self) -> CResult<()> {
        self.wait_response(10)?;
        //self.configure(|_| Ok(()));
        self.dev.sleep();
        Ok(())
    }
    /// Polling controller state
    fn poll(&mut self) -> CResult<u16> {
        self.send_command(Command::MainPoll, |me| {
            me.send_bytes(TX_PS2, me.rx_data_rest_len());
        })?;
        Ok(self.buf.rx_buttons())
    }

    fn poll_vibrate(&mut self, m1: u8, m2: u8) -> CResult<u16> {
        self.send_command(Command::MainPoll, |me| {
            me.send_byte(DeviceBuffer::tx_normolize_motor(m1));
            me.send_byte(DeviceBuffer::tx_normolize_motor(m2));
            me.send_bytes(TX_PSX, me.rx_data_rest_len());
        })?;
        Ok(self.buf.rx_buttons())
    }

    fn poll_vibrate_ex(&mut self, m1: u8, m2: u8) -> CResult<u16> {
        self.send_command(Command::MainPoll, |me| {
            me.send_byte(DeviceBuffer::tx_normolize_motor(m1));
            me.send_byte(DeviceBuffer::tx_normolize_motor(m2));
            me.send_bytes(TX_PSX, me.rx_data_rest_len());
        })?;
        Ok(self.buf.rx_buttons())
    }

    fn query_model_and_mode(&mut self) -> CResult<(u8, u8)> {
        self.configure(|me| {
            me.send_command(Command::QueryModelAndMode, |drv| {
                drv.send_bytes(TX_PS2, drv.rx_data_rest_len());
            })?;
            Ok((me.buf.rx_data_model(), me.buf.rx_data_mode()))
        })
    }

    fn read_device_info(&mut self) -> CResult<DeviceCInfo> {
        self.configure(|me| {
            let mut data = DeviceCInfo::default();
            // read unknown 1 part 1
            me.send_command(Command::GetConst1, |drv| {
                drv.send_byte(0x00u8);
                drv.send_bytes(TX_PS2, drv.rx_data_rest_len());
            })?;
            me.dev.sleep();
            for i in 0..5 {
                data.unknown1[i] = me.buf.data[i + 4];
            }
            // read unknown 1 part 2
            me.send_command(Command::GetConst1, |drv| {
                drv.send_byte(0x01u8);
                drv.send_bytes(TX_PS2, drv.rx_data_rest_len());
            })?;
            me.dev.sleep();
            for i in 0..5 {
                data.unknown1[i + 5] = me.buf.data[i + 4];
            }
            // read unknown 2 part 1
            me.send_command(Command::GetConst2, |drv| {
                drv.send_byte(0x00u8);
                drv.send_bytes(TX_PS2, drv.rx_data_rest_len());
            })?;
            me.dev.sleep();
            for i in 0..5 {
                data.unknown2[i] = me.buf.data[i + 4];
            }
            // read unknown 3 part 1
            me.send_command(Command::GetConst3, |drv| {
                drv.send_byte(0x00u8);
                drv.send_bytes(TX_PS2, drv.rx_data_rest_len());
            })?;
            me.dev.sleep();
            for i in 0..5 {
                data.unknown3[i] = me.buf.data[i + 4];
            }
            // read unknown 3 part 2
            me.send_command(Command::GetConst3, |drv| {
                drv.send_byte(0x01u8);
                drv.send_bytes(TX_PS2, drv.rx_data_rest_len());
            })?;
            me.dev.sleep();
            for i in 0..5 {
                data.unknown3[i + 5] = me.buf.data[i + 4];
            }
            //
            Ok(data)
        })
    }

    fn analog_sticks(&self) -> AnalogSticks {
        if self.buf.rx_is_analog_mode() {
            self.buf.rx_analog_sticks()
        } else {
            AnalogSticks::default()
        }
    }

    fn current_mode(&self) -> DeviceMode {
        self.buf.rx_data_id().into()
    }
}

/// Buffer for data
pub struct DeviceBuffer {
    pub data: [u8; DATA_SIZE],
}

impl DeviceBuffer {
    /// function rx_data_id, RX data[1] - ID
    /// 0x4w => digital mode
    /// 0x7w => analog mode
    /// 0xFw => config mode
    /// where w is data len in u16
    fn rx_data_id(&self) -> u8 {
        // upper nibble is the ID of the peripheral
        0xF0 & self.data[1]
    }
    fn rx_data_word_count(&self) -> u8 {
        // lower nibble is the output size in u16 (1, 3 or 9)
        0x0F & self.data[1]
    }
    fn rx_data_model(&self) -> u8 {
        self.data[3]
    }
    fn rx_data_mode(&self) -> u8 {
        self.data[5]
    }
    fn rx_analog_sticks(&self) -> AnalogSticks {
        AnalogSticks::new(self.data[7], self.data[8], self.data[5], self.data[6])
    }
    fn rx_data_len(&self) -> u8 {
        match self.rx_data_word_count() {
            1 => 2,
            3 => 6,
            _ => 18,
        }
    }
    fn rx_is_header_success(&self) -> bool {
        let wc = self.rx_data_word_count();
        if wc != 1 && wc != 3 && wc != 9 {
            return false;
        }
        0xFF == self.data[0] && self.rx_is_any_mode()
    }
    fn rx_is_any_mode(&self) -> bool {
        match self.rx_data_id() {
            0x40 | 0x70 | 0xF0 => true,
            _ => false,
        }
    }

    fn rx_is_analog_mode(&self) -> bool {
        0x70 == self.rx_data_id()
    }
    // fn rx_is_config_mode(&self) -> bool {
    //     0xF0 == self.rx_data_id()
    // }
    // fn rx_is_mode_changed(&self) -> bool {
    //     0x00 == self.data[2]
    // }
    // fn is_still_in_config_mode(&self) -> bool {
    //     self.rx_is_config_mode() && !self.rx_is_mode_changed()
    // }
    fn rx_buttons(&self) -> u16 {
        ((self.data[4] as u16) << 8) | (self.data[3] as u16)
    }
    // fn rx_is_dual_shock_native_mode(&self) -> bool {
    //     0xFF == self.data[3] && 0xFF == self.data[4]
    // }
    fn tx_normolize_motor(motor_value: u8) -> u8 {
        match motor_value {
            0x00 => 0x00,
            _ => (0x40 + (motor_value as u16) * (0xff - 0x40) / 0xff) as u8,
        }
    }
}
impl Default for DeviceBuffer {
    fn default() -> Self {
        Self {
            data: [0u8; DATA_SIZE],
        }
    }
}
