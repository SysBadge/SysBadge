use rusb::Recipient::Endpoint;

fn main() {
    println!("Hello, world!");
    let mut handle = rusb::open_device_with_vid_pid(0xc0de, 0xcafe).unwrap();
    handle.set_auto_detach_kernel_driver(true).unwrap();

    let mut device = handle.device();
    let desc = device.device_descriptor().unwrap();
    println!("Device descriptor: {:?}", desc);

    let timeout = std::time::Duration::from_secs(1);
    let languages = handle.read_languages(timeout).unwrap();

    println!(
        "Active configuration: {}",
        handle.active_configuration().unwrap()
    );
    println!("Languages: {:?}", languages);

    if !languages.is_empty() {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            handle
                .read_manufacturer_string(language, &desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            handle.read_product_string(language, &desc, timeout).ok()
        );
        println!(
            "Serial Number: {:?}",
            handle
                .read_serial_number_string(language, &desc, timeout)
                .ok()
        );

        println!("Configurations: {:?}", desc.num_configurations());

        let desc = device.config_descriptor(0).unwrap();
        println!(
            "Configuration: {:?}",
            handle
                .read_configuration_string(language, &desc, timeout)
                .ok()
        );
    }

    handle.set_active_configuration(1).unwrap();
    let mut buf = [0; 2];
    //handle.write_bulk(0, &[0x01, 0x02, 0x03], timeout).unwrap();
    //handle.read_control(0x80 | rusb::constants::LIBUSB_REQUEST_TYPE_VENDOR | rusb::constants::LIBUSB_RECIPIENT_INTERFACE, sysbadge::usb::Request::GetMemberCount as u8, 0x00, 0, &mut buf, timeout).unwrap();
    handle
        .write_control(
            rusb::constants::LIBUSB_ENDPOINT_OUT
                | rusb::constants::LIBUSB_REQUEST_TYPE_VENDOR
                | rusb::constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::ButtonPress as u8,
            sysbadge::Button::B as u16,
            0,
            &buf,
            timeout,
        )
        .unwrap();
    println!("Read: {:?}", buf);
    //handle.write_control(0x41, 0x33, 1, 0, &[1], timeout).unwrap();
    // mRequestType=0x41, bRequest=0x33, \
    //         wValue=int(led_on), wIndex=0

    //let device = device.device();
}
