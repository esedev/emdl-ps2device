use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    //spi::{ErrorType, Operation as SpiOperation, SpiDevice},
};

const CTRL_CLK: u32 = 2000; // ns
const CTRL_8BIT: u32 = 4000; // ns
const DRIVER_SLEEP: u32 = 8_000_000; // ns

/// Connector
struct Connector<Dat, Cmd, Att, Clk> {
    dat: Dat,
    cmd: Cmd,
    att: Att,
    clk: Clk,
}
impl<Dat, Cmd, Att, Clk> Connector<Dat, Cmd, Att, Clk> {
    pub fn new(dat: Dat, cmd: Cmd, att: Att, clk: Clk) -> Self {
        Self { dat, cmd, att, clk }
    }
}

/// PsxDevice - connecting the device to the digital pins
pub struct PsxDevice<Dat, Cmd, Att, Clk, D> {
    wires: Connector<Dat, Cmd, Att, Clk>,
    delay: D,
    //half_cycle_ns: u32,
}
impl<Dat, Cmd, Att, Clk, D> PsxDevice<Dat, Cmd, Att, Clk, D>
where
    Dat: InputPin,
    Cmd: OutputPin,
    Att: OutputPin,
    Clk: OutputPin,
    D: DelayNs,
{
    pub fn new(dat: Dat, cmd: Cmd, att: Att, clk: Clk, delay: D) -> Self {
        let wires = Connector::new(dat, cmd, att, clk);
        Self {
            wires,
            delay,
            //half_cycle_ns: 2000,
        }
    }
}

/// Send commands and recieve data.
/// Full-duplex protocol operating at 250 kHz
pub trait PsxTrasferData: DelayNs {
    const SLEEP_NS: u32;

    fn start_cmd(&mut self);
    fn stop_cmd(&mut self);
    fn send_8bit(&mut self, byte: u8) -> u8;
    fn sleep(&mut self) {
        self.delay_ns(Self::SLEEP_NS);
    }
}

impl<Dat, Cmd, Att, Clk, D> DelayNs for PsxDevice<Dat, Cmd, Att, Clk, D>
where
    D: DelayNs,
{
    fn delay_ns(&mut self, ns: u32) {
        self.delay.delay_ns(ns)
    }
}

impl<Dat, Cmd, Att, Clk, D> PsxTrasferData for PsxDevice<Dat, Cmd, Att, Clk, D>
where
    Dat: InputPin,
    Cmd: OutputPin,
    Att: OutputPin,
    Clk: OutputPin,
    Self: DelayNs,
{
    const SLEEP_NS: u32 = DRIVER_SLEEP;

    fn start_cmd(&mut self) {
        self.wires.cmd.set_high().unwrap_or(());
        self.wires.clk.set_high().unwrap_or(());
        //self.delay.delay_ns(self.half_cycle_ns);
        self.delay_ns(CTRL_CLK);

        self.wires.att.set_low().unwrap_or(()); // low enable device
                                                //self.delay.delay_ns(self.half_cycle_ns << 1);
        self.delay_ns(CTRL_8BIT)
    }

    fn stop_cmd(&mut self) {
        self.wires.att.set_high().unwrap_or(()); //high disable device
                                                 // self.delay.delay_ns(self.half_cycle_ns << 1);
        self.delay_ns(CTRL_8BIT)
    }

    fn send_8bit(&mut self, byte: u8) -> u8 {
        let mut answer: u8 = 0;
        for i in 0..8 {
            let bit: u8 = 1 << i;
            let val: bool = (byte & bit) > 0;
            self.wires.cmd.set_state(val.into()).unwrap_or(());
            self.wires.clk.set_low().unwrap_or(());
            // self.delay.delay_ns(self.half_cycle_ns);
            self.delay_ns(CTRL_CLK);
            if self.wires.dat.is_high().unwrap_or(false) {
                answer |= bit;
            }
            self.wires.clk.set_high().unwrap_or(());
            // self.delay.delay_ns(self.half_cycle_ns);
            self.delay_ns(CTRL_CLK);
        }
        self.wires.cmd.set_high().unwrap_or(());
        // self.delay.delay_ns(self.half_cycle_ns << 1);
        self.delay_ns(CTRL_8BIT);
        answer
    }
}
