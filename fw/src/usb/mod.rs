use ::sysbadge::usb as sysusb;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::usb::Driver;
use embassy_usb::{Builder, Config};
use static_cell::make_static;

pub mod sysbadge;

embassy_nrf::bind_interrupts!(struct Irqs {
    USBD => embassy_nrf::usb::InterruptHandler<embassy_nrf::peripherals::USBD>;
});

#[embassy_executor::task]
pub async fn init(vbus_detect: &'static embassy_nrf::usb::vbus_detect::SoftwareVbusDetect) {
    let driver = Driver::new(
        unsafe { embassy_nrf::peripherals::USBD::steal() },
        Irqs,
        vbus_detect,
    );

    let mut config = Config::new(sysusb::VID, sysusb::PID);
    config.manufacturer = Some("SysBadge");
    config.product = Some("SysBadge");
    config.serial_number = None; // FIXME
    config.max_power = 250; // FIXME
    config.max_packet_size_0 = 64;

    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    let mut builder = Builder::new(
        driver,
        config,
        make_static!([0u8; 256]),
        make_static!([0u8; 256]),
        make_static!([0u8; 256]),
        make_static!([0u8; 64]),
    );

    let sysbadge_class = sysbadge::SysBadgeClass::new(&mut builder);

    let mut usb = builder.build();

    embassy_futures::join::join(sysbadge_class.run(), usb.run()).await;
}
