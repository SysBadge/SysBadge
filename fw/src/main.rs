#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
// global logger
use embassy_nrf::{
    self as _,
    gpio::{Level, Output},
};
use embassy_time::{Delay, Duration, Timer};
use nrf_softdevice::Softdevice;
// time driver
use panic_probe as _;

use core::default::Default;
use defmt::*;
use nrf_softdevice::raw;

mod ble;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = embassy_nrf::config::Config::default();
    let p = embassy_nrf::init(config);

    let pin = p.P0_13;
    let mut pin = Output::new(pin, Level::Low, embassy_nrf::gpio::OutputDrive::Standard);

    loop {
        pin.set_high();
        Timer::after(Duration::from_secs(1)).await;
        pin.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}

/*#[cortex_m_rt::entry]
fn main() -> ! {
    let config = embassy_nrf::config::Config::default();
    let p = embassy_nrf::init(config);

    let pin = p.P0_13;
    let mut pin = Output::new(pin, Level::Low, embassy_nrf::gpio::OutputDrive::Standard);
    pin.set_high();
    core::mem::forget(pin);

    let executor = static_cell::make_static!(embassy_executor::Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_ble(spawner)));
    })
}

#[embassy_executor::task]
async fn main_ble(spawner: Spawner) {
    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t { attr_tab_size: 32768 }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
            central_role_count: 3,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { core::mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    let server = unwrap!(ble::Server::new(sd));
    unwrap!(spawner.spawn(ble::softdevice_task(sd)));

    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x03, 0x03, 0x09, 0x18,
    ];

    static BONDER: static_cell::StaticCell<ble::Bonder> = static_cell::StaticCell::new();
    let bonder = BONDER.init(ble::Bonder::default());

    loop {
        let config = nrf_softdevice::ble::peripheral::Config::default();
        let adv = nrf_softdevice::ble::peripheral::ConnectableAdvertisement::ScannableUndirected { adv_data, scan_data };
        let conn = unwrap!(nrf_softdevice::ble::peripheral::advertise_pairable(sd, adv, &config, bonder).await);

        info!("advertising done!");

        let e = nrf_softdevice::ble::gatt_server::run(&conn, &server, |_| {}).await;

        info!("gatt server done: {:?}", e);
    }
}*/
