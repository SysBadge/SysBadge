#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod usb;

use defmt::*;
use embassy_time::{Duration, Timer};

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::flash::Flash;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::CORE1;
use embassy_rp::spi::Spi;
use embassy_rp::{peripherals, Peripherals};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use panic_probe as _;

use uc8151::Uc8151;

use sysbadge::badge::Sysbadge;
use sysbadge::system::SystemReader;
use sysbadge::Button;

pub enum UsbControl {
    GetMemberCount,
}
pub enum UsbResponse {
    MemberCount(u16),
}

static mut CORE1_STACK: embassy_rp::multicore::Stack<4096> = embassy_rp::multicore::Stack::new();
static EXECUTOR0: static_cell::StaticCell<embassy_executor::Executor> =
    static_cell::StaticCell::new();
static EXECUTOR1: static_cell::StaticCell<embassy_executor::Executor> =
    static_cell::StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, Button, 1> = Channel::new();

static BADGE: static_cell::StaticCell<Mutex<CriticalSectionRawMutex, SysbadgeUc8151>> =
    static_cell::StaticCell::new();

const FLASH_SIZE: usize = 2 * 1024 * 1024;
pub type RpFlash<'a> = Flash<'a, peripherals::FLASH, embassy_rp::flash::Blocking, FLASH_SIZE>;
static FLASH: static_cell::StaticCell<Mutex<CriticalSectionRawMutex, RpFlash<'static>>> =
    static_cell::StaticCell::new();
pub type RpFlashMutex<'a> = Mutex<CriticalSectionRawMutex, RpFlash<'a>>;

type SysbadgeUc8151<'a> = Sysbadge<
    Uc8151<
        Spi<'a, peripherals::SPI0, embassy_rp::spi::Blocking>,
        Output<'a, peripherals::PIN_17>,
        Output<'a, peripherals::PIN_20>,
        Input<'a, peripherals::PIN_26>,
        Output<'a, peripherals::PIN_21>,
    >,
    SystemReader<sysbadge::system::capnp::serialize::NoAllocSliceSegments<'a>>,
>;

const SERIAL_LEN: usize = 16;
static mut SERIAL: [u8; SERIAL_LEN] = [0; 16];

pub async fn get_serial_str(flash: &RpFlashMutex<'_>) -> &'static str {
    unsafe { core::str::from_utf8_unchecked(get_serial(flash).await) }
}
pub async fn get_serial(flash: &RpFlashMutex<'_>) -> &'static [u8] {
    if u64::from_le_bytes(unsafe { SERIAL[0..8].try_into().unwrap() }) == 0 {
        let mut buf = [0; SERIAL_LEN.div_ceil(2)];
        let mut flash = flash.lock().await;
        defmt::unwrap!(flash.blocking_unique_id(&mut buf));
        let mut out = [0; SERIAL_LEN];
        defmt::unwrap!(
            hex::encode_to_slice(&buf, &mut out),
            "Failed to encode serial"
        );
        defmt::info!("serial: {:?}", out);
        unsafe { SERIAL = out };
    }

    unsafe { &SERIAL }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let _enable_pmic = Output::new(unsafe { peripherals::PIN_10::steal() }, Level::High);

    for _ in 0..20 {
        unsafe { core::arch::asm!("nop") }
    }

    let badge = init(p);
    let badge = BADGE.init(Mutex::new(badge));
    let badge: &Mutex<_, _> = badge;

    embassy_rp::multicore::spawn_core1(
        unsafe { CORE1::steal() },
        unsafe { &mut CORE1_STACK },
        move || {
            let executor1 = EXECUTOR1.init(embassy_executor::Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_init(spawner, badge))));
        },
    );

    let executor0 = EXECUTOR0.init(embassy_executor::Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_init(spawner, badge))));
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

    for _ in 0..10 {
        unsafe { core::arch::asm!("nop") }
    }

    display.enable();
    unwrap!(
        display.setup(&mut embassy_time::Delay, uc8151::LUT::Fast),
        "Failed to setup display"
    );

    let system = unsafe { sysbadge::system::SystemReader::from_linker_symbols() };
    let mut sysbadge = Sysbadge::new(display, system);

    info!("updating display");
    unwrap!(sysbadge.draw(), "Failed to draw display");
    unwrap!(sysbadge.display.update(), "Failed to update display");

    sysbadge
}

#[embassy_executor::task]
async fn core0_init(
    spawner: Spawner,
    badge: &'static Mutex<CriticalSectionRawMutex, SysbadgeUc8151<'static>>,
) {
    info!("Starting tasks on core 0");

    // add some delay to give an attached debug probe time to parse the
    // defmt RTT header. Reading that header might touch flash memory, which
    // interferes with flash write operations.
    // https://github.com/knurling-rs/defmt/pull/683
    Timer::after(Duration::from_millis(10)).await;

    let flash = Flash::new_blocking(unsafe { peripherals::FLASH::steal() });
    let flash = FLASH.init(Mutex::new(flash));

    // adding serial number
    {
        let serial = get_serial_str(&flash).await;
        badge.lock().await.serial = Some(serial);
    }

    spawner.spawn(button_task_a()).unwrap();
    spawner.spawn(button_task_b()).unwrap();
    spawner.spawn(button_task_c()).unwrap();
    spawner.spawn(button_task_up()).unwrap();
    spawner.spawn(button_task_down()).unwrap();
    spawner.spawn(usb::init(spawner, badge, flash)).unwrap();
}

#[embassy_executor::task]
async fn core1_init(
    spawner: Spawner,
    badge: &'static Mutex<CriticalSectionRawMutex, SysbadgeUc8151<'static>>,
) {
    info!("Starting tasks on core 1");

    spawner.spawn(update_redraw_timer_task(badge)).unwrap();
}

const UPDATE_REDRAW_TIMER_TASK_DELAY: u64 = 500;
#[embassy_executor::task]
async fn update_redraw_timer_task(
    badge: &'static Mutex<CriticalSectionRawMutex, SysbadgeUc8151<'static>>,
) {
    'outer: loop {
        let button = CHANNEL.receive().await;
        //let badge = unsafe { unwrap!(SYSBADGE.as_mut()) };
        {
            let mut badge = badge.lock().await;
            badge.press(button);
        }
        loop {
            let ret = select(
                CHANNEL.receive(),
                Timer::after(Duration::from_millis(UPDATE_REDRAW_TIMER_TASK_DELAY)),
            )
            .await;
            match ret {
                embassy_futures::select::Either::First(btn) => {
                    let mut badge = badge.lock().await;
                    badge.press(btn);
                }
                embassy_futures::select::Either::Second(_) => {
                    let mut badge = badge.lock().await;
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
