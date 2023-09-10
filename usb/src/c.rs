use crate::UsbSysbadge;
use rusb::{Context, Device, Hotplug, HotplugBuilder, UsbContext};
use std::ptr::NonNull;
use std::thread::JoinHandle;

pub struct UsbSysbadgeHotplug {
    pub context: Context,
    pub jhandle: Option<JoinHandle<()>>,
}

impl UsbSysbadgeHotplug {
    pub fn new() -> Self {
        Self {
            context: Context::new().unwrap(),
            jhandle: None,
        }
    }
}

struct UsbSysbadgeHotplugHandler {
    cb_arrived: extern "C" fn(*mut UsbSysbadge<Context>),
    //cb_left: extern "C" fn(*mut UsbSysbadge<Context>),
}

impl Hotplug<rusb::Context> for UsbSysbadgeHotplugHandler {
    fn device_arrived(&mut self, device: Device<Context>) {
        let badge = UsbSysbadge::open(device.open().unwrap()).unwrap();
        let badge = Box::leak(Box::new(badge));
        (self.cb_arrived)(badge);
    }

    fn device_left(&mut self, device: Device<Context>) {
        todo!()
    }
}

#[export_name = "sysbadge_usb_hotplug_new"]
pub extern "C" fn hotplug_new<'a>() -> &'a mut UsbSysbadgeHotplug {
    Box::leak(Box::new(UsbSysbadgeHotplug::new()))
}

#[export_name = "sysbadge_usb_hotplug_start"]
pub extern "C" fn hotplug_start<'a>(
    hotplug: &mut UsbSysbadgeHotplug,
    arrived: extern "C" fn(*mut UsbSysbadge<Context>),
) {
    let context = hotplug.context.clone();
    let reg = HotplugBuilder::new()
        .vendor_id(crate::VID)
        .product_id(crate::PID)
        .enumerate(true)
        .register(
            &context,
            Box::new(UsbSysbadgeHotplugHandler {
                cb_arrived: arrived,
            }),
        );
    let jhandle = std::thread::spawn(move || {
        context.handle_events(None).unwrap();
    });

    hotplug.jhandle = Some(jhandle);
}

#[export_name = "sysbadge_usb_hotplug_free"]
pub extern "C" fn hotplug_free(hotplug: &mut UsbSysbadgeHotplug) {
    let hotplug = unsafe { Box::from_raw(hotplug) };
    // FIXME: stop task
    hotplug.jhandle.map(|h| {});
}
