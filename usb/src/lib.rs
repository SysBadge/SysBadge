#![feature(return_position_impl_trait_in_trait, result_flattening)]
#![feature(iter_array_chunks)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::time::Duration;

pub use rusb;
use rusb::UsbContext;

pub mod err;
pub use err::{Error, Result};
use sysbadge::badge::CurrentMenu;
use sysbadge::system::Member;
use sysbadge::usb::{BootSel, SystemId, SystemIdType, SystemUpdateStatus, VersionType};
use sysbadge::System;
use tracing::*;

use crate::raw::UsbSysBadgeRaw;

pub mod raw;

pub const VID: u16 = sysbadge::usb::VID;
pub const PID: u16 = sysbadge::usb::PID;

/// USB connected SysBadge.
pub struct UsbSysBadge<T: UsbContext> {
    inner: UsbSysBadgeRaw<T>,
}

impl<T: UsbContext> UsbSysBadge<T> {
    pub fn find_badges(context: T) -> Result<Vec<Self>> {
        UsbSysBadgeRaw::find_devices(context.clone())
            .map(|v| v.into_iter().map(|v| v.into()).collect())
    }

    pub fn find_badge(context: T) -> Result<Self> {
        Self::find_badges(context)
            .map(|mut v| v.pop().ok_or(Error::NoDevice))
            .flatten()
    }

    /// Issues a press button command to the SysBadge.
    #[inline(always)]
    pub fn button_press(&self, button: sysbadge::Button) -> Result<()> {
        self.inner.button_press(button)
    }

    /// Get the system name currently active on the SysBadge.
    #[inline(always)]
    pub fn system_name(&self) -> Result<String> {
        self.inner.system_name()
    }

    /// Get the ID of the provider of the current system on the SysBadge.
    #[inline]
    pub fn system_id(&self) -> Result<SystemId> {
        let (id, str) = self.inner.system_id()?;
        Ok(SystemId::new(id, str))
    }

    /// Get the member count of the currently loaded system on the SysBadge.
    #[inline(always)]
    pub fn system_member_count(&self) -> Result<u16> {
        self.inner.system_member_count()
    }

    /// Get the member name of a member of the currently loaded system on the SysBadge.
    #[inline(always)]
    pub fn system_member_name(&self, index: u16) -> Result<String> {
        self.inner.system_member_name(index)
    }

    /// Get the member pronouns of a member of the currently loaded system on the SysBadge.
    #[inline(always)]
    pub fn system_member_pronouns(&self, index: u16) -> Result<String> {
        self.inner.system_member_pronouns(index)
    }

    /// Get the current display state of the SysBadge.
    #[inline(always)]
    pub fn display_state(&self) -> Result<CurrentMenu> {
        self.inner.get_state()
    }

    /// Sets the current display state of the SysBadge.
    #[inline(always)]
    pub fn set_display_state(&self, state: &CurrentMenu) -> Result<()> {
        self.inner.set_state(&state)
    }

    /// Request to update the display of the SysBadge.
    #[inline(always)]
    pub fn update_display(&self) -> Result<()> {
        self.inner.update_display()
    }

    /// Read the current SemVer version of the SysBadge.
    pub fn semver_version(&self) -> Result<String> {
        let mut buf = [0; 64];
        let n = self.inner.read_version(VersionType::SemVer, &mut buf)?;
        Ok(String::from_utf8((&buf[..n]).to_vec())?)
    }

    /// Reboot the Sysbadge.
    #[inline(always)]
    pub fn reboot(self) -> Result {
        self.inner.reboot(BootSel::Application)
    }

    pub fn system_update_blocking(
        &self,
        erase: bool,
        data: impl Iterator<Item = u8>,
    ) -> Result<usize> {
        self.inner.system_prepare_update(erase)?;
        let status = self.system_update_wait_status_blocking()?;
        debug!("system update status: {:?}", status);

        let mut data = data.array_chunks::<64>();
        let mut offset = 0;
        loop {
            if let Some(data) = data.next() {
                self.inner.system_write_chunk(offset, &data)?;
                offset += 64;
                let status = self.system_update_wait_status_blocking()?;
                debug!("system update status: {:?}", status);
            } else {
                break;
            }
        }
        if let Some(reminder) = data.into_remainder() {
            if (reminder.len() % 4) != 0 {
                return Err(Error::Unaligned);
            }
            let data: Vec<u8> = reminder.collect();
            self.inner.system_write_chunk(offset, &data)?;
            let status = self.system_update_wait_status_blocking()?;
            debug!("system update status: {:?}", status);
        }
        self.inner.system_write_chunk(0, &[])?;
        let status = self.system_update_wait_status_blocking()?;
        debug!("system update status: {:?}", status);

        Ok(offset as usize)
    }

    // TODO: Reboot bootloader (maybe also via DFU)

    /// Set the timeout for USB transfers.
    #[inline(always)]
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.inner.set_timeout(timeout)
    }

    /// Return a reference to the inner `rusb::DeviceHandle`.
    #[inline(always)]
    pub fn handle(&self) -> &rusb::DeviceHandle<T> {
        self.inner.handle()
    }

    /// Return a mutable reference to the inner `rusb::DeviceHandle`.
    #[inline(always)]
    pub fn handle_mut(&mut self) -> &mut rusb::DeviceHandle<T> {
        self.inner.handle_mut()
    }

    /// Wait untill a success status is returned from the SysBadge.
    pub fn system_update_wait_status_blocking(&self) -> Result<SystemUpdateStatus> {
        let mut duration = Duration::from_millis(100);
        loop {
            let status = self.inner.system_update_status()?;
            trace!(status = ?status, timeout = ?duration, "Waiting for system update status");
            match status {
                SystemUpdateStatus::Writing => {
                    std::thread::sleep(duration);
                },
                v => return Ok(v),
            }
            duration *= 2;
        }
    }
}

impl<T: UsbContext> From<UsbSysBadgeRaw<T>> for UsbSysBadge<T> {
    fn from(inner: UsbSysBadgeRaw<T>) -> Self {
        Self { inner }
    }
}

impl<T: UsbContext> From<UsbSysBadge<T>> for UsbSysBadgeRaw<T> {
    fn from(badge: UsbSysBadge<T>) -> Self {
        badge.inner
    }
}

impl<T: UsbContext> System for UsbSysBadge<T> {
    #[inline]
    fn name(&self) -> String {
        self.system_name().unwrap()
    }

    #[inline]
    fn member_count(&self) -> usize {
        self.system_member_count().unwrap() as usize
    }

    #[inline]
    fn member(&self, index: usize) -> UsbSysBadgeMember<T> {
        let count = self.member_count();
        if index >= count {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                count, index
            );
        }

        UsbSysBadgeMember {
            badge: self,
            index: index as u16,
        }
    }
}

/// Reference to a member of the currently loaded system on the SysBadge.
pub struct UsbSysBadgeMember<'u, T: UsbContext + 'u> {
    badge: &'u UsbSysBadge<T>,
    index: u16,
}

impl<'u, T: UsbContext + 'u> Member for UsbSysBadgeMember<'u, T> {
    #[inline]
    fn name(&self) -> String {
        self.badge.system_member_name(self.index).unwrap()
    }

    #[inline]
    fn pronouns(&self) -> String {
        self.badge.system_member_pronouns(self.index).unwrap()
    }
}
