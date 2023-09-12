mod class;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::{Builder, Config};
use static_cell::make_static;

use sysbadge::usb as sysusb;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<peripherals::USB>;
});

#[embassy_executor::task]
pub async fn init(
    spawner: Spawner,
    badge: &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>,
    flash: &'static crate::RpFlashMutex<'static>,
) {
    let driver = Driver::new(unsafe { embassy_rp::peripherals::USB::steal() }, Irqs);

    let serial = {
        let mut buf = [0; 8];
        let mut flash = flash.lock().await;
        unwrap!(flash.blocking_unique_id(&mut buf));
        let mut out = [0; 16];
        unwrap!(
            hex::encode_to_slice(&buf, &mut out),
            "Failed to encode serial"
        );
        out
    };
    let serial = make_static!(serial);
    let serial: &'static str = unsafe { core::str::from_utf8_unchecked(serial) };

    // Create config
    let mut config = Config::new(sysusb::VID, sysusb::PID);
    config.manufacturer = Some("nyantec GmbH");
    config.product = Some("Sysbadge");
    config.serial_number = Some(serial);
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    let mut builder = Builder::new(
        driver,
        config,
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 128])[..],
    );

    let _class = class::SysbadgeClass::new(
        &mut builder,
        make_static!(class::State::new(badge, flash)),
        64,
    );

    let usb = builder.build();
    unwrap!(spawner.spawn(usb_task(usb)));

    /*let task_fut = async {
        /*loop {
            class.wait_connection().await;
            info!("Connected");
            class.write_packet(b"H").await.unwrap();
            let _ = echo(&mut class).await;
            info!("Disconnected");
        }*/
    };

    embassy_futures::join::join(usb_fut, task_fut).await;*/
}

#[embassy_executor::task]
async fn usb_task(
    mut device: embassy_usb::UsbDevice<'static, Driver<'static, peripherals::USB>>,
) -> ! {
    device.run().await
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
