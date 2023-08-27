#![no_std]
#![no_main]
#![feature(allocator_api, alloc_error_handler)]

use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use cortex_m::delay::Delay;
use defmt::info;
// The macro for our start-up function
use pimoroni_badger2040::entry;
use pimoroni_badger2040::hal;
use pimoroni_badger2040::hal::{pac, Clock};

extern crate alloc;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use defmt_rtt as _;
use embedded_graphics::mono_font::iso_8859_3::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use panic_probe as _;

use uc8151::Uc8151;

// GPIO traits
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};
use embedded_hal::digital::v2::OutputPin;
use fugit::{HertzU32, RateExtU32};

#[entry]
fn main() -> ! {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, 2048) }

    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().expect("Failed to get pac");
    let cp = pac::CorePeripherals::take().expect("Failed to get CP");

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        pimoroni_badger2040::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .expect("Failed to setup clock");

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = pimoroni_badger2040::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    pins.sclk.into_mode::<hal::gpio::FunctionSpi>();
    pins.mosi.into_mode::<hal::gpio::FunctionSpi>();
    let spi: hal::spi::Spi<_, _, 8> = hal::spi::Spi::new(pac.SPI0);
    let spi = spi.init(
        &mut pac.RESETS,
        &clocks.peripheral_clock,
        HertzU32::Hz(1000000),
        &embedded_hal::spi::MODE_0,
    );

    let dc = pins.inky_dc.into_push_pull_output();
    let cs = pins.inky_cs_gpio.into_push_pull_output();
    let busy = pins.inky_busy.into_pull_up_input();
    let reset = pins.inky_res.into_push_pull_output();

    let mut delay = Delay::new(cp.SYST, clocks.system_clock.freq().to_Hz());
    let mut display = Uc8151::new(spi, cs, dc, busy, reset);
    info!("Setting up display");
    display
        .setup(&mut delay, uc8151::LUT::Fast)
        .expect("setting up display");

    Text::with_alignment(
        "foo",
        display.bounding_box().center() + Point::new(0, 15),
        MonoTextStyle::new(&FONT_6X10, BinaryColor::Off),
        Alignment::Center,
    )
    .draw(&mut display);

    display.update().expect("updating display");

    loop {}
    defmt::todo!()
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    defmt::panic!("allocation error: {:?}", layout);
}
