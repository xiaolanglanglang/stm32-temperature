use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use crate::protocol::one_wire::OneWirePin;

pub struct Ds18b20<P> {
    pin: OneWirePin<P>,
}

impl<'a, P, E> Ds18b20<P>
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
{
    pub fn new(raw_pin: P) -> Ds18b20<P> {
        let one_wire_pin = OneWirePin::new(raw_pin);
        Ds18b20 { pin: one_wire_pin }
    }

    pub fn measure_temperature(&mut self, delay: &mut impl DelayUs<u16>) -> Result<(), E> {
        self.pin.init(delay)?;
        self.pin.write_byte(0xCC, delay)?;
        self.pin.write_byte(0x44, delay)?;
        Ok(())
    }

    pub fn read_temperature(&mut self, delay: &mut impl DelayUs<u16>) -> Result<f32, E> {
        let mut output = [0 as u8; 9];
        self.pin.init(delay)?;
        self.pin.write_byte_array(&[0xCC, 0xBE], delay)?;
        self.pin.read_byte_array(&mut output, delay)?;
        let temperature_hex: u16 = u16::from_le_bytes([output[0], output[1]]);
        let temperature: f32 = f32::from(temperature_hex) / 16_f32;
        Ok(temperature)
    }
}
