#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(allocator_api, alloc_error_handler)]

use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use cortex_m::delay::Delay;
use defmt::info;
use embassy_time::{Duration, Timer};

// The macro for our start-up function
/*use pimoroni_badger2040::entry;
use pimoroni_badger2040::hal;
use pimoroni_badger2040::hal::{pac, Clock};*/

extern crate alloc;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::spi::Spi;
use embassy_rp::{peripherals, Peripherals};
use panic_probe as _;

use uc8151::Uc8151;

// GPIO traits
use embedded_hal::digital::v2::OutputPin;
use fugit::{HertzU32, RateExtU32};
use sysbadge::{Button, Sysbadge};

static mut SYSBADGE: Option<
    Sysbadge<
        Uc8151<
            Spi<peripherals::SPI0, embassy_rp::spi::Blocking>,
            Output<peripherals::PIN_17>,
            Output<peripherals::PIN_20>,
            Input<peripherals::PIN_26>,
            Output<peripherals::PIN_21>,
        >,
    >,
> = None;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, 2048) }

    let p = embassy_rp::init(Default::default());

    let spi = Spi::new_blocking(
        p.SPI0,
        p.PIN_18,
        p.PIN_19,
        p.PIN_16,
        embassy_rp::spi::Config::default(),
    );
    let cs = Output::new(p.PIN_17, Level::Low);
    let dc = Output::new(p.PIN_20, Level::Low);
    let busy = Input::new(p.PIN_26, Pull::Up);
    let reset = Output::new(p.PIN_21, Level::Low);

    let mut display = Uc8151::new(spi, cs, dc, busy, reset);
    display
        .setup(&mut embassy_time::Delay, uc8151::LUT::Fast)
        .unwrap();

    display.enable();
    display.update().expect("Failed to update");

    let mut sysbadge = Sysbadge::new(display);

    sysbadge.draw().expect("failed to draw display");

    info!("updating display");
    sysbadge.display.update().expect("failed to update");

    unsafe {
        SYSBADGE = Some(sysbadge);
    }

    spawner.spawn(button_task_a()).unwrap();
    spawner.spawn(button_task_b()).unwrap();
    spawner.spawn(button_task_c()).unwrap();
    spawner.spawn(button_task_up()).unwrap();
    spawner.spawn(button_task_down()).unwrap();
}

const DELAY: u64 = 150;
#[embassy_executor::task]
async fn button_task_a() {
    let mut pin = Input::new(unsafe { peripherals::PIN_12::steal() }, Pull::Down);
    loop {
        pin.wait_for_high().await;
        press_button(Button::A).await;
        Timer::after(Duration::from_millis(DELAY)).await;
    }
}

#[embassy_executor::task]
async fn button_task_b() {
    let mut pin = Input::new(unsafe { peripherals::PIN_13::steal() }, Pull::Down);
    loop {
        pin.wait_for_high().await;
        press_button(Button::B).await;
        Timer::after(Duration::from_millis(DELAY)).await;
    }
}

#[embassy_executor::task]
async fn button_task_c() {
    let mut pin = Input::new(unsafe { peripherals::PIN_14::steal() }, Pull::Down);
    loop {
        pin.wait_for_high().await;
        press_button(Button::C).await;
        Timer::after(Duration::from_millis(DELAY)).await;
    }
}

#[embassy_executor::task]
async fn button_task_up() {
    let mut pin = Input::new(unsafe { peripherals::PIN_15::steal() }, Pull::Down);
    loop {
        pin.wait_for_high().await;
        press_button(Button::Up).await;
        Timer::after(Duration::from_millis(DELAY)).await;
    }
}

#[embassy_executor::task]
async fn button_task_down() {
    let mut pin = Input::new(unsafe { peripherals::PIN_11::steal() }, Pull::Down);
    loop {
        pin.wait_for_high().await;
        press_button(Button::Down).await;
        Timer::after(Duration::from_millis(DELAY)).await;
    }
}

async fn press_button(button: Button) {
    unsafe {
        if let Some(sysbadge) = &mut SYSBADGE {
            sysbadge.press(button);
            if sysbadge.draw().expect("failed to draw display") {
                sysbadge.display.update().expect("failed to update");
            }
        }
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    defmt::panic!("allocation error: {:?}", layout);
}
