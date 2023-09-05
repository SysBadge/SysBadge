#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(allocator_api, alloc_error_handler)]

use alloc_cortex_m::CortexMHeap;
use defmt::*;
use embassy_time::{Duration, Timer};

extern crate alloc;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::CORE1;
use embassy_rp::spi::Spi;
use embassy_rp::{peripherals, Peripherals};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use panic_probe as _;

use uc8151::Uc8151;

use sysbadge::{Button, Sysbadge};

static mut CORE1_STACK: embassy_rp::multicore::Stack<4096> = embassy_rp::multicore::Stack::new();
static EXECUTOR0: static_cell::StaticCell<embassy_executor::Executor> =
    static_cell::StaticCell::new();
static EXECUTOR1: static_cell::StaticCell<embassy_executor::Executor> =
    static_cell::StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, Button, 1> = Channel::new();

type SysbadgeUc8151<'a> = Sysbadge<
    'a,
    Uc8151<
        Spi<'a, peripherals::SPI0, embassy_rp::spi::Blocking>,
        Output<'a, peripherals::PIN_17>,
        Output<'a, peripherals::PIN_20>,
        Input<'a, peripherals::PIN_26>,
        Output<'a, peripherals::PIN_21>,
    >,
>;

#[cortex_m_rt::entry]
fn main() -> ! {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, 2048) }

    let p = embassy_rp::init(Default::default());
    let badge = init(p);

    embassy_rp::multicore::spawn_core1(
        unsafe { CORE1::steal() },
        unsafe { &mut CORE1_STACK },
        move || {
            let executor1 = EXECUTOR1.init(embassy_executor::Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core0_init(spawner))));
        },
    );

    let executor0 = EXECUTOR0.init(embassy_executor::Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core1_init(spawner, badge))));
}

fn init(p: Peripherals) -> SysbadgeUc8151<'static> {
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
    unwrap!(
        display.setup(&mut embassy_time::Delay, uc8151::LUT::Fast),
        "Failed to setup display"
    );

    display.enable();

    let mut sysbadge = Sysbadge::new(display);

    info!("updating display");
    unwrap!(sysbadge.draw(), "Failed to draw display");
    unwrap!(sysbadge.display.update(), "Failed to update display");

    sysbadge
}

#[embassy_executor::task]
async fn core0_init(spawner: Spawner) {
    info!("Starting tasks on core 0");

    spawner.spawn(button_task_a()).unwrap();
    spawner.spawn(button_task_b()).unwrap();
    spawner.spawn(button_task_c()).unwrap();
    spawner.spawn(button_task_up()).unwrap();
    spawner.spawn(button_task_down()).unwrap();
}

#[embassy_executor::task]
async fn core1_init(spawner: Spawner, badge: SysbadgeUc8151<'static>) {
    info!("Starting tasks on core 1");

    spawner.spawn(update_redraw_timer_task(badge)).unwrap();
}

const UPDATE_REDRAW_TIMER_TASK_DELAY: u64 = 500;
#[embassy_executor::task]
async fn update_redraw_timer_task(mut badge: SysbadgeUc8151<'static>) {
    'outer: loop {
        let button = CHANNEL.receive().await;
        //let badge = unsafe { unwrap!(SYSBADGE.as_mut()) };
        badge.press(button);
        loop {
            let ret = select(
                CHANNEL.receive(),
                Timer::after(Duration::from_millis(UPDATE_REDRAW_TIMER_TASK_DELAY)),
            )
            .await;
            match ret {
                embassy_futures::select::Either::First(btn) => {
                    badge.press(btn);
                }
                embassy_futures::select::Either::Second(_) => {
                    unwrap!(badge.draw());
                    unwrap!(badge.display.update(), "Failed to update display");
                    continue 'outer;
                }
            }
        }
    }
}

const DELAY: u64 = 250;
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
    CHANNEL.send(button).await;
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    defmt::panic!("allocation error: {:?}", layout);
}
