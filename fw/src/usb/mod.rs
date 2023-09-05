mod class;

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::control::{InResponse, Recipient, Request, RequestType};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Config, Handler};

use sysbadge::usb as sysusb;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<embassy_rp::peripherals::USB>;
});

#[embassy_executor::task]
pub async fn init(
    spawner: Spawner,
    badge: &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>,
) {
    let driver = Driver::new(unsafe { embassy_rp::peripherals::USB::steal() }, Irqs);

    // Create config
    let mut config = Config::new(sysusb::VID, sysusb::PID);
    config.manufacturer = Some("Kloenk");
    config.product = Some("Sysbadge");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = class::State::new(badge);

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
    );

    //let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);
    let mut class = class::SysbadgeClass::new(&mut builder, &mut state, 64);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let tasK_fut = async {
        /*loop {
            class.wait_connection().await;
            info!("Connected");
            class.write_packet(b"H").await.unwrap();
            let _ = echo(&mut class).await;
            info!("Disconnected");
        }*/
    };

    embassy_futures::join::join(usb_fut, tasK_fut).await;
}

/*
struct Disconnected {}

impl From<embassy_usb::driver::EndpointError> for Disconnected {
    fn from(val: embassy_usb::driver::EndpointError) -> Self {
        match val {
            embassy_usb::driver::EndpointError::BufferOverflow => defmt::panic!("Buffer overflow"),
            embassy_usb::driver::EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}*/
