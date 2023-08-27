#![no_std]

extern crate alloc;

use crate::system::SystemUf2;
use defmt::{debug, info, println};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};

pub mod system;

#[cfg(all(feature = "pico", feature = "simulator"))]
compile_error!("Cannot both select hardware and simulator");

#[cfg(feature = "simulator")]
pub type Display = embedded_graphics_simulator::SimulatorDisplay<BinaryColor>;

pub type DrawResult<T = ()> = Result<T, <Display as DrawTarget>::Error>;

#[cfg(feature = "pico")]
const BINARY_COLOR_ON: BinaryColor = BinaryColor::Off;

#[cfg(feature = "simulator")]
const BINARY_COLOR_ON: BinaryColor = BinaryColor::On;

#[cfg(feature = "simulator")]
const BINARY_COLOR_OFF: BinaryColor = BinaryColor::Off;

#[cfg(feature = "pico")]
const BINARY_COLOR_OFF: BinaryColor = BinaryColor::On;

#[cfg(feature = "pico")]
pub type Display = uc8151::Uc8151<
    rp2040_hal::spi::Spi<rp2040_hal::spi::Enabled, rp2040_hal::pac::SPI0, 8>,
    rp2040_hal::gpio::Pin<
        rp2040_hal::gpio::bank0::Gpio17,
        rp2040_hal::gpio::Output<rp2040_hal::gpio::PushPull>,
    >,
    rp2040_hal::gpio::Pin<
        rp2040_hal::gpio::bank0::Gpio20,
        rp2040_hal::gpio::Output<rp2040_hal::gpio::PushPull>,
    >,
    rp2040_hal::gpio::Pin<
        rp2040_hal::gpio::bank0::Gpio26,
        rp2040_hal::gpio::Input<rp2040_hal::gpio::PullUp>,
    >,
    rp2040_hal::gpio::Pin<
        rp2040_hal::gpio::bank0::Gpio21,
        rp2040_hal::gpio::Output<rp2040_hal::gpio::PushPull>,
    >,
>;

#[derive(Eq, PartialEq, Debug, Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum Button {
    A,
    B,
    C,
    D,
    USER,
}

#[derive(Eq, PartialEq, Debug, defmt::Format)]
#[repr(u8)]
enum CurrentMenu {
    SystemName,
    Version,
}

impl CurrentMenu {
    pub fn change(&mut self, button: Button) {
        match self {
            Self::SystemName if button == Button::B => *self = Self::Version,
            Self::Version if button == Button::B => *self = Self::SystemName,
            _ => {}
        }
    }
}

pub struct Sysbadge<'a> {
    pub display: Display,
    system: &'a SystemUf2,
    current: CurrentMenu,
}

impl<'a> Sysbadge<'a> {
    pub fn new_with_system(display: Display, system: &'a SystemUf2) -> Self {
        Self {
            display,
            system,
            current: CurrentMenu::SystemName,
        }
    }

    pub fn press(&mut self, button: Button) {
        self.current.change(button);
        debug!(
            "Pressed button: {:?}, switched to: {:?}",
            button, self.current
        );
    }

    pub fn draw(&mut self) -> DrawResult {
        self.display.clear(BINARY_COLOR_OFF);
        match self.current {
            CurrentMenu::SystemName => self.draw_system_name(),
            CurrentMenu::Version => self.draw_version(),
        }
    }

    fn draw_system_name(&mut self) -> DrawResult {
        Text::with_alignment(
            self.system.name(),
            self.display.bounding_box().center(),
            MonoTextStyle::new(
                &embedded_graphics::mono_font::ascii::FONT_10X20,
                BINARY_COLOR_ON,
            ),
            Alignment::Center,
        )
        .draw(&mut self.display)?;

        Ok(())
    }

    fn draw_version(&mut self) -> DrawResult {
        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_10X20,
            BINARY_COLOR_ON,
        );

        Text::with_alignment(
            "Sysbadge",
            self.display.bounding_box().center().x_axis() + Point::new(0, 20),
            text_style,
            Alignment::Center,
        )
        .draw(&mut self.display)?;

        Text::with_alignment(
            concat!("Version: ", env!("CARGO_PKG_VERSION")),
            Point::new(5, 60),
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;

        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_9X18,
            BINARY_COLOR_ON,
        );
        Text::with_alignment(
            concat!(
                "matrix: ",
                env!("SYSBADGE_MATRIX", "missing matrix configuration")
            ),
            Point::new(5, 105),
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;
        Text::with_alignment(
            concat!("web: ", env!("SYSBADGE_WEB", "missing web configuration")),
            Point::new(5, 120),
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;

        Ok(())
    }
}
