use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};

pub(crate) struct OneWirePin<P> {
    pin: P,
}

impl<'a, P, E> OneWirePin<P>
    where P: OutputPin<Error=E> + InputPin<Error=E>,
{
    pub fn new(pin: P) -> OneWirePin<P> {
        OneWirePin { pin }
    }

    pub fn init(&mut self, delay: &mut impl DelayUs<u16>) -> Result<bool, E> {
        self.pin.set_low()?;
        delay.delay_us(480);
        self.pin.set_high()?;
        delay.delay_us(65);
        let result = self.pin.is_low();
        delay.delay_us(415);
        result
    }

    pub fn write_byte_array(&mut self, data: &[u8], delay: &mut impl DelayUs<u16>) -> Result<(), E> {
        for byte in data {
            self.write_byte(*byte, delay)?;
        }
        Ok(())
    }

    pub fn write_byte(&mut self, mut byte: u8, delay: &mut impl DelayUs<u16>) -> Result<(), E> {
        for _ in 0..8 {
            if (byte & 0x01) == 0x01 {
                self.pin.set_low()?;
                delay.delay_us(1 as u16);
                self.pin.set_high()?;
                delay.delay_us(60 as u16);
            } else {
                self.pin.set_low()?;
                delay.delay_us(60 as u16);
                self.pin.set_high()?;
                delay.delay_us(1 as u16);
            };
            byte >>= 1;
        }
        Ok(())
    }

    pub fn read_byte_array(&mut self, buffer: &mut [u8], delay: &mut impl DelayUs<u16>) -> Result<(), E> {
        for i in 0..buffer.len() {
            buffer[i] = self.read_byte(delay)?;
        }
        Ok(())
    }

    fn read_byte(&mut self, delay: &mut impl DelayUs<u16>) -> Result<u8, E> {
        let mut byte: u8 = 0;
        for _ in 0..8 {
            byte >>= 1;
            self.pin.set_low()?;
            delay.delay_us(1 as u16);
            self.pin.set_high()?;
            delay.delay_us(20 as u16);
            let result = self.pin.is_high();
            delay.delay_us(40);
            if result? {
                byte |= 0x80;
            }
        }
        Ok(byte)
    }
}
