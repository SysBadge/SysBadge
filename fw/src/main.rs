#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate defmt_rtt as _; // global logger
extern crate embassy_nrf as _; // time driver
extern crate panic_probe as _; // panic handler

use core::default::Default;
use core::mem::MaybeUninit;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output};
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use nrf_softdevice::ble::gatt_server;
use nrf_softdevice::{raw, Flash, Softdevice};
use static_cell::make_static;
use sysbadge::badge::Sysbadge;
use sysbadge::system::SystemReader;

use crate::ble::Server;

mod ble;
mod usb; // TODO: feature

pub(crate) static mut FLASH: MaybeUninit<Mutex<NoopRawMutex, Flash>> = MaybeUninit::uninit();
pub(crate) static mut BADGE: MaybeUninit<Mutex<NoopRawMutex, SysBadge>> = MaybeUninit::uninit();

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    config.lfclk_source = embassy_nrf::config::LfclkSource::ExternalXtal;
    config.gpiote_interrupt_priority = embassy_nrf::interrupt::Priority::P2;
    config.time_interrupt_priority = embassy_nrf::interrupt::Priority::P2;
    let p = embassy_nrf::init(config);

    // FIXME: ??
    let pin = p.P0_13;
    let mut pin = Output::new(pin, Level::Low, embassy_nrf::gpio::OutputDrive::Standard);
    pin.set_high();
    core::mem::forget(pin);

    let executor = make_static!(embassy_executor::Executor::new());
    executor.run(init)
}

//#[embassy_executor::task]
fn init(spawner: Spawner) {
    let sd = enable_softdevice();
    let flash = Flash::take(sd);
    unsafe {
        FLASH.write(Mutex::new(flash));
    }
    trace!("Softdevice and flash enabled");

    let vbus_detect = init_vbus_detect();

    init_badge();

    let server = unwrap!(ble::Server::new(sd));

    unwrap!(spawner.spawn(softdevice_task(sd, vbus_detect)));
    unwrap!(spawner.spawn(main_ble(server, sd)));
    unwrap!(spawner.spawn(usb::init(vbus_detect)));

    info!("init done");
}

#[embassy_executor::task]
async fn main_ble(server: Server, sd: &'static Softdevice) {
    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x03, 0x03, 0x0A, 0x18,
    ];

    loop {
        let config = nrf_softdevice::ble::peripheral::Config::default();
        let adv = nrf_softdevice::ble::peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data,
            scan_data,
        };
        let conn =
            unwrap!(nrf_softdevice::ble::peripheral::advertise_connectable(sd, adv, &config).await);

        info!("advertising done!");

        // Run the GATT server on the connection. This returns when the connection gets disconnected.
        let e = gatt_server::run(&conn, &server, |_| {}).await;

        info!("gatt_server run exited with error: {:?}", e);
    }
    /*#[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x03, 0x03, 0x09, 0x18,
    ];

    //static BONDER: static_cell::StaticCell<ble::Bonder> = static_cell::StaticCell::new();
    //let bonder = BONDER.init(ble::Bonder::default());

    loop {
        let config = nrf_softdevice::ble::peripheral::Config::default();
        let adv = nrf_softdevice::ble::peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data,
            scan_data,
        };
        let conn = unwrap!(
            nrf_softdevice::ble::peripheral::advertise_pairable(sd, adv, &config, bonder).await
        );

        info!("advertising done!");

        let e = nrf_softdevice::ble::gatt_server::run(&conn, &server, |_| {}).await;

        info!("gatt server done: {:?}", e);
    }*/
}

// Softdevice
fn enable_softdevice() -> &'static mut Softdevice {
    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_XTAL as u8,
            rc_ctiv: 0,      //16,
            rc_temp_ctiv: 0, //2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 32768,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { core::mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);

    sd
}

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice, usb_detect: &'static SoftwareVbusDetect) -> ! {
    use nrf_softdevice::SocEvent;
    sd.run_with_callback(|evt| match evt {
        SocEvent::PowerUsbPowerReady => {
            debug!("USB power ready");
            usb_detect.ready()
        },
        SocEvent::PowerUsbDetected => {
            debug!("USB detected");
            usb_detect.detected(true)
        },
        SocEvent::PowerUsbRemoved => {
            debug!("USB removed");
            usb_detect.detected(false)
        },
        v => trace!("sd event {:?}", v),
    })
    .await
}

/// Badge

pub struct DummyDrawTarget;
impl DummyDrawTarget {
    pub fn update(&mut self) -> Result<(), ()> {
        warn!("Add real draw target");
        Ok(())
    }
}
impl embedded_graphics_core::geometry::Dimensions for DummyDrawTarget {
    fn bounding_box(&self) -> embedded_graphics_core::primitives::Rectangle {
        embedded_graphics_core::primitives::Rectangle::new(
            embedded_graphics_core::geometry::Point::new(0, 0),
            embedded_graphics_core::geometry::Size::new(128, 296),
        )
    }
}

impl embedded_graphics_core::draw_target::DrawTarget for DummyDrawTarget {
    type Color = embedded_graphics_core::pixelcolor::BinaryColor;
    type Error = ();

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        warn!("Add real draw target");
        Ok(())
    }
}
pub type SysBadge = sysbadge::badge::Sysbadge<
    SystemReader<sysbadge::system::capnp::serialize::NoAllocSliceSegments<'static>>,
>;

/// Create a new badge instance.
fn init_badge() -> &'static Mutex<NoopRawMutex, SysBadge> {
    let system = unsafe { sysbadge::system::SystemReader::from_linker_symbols() }.ok();
    let sysbadge = Sysbadge::new(system);
    info!("Opend system: {:?}", sysbadge.system.is_some());

    #[cfg(debug_assertions)]
    if sysbadge.system.is_some() {
        use sysbadge::system::System;
        info!(
            "Loaded system from flash with name: {}",
            sysbadge.system.as_ref().unwrap().name()
        );
    }

    unsafe { BADGE.write(Mutex::new(sysbadge)) }
}

/// Create a VBUS detect software instance.
fn init_vbus_detect() -> &'static SoftwareVbusDetect {
    let mut out = 0u32;
    if unsafe { nrf_softdevice::raw::sd_power_usbregstatus_get(&mut out) }
        != nrf_softdevice::raw::NRF_SUCCESS
    {
        warn!("Failed to get USBREGSTATUS");
    }

    #[cfg(debug_assertions)]
    info!(
        "USBREGSTATUS: detect: {}, ready: {}",
        out & 0b01 != 0,
        out & 0b10 != 0
    );

    // enable usb events
    unsafe {
        nrf_softdevice::raw::sd_power_usbdetected_enable(1);
        nrf_softdevice::raw::sd_power_usbpwrrdy_enable(1);
        nrf_softdevice::raw::sd_power_usbremoved_enable(1);
    }

    make_static!(SoftwareVbusDetect::new(out & 0b01 != 0, out & 0b10 != 0))
}
