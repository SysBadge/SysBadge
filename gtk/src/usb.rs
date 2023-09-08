use log::info;
use relm4::AsyncComponentSender;
use sysbadge_usb::rusb;
use sysbadge_usb::rusb::{HotplugBuilder, UsbContext};

pub struct Hotplug {
    sender: AsyncComponentSender<crate::App>,
}

pub(crate) fn run(sender: AsyncComponentSender<crate::App>) {
    let context = sysbadge_usb::rusb::Context::new().unwrap();
    let mut builder = HotplugBuilder::new()
        .vendor_id(sysbadge_usb::VID)
        .product_id(sysbadge_usb::PID)
        .enumerate(true)
        .register(context.clone(), Box::new(Hotplug { sender }))
        .unwrap();

    info!("Starting hotplug loop");
    loop {
        context.handle_events(None).unwrap();
    }
}

impl rusb::Hotplug<rusb::Context> for Hotplug {
    fn device_arrived(&mut self, handle: rusb::Device<rusb::Context>) {
        let handle = handle.open().unwrap();
        self.sender.input(crate::Msg::AddBadge(handle));
    }

    fn device_left(&mut self, handle: rusb::Device<rusb::Context>) {
        self.sender.input(crate::Msg::RemoveBadge(handle));
    }
}
