use log::info;
use rusb::{
    constants, Device, DeviceDescriptor, DeviceHandle, Hotplug, HotplugBuilder, Registration,
    UsbContext,
};
use std::sync::{Arc, Mutex};

pub use rusb;
pub mod err;

pub use err::{Error, Result};
use sysbadge::CurrentMenu;

pub struct UsbSysbadge<T: UsbContext> {
    context: T,
    handle: DeviceHandle<T>,
    timeout: std::time::Duration,
}

impl<T: UsbContext> UsbSysbadge<T> {
    pub fn open(mut handle: DeviceHandle<T>) -> Result<Self> {
        let _ = handle.set_auto_detach_kernel_driver(true);
        handle.set_active_configuration(0)?;

        Ok(Self {
            context: handle.context().clone(),
            handle,
            timeout: std::time::Duration::from_secs(1),
        })
    }

    pub fn find(mut context: T) -> Result<Self> {
        let (device, descriptor, mut handle) =
            Self::open_device(&mut context, sysbadge::usb::VID, sysbadge::usb::PID)?
                .ok_or(Error::NoDevice)?;
        let _ = handle.set_auto_detach_kernel_driver(true);
        handle.set_active_configuration(0)?;

        Ok(Self {
            context,
            handle,
            timeout: std::time::Duration::from_secs(1),
        })
    }

    pub fn press(&mut self, button: sysbadge::Button) -> Result {
        self.handle.write_control(
            constants::LIBUSB_ENDPOINT_OUT
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::ButtonPress as u8,
            button as u16,
            0,
            &[0; 0],
            self.timeout,
        )?;

        Ok(())
    }

    pub fn system_name(&mut self) -> Result<String> {
        let mut buf = [0; 64];
        self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetSystemName as u8,
            0,
            0,
            &mut buf,
            self.timeout,
        )?;

        Ok(String::from_utf8(buf.to_vec())?)
    }

    pub fn member_count(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetMemberCount as u8,
            0,
            0,
            &mut buf,
            self.timeout,
        )?;

        let count = u16::from_le_bytes(buf);
        Ok(count)
    }

    pub fn member_name(&mut self, index: u16) -> Result<String> {
        let mut buf = [0; 64];
        self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetMemberName as u8,
            index,
            0,
            &mut buf,
            self.timeout,
        )?;

        Ok(String::from_utf8(buf.to_vec())?)
    }

    pub fn member_pronouns(&mut self, index: u16) -> Result<String> {
        let mut buf = [0; 64];
        self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetMemberPronouns as u8,
            index,
            0,
            &mut buf,
            self.timeout,
        )?;

        Ok(String::from_utf8(buf.to_vec())?)
    }

    pub fn get_state(&mut self) -> Result<CurrentMenu> {
        let mut buf = [0; core::mem::size_of::<CurrentMenu>()];
        self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetState as u8,
            0,
            0,
            &mut buf,
            self.timeout,
        )?;

        Ok(CurrentMenu::from_bytes(&buf))
    }

    pub fn set_state(&mut self, state: &CurrentMenu) -> Result {
        let buf = state.as_bytes();
        self.handle.write_control(
            constants::LIBUSB_ENDPOINT_OUT
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::SetState as u8,
            0,
            0,
            &buf,
            self.timeout,
        )?;

        Ok(())
    }

    pub fn update_display(&mut self) -> Result {
        self.handle.write_control(
            constants::LIBUSB_ENDPOINT_OUT
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::UpdateDisplay as u8,
            0,
            0,
            &[0; 0],
            self.timeout,
        )?;

        Ok(())
    }

    fn open_device(
        context: &mut T,
        vid: u16,
        pid: u16,
    ) -> Result<Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)>> {
        let devices = match context.devices() {
            Ok(d) => d,
            Err(_) => return Ok(None),
        };

        for device in devices.iter() {
            let device_desc = match device.device_descriptor() {
                Ok(d) => d,
                Err(_) => continue,
            };

            if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
                let handler = device.open()?;
                return Ok(Some((device, device_desc, handler)));
            }
        }

        Ok(None)
    }
}
