mod class;
mod web;

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_rp::peripherals;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner, State as NetState};
use embassy_usb::{Builder, Config};
use embedded_io_async::Write;
use static_cell::make_static;

use sysbadge::usb as sysusb;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<peripherals::USB>;
});

const MTU: usize = 1514;

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

    // Our MAC addr.
    let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
    let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

    // Create classes on the builder.
    let class = embassy_usb::class::cdc_ncm::CdcNcmClass::new(
        &mut builder,
        make_static!(embassy_usb::class::cdc_ncm::State::new()),
        host_mac_addr,
        64,
    );

    let usb = builder.build();
    unwrap!(spawner.spawn(usb_task(usb)));

    let (runner, device) =
        class.into_embassy_net_device::<MTU, 4, 4>(make_static!(NetState::new()), our_mac_addr);
    unwrap!(spawner.spawn(usb_ncm_task(runner)));

    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(embassy_net::Ipv4Address::new(169, 254, 0, 61), 16),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });

    // Generate random seed
    let seed = 1234; // guaranteed random, chosen by a fair dice roll

    // Init network stack
    let stack = &*make_static!(Stack::new(
        device,
        config,
        make_static!(StackResources::<2>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));
    unwrap!(spawner.spawn(web::web_server_task(stack, badge, flash)));

    /*loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        info!("Listening on TCP:80...");
        if let Err(e) = socket.accept(80).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());

        let (socket_rx, socket_tx) = socket.split();

        match picoserve::serve_with_state(
            app,
            EmbassyTimer,
            config,
            &mut [0; 2048],
            socket_rx,
            socket_tx,
            &state,
        )
        .await
        {
            Ok(handled_requests_count) => {
                info!(
                    "{} requests handled from {:?}",
                    handled_requests_count,
                    socket.remote_endpoint()
                );
            }
            Err(err) => error!("Failed to server"), //error!("{:?}", err),
        }

        /*let n = match socket.read(&mut buf).await {
            Ok(0) => {
                warn!("read EOF");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                warn!("read error: {:?}", e);
                break;
            }
        };

        info!("rxd {:02x}", &buf[..n]);

        match socket.write_all(&buf[..n]).await {
            Ok(()) => {}
            Err(e) => {
                warn!("write error: {:?}", e);
                break;
            }
        };*/
    }*/
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

#[embassy_executor::task]
async fn usb_ncm_task(class: Runner<'static, Driver<'static, peripherals::USB>, MTU>) -> ! {
    class.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device<'static, MTU>>) -> ! {
    stack.run().await
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
