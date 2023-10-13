use std::alloc::System;
use std::time::Duration;

use rusb::{Device, DeviceDescriptor, DeviceHandle, UsbContext};
use sysbadge::badge::CurrentMenu;
pub use sysbadge::usb as types;
use sysbadge::usb::{SystemIdType, SystemUpdateStatus};
use tracing::*;
use types::{BootSel, Request, VersionType};

use crate::{Error, Result};

pub struct UsbSysBadgeRaw<T: UsbContext> {
    context: T,
    handle: DeviceHandle<T>,
    timeout: Duration,
    com_index: u16,
}

impl<T: UsbContext> UsbSysBadgeRaw<T> {
    /// Open a device handle as a raw SysBadge.
    pub fn open(mut handle: DeviceHandle<T>) -> Result<Self> {
        debug!(
            "Opening SysBadge at: {:?}",
            handle.device().device_descriptor()
        );
        let _ = handle.set_auto_detach_kernel_driver(true);
        handle.set_active_configuration(0)?;

        Ok(Self {
            context: handle.context().clone(),
            handle,
            timeout: Duration::from_secs(1),
            com_index: 0,
        })
    }

    pub fn find_devices(mut context: T) -> Result<Vec<Self>> {
        let devices = match context.devices() {
            Ok(d) => d,
            Err(_) => return Ok(Vec::new()),
        };

        let mut ret = Vec::new();

        for device in devices.iter() {
            let device_desc = match device.device_descriptor() {
                Ok(d) => d,
                Err(_) => continue,
            };

            if device_desc.vendor_id() == sysbadge::usb::VID
                && device_desc.product_id() == sysbadge::usb::PID
            {
                trace!("Found USB device: {:?}", device_desc);
                let handler = device.open()?;
                ret.push(Self::open(handler)?);
            }
        }
        Ok(ret)
    }

    /// Write a control message.
    ///
    /// This is a wrapper around `DeviceHandle::write_control`.
    #[inline]
    pub fn write_control(
        &self,
        request: sysbadge::usb::Request,
        value: u16,
        buf: &[u8],
        timeout: Option<Duration>,
    ) -> Result<usize> {
        debug!(request = ?request, value, buf = buf.len(), "Write controll message");
        Ok(self.handle.write_control(
            rusb::request_type(
                rusb::Direction::Out,
                rusb::RequestType::Vendor,
                rusb::Recipient::Interface,
            ),
            request as u8,
            value,
            self.com_index,
            buf,
            timeout.unwrap_or(self.timeout),
        )?)
    }

    /// Read a control message.
    ///
    /// This is a wrapper around `DeviceHandle::read_control`.
    #[inline]
    pub fn read_control(
        &self,
        request: sysbadge::usb::Request,
        value: u16,
        buf: &mut [u8],
        timeout: Option<Duration>,
    ) -> Result<usize> {
        Ok(self.handle.read_control(
            rusb::request_type(
                rusb::Direction::In,
                rusb::RequestType::Vendor,
                rusb::Recipient::Interface,
            ),
            request as u8,
            value,
            self.com_index,
            buf,
            timeout.unwrap_or(self.timeout),
        )?)
    }

    /// Issues a press button command to the Badge.
    #[inline]
    pub fn button_press(&self, button: sysbadge::Button) -> Result {
        self.write_control(
            sysbadge::usb::Request::ButtonPress,
            button as u16,
            &[0; 0],
            None,
        )?;

        Ok(())
    }

    /// Reads the system name currently active on the SysBadge.
    pub fn system_name(&self) -> Result<String> {
        let mut buf = [0; 64];
        let n = self.read_control(sysbadge::usb::Request::GetSystemName, 0, &mut buf, None)?;

        Ok(String::from_utf8((&buf[..n]).to_vec())?)
    }

    /// Read the system ID information
    pub fn system_id(&self) -> Result<(SystemIdType, String)> {
        let mut buf = [0; 64];
        let n = self.read_control(sysbadge::usb::Request::GetSystemName, 1, &mut buf, None)?;
        assert!(n > 1, "GetSystemId returned {} bytes", n);

        let id = SystemIdType::try_from(buf[0]).map_err(|err| Error::IntEnumError(err.value()))?;
        let str = String::from_utf8((&buf[1..n]).to_vec())?;

        Ok((id, str))
    }

