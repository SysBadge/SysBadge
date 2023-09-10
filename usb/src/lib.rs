use log::info;
use rusb::{
    constants, Context, Device, DeviceDescriptor, DeviceHandle, Hotplug, HotplugBuilder,
    Registration, UsbContext,
};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

pub use rusb;
use rusb::ffi::libusb_version;

mod c;
pub mod err;

pub use err::{Error, Result};
use sysbadge::system::Member;
use sysbadge::usb::{BootSel, VersionType};
use sysbadge::{badge::CurrentMenu, System};

pub const VID: u16 = sysbadge::usb::VID;
pub const PID: u16 = sysbadge::usb::PID;

pub struct UsbSysbadge<T: UsbContext> {
    context: T,
    handle: DeviceHandle<T>,
    timeout: std::time::Duration,
}

impl<T: UsbContext> UsbSysbadge<T> {
    #[export_name = "sysbadge_open"]
    pub extern "C" fn open(mut handle: DeviceHandle<T>) -> Result<Self> {
        let _ = handle.set_auto_detach_kernel_driver(true);
        handle.set_active_configuration(0)?;

        Ok(Self {
            context: handle.context().clone(),
            handle,
            timeout: std::time::Duration::from_secs(1),
        })
    }

    pub extern "C" fn find(mut context: T) -> Result<Self> {
        let (device, descriptor, mut handle) =
            Self::open_device(&mut context, sysbadge::usb::VID, sysbadge::usb::PID)?
                .ok_or(Error::NoDevice)?;

        Self::open(handle)
    }

    pub extern "C" fn press(&self, button: sysbadge::Button) -> Result {
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

    pub fn system_name(&self) -> Result<String> {
        let mut buf = [0; 64];
        let n = self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetSystemName as u8,
            0,
            0,
            &mut buf,
            self.timeout,
        )?;

        Ok(String::from_utf8((&buf[..n]).to_vec())?)
    }

    pub fn member_count(&self) -> Result<u16> {
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

    pub fn member_name(&self, index: u16) -> Result<String> {
        let mut buf = [0; 64];
        let n = self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetMemberName as u8,
            index,
            0,
            &mut buf,
            self.timeout,
        )?;

        Ok(String::from_utf8((&buf[..n]).to_vec())?)
    }

    pub fn member_pronouns(&self, index: u16) -> Result<String> {
        let mut buf = [0; 64];
        let n = self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetMemberPronouns as u8,
            index,
            0,
            &mut buf,
            self.timeout,
        )?;

        Ok(String::from_utf8((&buf[..n]).to_vec())?)
    }

    pub fn get_state(&self) -> Result<CurrentMenu> {
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

    pub fn set_state(&self, state: &CurrentMenu) -> Result {
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

    pub fn update_display(&self) -> Result {
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

    pub fn get_version(&self, version: VersionType) -> Result<[u8; 64]> {
        let mut buf = [0; 64];
        let n = self.handle.read_control(
            constants::LIBUSB_ENDPOINT_IN
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::GetVersion as u8,
            version as u16,
            0,
            &mut buf,
            self.timeout,
        )?;
        Ok(buf)
    }

    pub fn get_version_string(&self, version: VersionType) -> Result<String> {
        let buf = self.get_version(version)?;
        Ok(String::from_utf8(buf.to_vec())?)
    }

    pub fn get_unique_id(&self) -> Result<u64> {
        let buf = self.get_version(VersionType::UniqueId)?;
        let id = u64::from_le_bytes(buf[..8].as_ref().try_into().unwrap());
        Ok(id)
    }

    pub fn reboot(&self, bootsel: BootSel) -> Result<()> {
        self.handle.write_control(
            constants::LIBUSB_ENDPOINT_OUT
                | constants::LIBUSB_REQUEST_TYPE_VENDOR
                | constants::LIBUSB_RECIPIENT_INTERFACE,
            sysbadge::usb::Request::Reboot as u8,
            bootsel as u16,
            0,
            &[0; 0],
            self.timeout,
        )?;
        Ok(())
    }

    pub fn handle(&self) -> &DeviceHandle<T> {
        &self.handle
    }

    pub fn handle_mut(&mut self) -> &mut DeviceHandle<T> {
        &mut self.handle
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

impl<T: UsbContext> System for UsbSysbadge<T> {
    fn name(&self) -> Cow<'_, str> {
        Cow::Owned(self.system_name().unwrap_or_else(|_| "Unknown".to_string()))
    }

    fn member_count(&self) -> usize {
        self.member_count().unwrap_or(0) as usize
    }

    fn member(&self, index: usize) -> &dyn Member {
        todo!()
    }
}
