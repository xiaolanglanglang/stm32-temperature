#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32f1xx_hal as hal;

use core::cmp::min;
use core::fmt;
use core::fmt::Write;

use cortex_m_semihosting::hio;
use embedded_graphics::fonts::Font8x16;
use embedded_graphics::{
    fonts::Text, pixelcolor::BinaryColor, prelude::*, style::TextStyleBuilder,
};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::afio::AfioExt;
use hal::delay::Delay;
use hal::i2c::{BlockingI2c, DutyCycle, Mode};
use hal::stm32;
use hal::time::U32Ext;
use rt::entry;
use ssd1306::prelude::GraphicsMode;
use ssd1306::{prelude::*, Builder};
use stm32f1xx_hal::flash::FlashExt;
use stm32f1xx_hal::gpio::GpioExt;
use stm32f1xx_hal::rcc::RccExt;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    // Try a different clock configuration
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz()) // 高速外部时钟源
        .sysclk(72.mhz()) // 系统时钟
        .hclk(72.mhz()) // AHB 高速总线
        .pclk1(36.mhz()) // APB1 低速外设总线
        .pclk2(72.mhz()) // APB2 高速外设总线
        .freeze(&mut flash.acr); // 应用时钟配置

    let mut stdout = hio::hstdout().unwrap();
    write!(stdout, "Rust on embedded is 1!\n").unwrap();

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000.hz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );
    let mut disp: GraphicsMode<_> = Builder::new()
        .size(DisplaySize::Display128x32)
        .connect_i2c(i2c)
        .into();
    disp.init().unwrap();
    let text_style = TextStyleBuilder::new(Font8x16)
        .text_color(BinaryColor::On)
        .build();

    let wire = gpiob.pb9.into_open_drain_output(&mut gpiob.crh);
    let mut pin = OneWirePin { pin: wire };

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    loop {
        delay.delay_ms(500 as u16);
        led.set_high().unwrap();
        pin.measure_temperature(&mut delay).unwrap();
        delay.delay_ms(500 as u16);
        led.set_low().unwrap();
        let temperature = pin.read_temperature(&mut delay).unwrap();
        let mut buf = [0u8; 64];
        let display_str: &str =
            show(&mut buf, format_args!("temperature:\n {}", temperature)).unwrap();

        disp.clear();
        Text::new(&display_str, Point::zero())
            .into_styled(text_style)
            .draw(&mut disp)
            .unwrap();

        disp.flush().unwrap();
    }
}

struct OneWirePin<P> {
    pin: P,
}

impl<P, E> OneWirePin<P>
    where
        P: OutputPin<Error=E> + InputPin<Error=E>,
{
    fn read_temperature(
        &mut self,
        delay: &mut (impl DelayUs<u16> + DelayMs<u16>),
    ) -> Result<f32, E> {
        let mut output = [0 as u8; 9];
        self.init(delay)?;
        self.write_byte(0xCC, delay)?;
        self.write_byte(0xBE, delay)?;
        for i in 0..output.len() {
            output[i] = self.read_byte(delay)?;
        }
        let temperature_hex: u16 = u16::from_le_bytes([output[0], output[1]]);
        let temperature: f32 = f32::from(temperature_hex) / 16_f32;
        Ok(temperature)
    }

    fn measure_temperature(
        &mut self,
        delay: &mut (impl DelayUs<u16> + DelayMs<u16>),
    ) -> Result<(), E> {
        self.init(delay)?;
        self.write_byte(0xCC, delay)?;
        self.write_byte(0x44, delay)?;
        Ok(())
    }

    fn init(&mut self, delay: &mut impl DelayUs<u16>) -> Result<bool, E> {
        self.pin.set_low()?;
        delay.delay_us(480);
        self.pin.set_high()?;
        delay.delay_us(65);
        let result = self.pin.is_low();
        delay.delay_us(415);
        result
    }

    fn write_byte(&mut self, mut byte: u8, delay: &mut impl DelayUs<u16>) -> Result<(), E> {
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

pub struct WriteTo<'a> {
    buffer: &'a mut [u8],
    // on write error (i.e. not enough space in buffer) this grows beyond
    // `buffer.len()`.
    used: usize,
}

impl<'a> WriteTo<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        WriteTo { buffer, used: 0 }
    }

    pub fn as_str(self) -> Option<&'a str> {
        if self.used <= self.buffer.len() {
            // only successful concats of str - must be a valid str.
            use core::str::from_utf8_unchecked;
            Some(unsafe { from_utf8_unchecked(&self.buffer[..self.used]) })
        } else {
            None
        }
    }
}

impl<'a> fmt::Write for WriteTo<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.used > self.buffer.len() {
            return Err(fmt::Error);
        }
        let remaining_buf = &mut self.buffer[self.used..];
        let raw_s = s.as_bytes();
        let write_num = min(raw_s.len(), remaining_buf.len());
        remaining_buf[..write_num].copy_from_slice(&raw_s[..write_num]);
        self.used += raw_s.len();
        if write_num < raw_s.len() {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

pub fn show<'a>(buffer: &'a mut [u8], args: fmt::Arguments) -> Result<&'a str, fmt::Error> {
    let mut w = WriteTo::new(buffer);
    fmt::write(&mut w, args)?;
    w.as_str().ok_or(fmt::Error)
}