    /// Get the member count of the currently loaded system on the SysBadge.
    pub fn system_member_count(&self) -> Result<u16> {
        let mut buf = [0; 2];
        let n = self.read_control(sysbadge::usb::Request::GetMemberCount, 0, &mut buf, None)?;
        assert_eq!(n, 2, "GetMemberCount returned {} bytes", n);

        let count = u16::from_le_bytes(buf);
        Ok(count)
    }

    /// Reads the name of a member of the currently loaded system on the SysBadge.
    pub fn system_member_name(&self, index: u16) -> Result<String> {
        let mut buf = [0; 64];
        let n = self.read_control(sysbadge::usb::Request::GetMemberName, index, &mut buf, None)?;

        Ok(String::from_utf8((&buf[..n]).to_vec())?)
    }

    /// Reads the pronouns of a member of the currently loaded system on the SysBadge.
    pub fn system_member_pronouns(&self, index: u16) -> Result<String> {
        let mut buf = [0; 64];
        let n = self.read_control(
            sysbadge::usb::Request::GetMemberPronouns,
            index,
            &mut buf,
            None,
        )?;

        Ok(String::from_utf8((&buf[..n]).to_vec())?)
    }

    /// Get the current display state of the SysBadge.
    pub fn get_state(&self) -> Result<CurrentMenu> {
        let mut buf = [0; core::mem::size_of::<CurrentMenu>()];
        let n = self.read_control(sysbadge::usb::Request::GetState, 0, &mut buf, None)?;

        Ok(CurrentMenu::from_bytes(&buf))
    }

    /// Set the current display state of the SysBadge.
    pub fn set_state(&self, state: &CurrentMenu) -> Result {
        let buf = state.as_bytes();
        let n = self.write_control(sysbadge::usb::Request::SetState, 0, buf, None)?;

        Ok(())
    }

    /// Trigger an update display request.
    #[inline]
    pub fn update_display(&self) -> Result {
        self.write_control(sysbadge::usb::Request::UpdateDisplay, 0, &[0; 0], None)?;

        Ok(())
    }

    /// Reads the version of the SysBadge.
    pub fn read_version(&self, version: VersionType, buf: &mut [u8]) -> Result<usize> {
        let n = self.read_control(
            sysbadge::usb::Request::GetVersion,
            version as u16,
            buf,
            None,
        )?;
        Ok(n)
    }

    /// Reboot the SysBadge into the given BootSel mode.
    pub fn reboot(&self, bootsel: BootSel) -> Result<()> {
        self.write_control(
            sysbadge::usb::Request::Reboot,
            bootsel as u16,
            &[0; 0],
            None,
        )?;
        Ok(())
    }

    /// Enter the system update mode.
    pub fn system_prepare_update(&self, erase: bool) -> Result {
        self.write_control(
            sysbadge::usb::Request::SystemUpload,
            if erase { 1 } else { 0 },
            &[0; 0],
            None,
        )?;
        Ok(())
    }

    /// Reads the current status of the system update mode.
    pub fn system_update_status(&self) -> Result<SystemUpdateStatus> {
        let mut buf = [0];
        let n = self.read_control(Request::SystemUpload, 0, &mut buf, None)?;
        assert_eq!(n, 1);

        Ok(SystemUpdateStatus::try_from(buf[0]).map_err(|err| Error::IntEnumError(err.value()))?)
    }

    /// Write a system update chunk.
    pub fn system_write_chunk(&self, offset: u16, chunk: &[u8]) -> Result<usize> {
        if chunk.len() > 64 || (chunk.len() % 4) != 0 || (offset % 4) != 0 {
            return Err(Error::Unaligned);
        }

        Ok(self.write_control(sysbadge::usb::Request::SystemDNLoad, offset, chunk, None)?)
    }

    /*pub fn erase_and_update_system(&self, erase: bool, data: &[u8]) -> Result<()> {
        self.enter_update_system(erase)?;

        self.update_system(data)
    }

    pub fn update_system(&self, data: &[u8]) -> Result<()> {
        let data = data.chunks(64);
        for (i, chunk) in data.enumerate() {
            self.write_system((i * 64) as u16, chunk)?;
            std::thread::sleep(std::time::Duration::from_millis(2050));
        }
        self.write_system(0, &[])?;
        Ok(())
    }*/

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
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
