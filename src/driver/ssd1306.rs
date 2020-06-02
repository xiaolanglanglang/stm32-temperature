use embedded_graphics::{
    fonts::Font8x16, fonts::Text, pixelcolor::BinaryColor, prelude::*, style::TextStyle,
    style::TextStyleBuilder,
};
use ssd1306::{prelude::*, Builder};
use stm32f1xx_hal::{
    i2c::{BlockingI2c, DutyCycle, Mode, Pins},
    pac::I2C1,
    rcc::Clocks,
    time::U32Ext,
};

pub struct Ssd1306<PINS> {
    display: GraphicsMode<I2cInterface<BlockingI2c<I2C1, PINS>>>,
    text_style: TextStyle<BinaryColor, Font8x16>,
}

impl<PINS> Ssd1306<PINS>
where
    PINS: Pins<I2C1>,
{
    pub fn i2c1(
        i2c: I2C1,
        mapr: &mut stm32f1xx_hal::afio::MAPR,
        apb1: &mut stm32f1xx_hal::rcc::APB1,
        clocks: Clocks,
        pins: PINS,
    ) -> Self {
        let i2c = BlockingI2c::i2c1(
            i2c,
            pins,
            mapr,
            Mode::Fast {
                frequency: 400_000.hz(),
                duty_cycle: DutyCycle::Ratio2to1,
            },
            clocks,
            apb1,
            1000,
            10,
            1000,
            1000,
        );
        let mut display: GraphicsMode<_> = Builder::new()
            .size(DisplaySize::Display128x32)
            .connect_i2c(i2c)
            .into();
        display.init().unwrap();
        let text_style = TextStyleBuilder::new(Font8x16)
            .text_color(BinaryColor::On)
            .build();
        Ssd1306 {
            display,
            text_style,
        }
    }

    pub fn show(&mut self, display_str: &str) {
        self.display.clear();
        Text::new(&display_str, Point::zero())
            .into_styled(self.text_style)
            .draw(&mut self.display)
            .unwrap();
        self.display.flush().unwrap();
    }
}
