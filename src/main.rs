#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32f1xx_hal as hal;

use embedded_graphics::{
    fonts::Font8x16, fonts::Text, pixelcolor::BinaryColor, prelude::*, style::TextStyleBuilder,
};
use embedded_hal::{
    blocking::delay::DelayMs,
    digital::v2::OutputPin,
};
use hal::{
    afio::AfioExt,
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    stm32,
    time::U32Ext,
};
use rt::entry;
use ssd1306::{Builder, prelude::*, prelude::GraphicsMode};
use stm32f1xx_hal::{flash::FlashExt, gpio::GpioExt, rcc::RccExt};

mod driver;
mod protocol;
mod utils;

#[entry]
fn main() -> ! {
    handle()
}

fn handle() -> ! {
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
    let mut ds18b20 = driver::Ds18b20::new(wire);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    loop {
        delay.delay_ms(500 as u16);
        led.set_high().unwrap();
        ds18b20.measure_temperature(&mut delay).unwrap();
        delay.delay_ms(500 as u16);
        led.set_low().unwrap();
        let temperature = ds18b20.read_temperature(&mut delay).unwrap();
        let mut buf = [0u8; 64];
        let display_str: &str = format_buffer!(&mut buf, "temperature:\n {}", temperature).unwrap();

        disp.clear();
        Text::new(&display_str, Point::zero())
            .into_styled(text_style)
            .draw(&mut disp)
            .unwrap();

        disp.flush().unwrap();
    }
}
