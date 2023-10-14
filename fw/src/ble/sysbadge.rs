use core::sync::atomic::AtomicU16;

use defmt::*;
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{self, CharacteristicHandles, RegisterError};
use nrf_softdevice::ble::{GattError, Uuid};
use nrf_softdevice::{RawError, Softdevice};
use sysbadge::{Member, System};

pub struct SysBadgeGatt {
    name_value: u16,
    member_count_value: u16,
    selected_member: AtomicU16,
    selected_member_value: u16,
    member_name_value: u16,
    member_pronouns_value: u16,
}

impl SysBadgeGatt {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let SYSBADGE_SERVICE: Uuid = Uuid::new_128(&[
            0x96, 0x25, 0x49, 0xd7, 0x0f, 0x59, 0x0a, 0x4c, 0x15, 0x82, 0xa1, 0x43, 0x65, 0xac,
            0xce, 0x59,
        ]);
        let mut sb = ServiceBuilder::new(sd, SYSBADGE_SERVICE)?;

        let SYSTEM_NAME_STRING: Uuid = Uuid::new_128(&[
            0x97, 0x25, 0x49, 0xd7, 0x0f, 0x59, 0x0a, 0x4c, 0x15, 0x82, 0xa1, 0x43, 0x65, 0xac,
            0xce, 0x59,
        ]);
        let SYSTEM_MEMBER_COUNT: Uuid = Uuid::new_128(&[
            0x98, 0x25, 0x49, 0xd7, 0x0f, 0x59, 0x0a, 0x4c, 0x15, 0x82, 0xa1, 0x43, 0x65, 0xac,
            0xce, 0x59,
        ]);
        let SYSTEM_SELECTED_MEMBER: Uuid = Uuid::new_128(&[
            0x99, 0x25, 0x49, 0xd7, 0x0f, 0x59, 0x0a, 0x4c, 0x15, 0x82, 0xa1, 0x43, 0x65, 0xac,
            0xce, 0x59,
        ]);
        let SYSTEM_MEMBER_NAME: Uuid = Uuid::new_128(&[
            0x9A, 0x25, 0x49, 0xd7, 0x0f, 0x59, 0x0a, 0x4c, 0x15, 0x82, 0xa1, 0x43, 0x65, 0xac,
            0xce, 0x59,
        ]);
        let SYSTEM_MEMBER_PRONOUNS: Uuid = Uuid::new_128(&[
            0x9B, 0x25, 0x49, 0xd7, 0x0f, 0x59, 0x0a, 0x4c, 0x15, 0x82, 0xa1, 0x43, 0x65, 0xac,
            0xce, 0x59,
        ]);

        let name_attr = Self::add_str_characteristic(&mut sb, SYSTEM_NAME_STRING, None)?;
        let member_count_attr =
            Self::add_str_characteristic(&mut sb, SYSTEM_MEMBER_COUNT, Some(&[0u8; 2]))?;
        let member_name_attr = Self::add_str_characteristic(&mut sb, SYSTEM_MEMBER_NAME, None)?;
        let member_pronouns_attr =
            Self::add_str_characteristic(&mut sb, SYSTEM_MEMBER_PRONOUNS, None)?;

        let selected_member_attr = {
            let attr = Attribute::new(&[0u8; 2]).deferred_read();
            let md = Metadata::new(Properties::new().read().write());
            sb.add_characteristic(SYSTEM_SELECTED_MEMBER, attr, md)?
                .build()
        };

        Ok(Self {
            name_value: name_attr.value_handle,
            member_count_value: member_count_attr.value_handle,
            selected_member: AtomicU16::new(0),
            selected_member_value: selected_member_attr.value_handle,
            member_name_value: member_name_attr.value_handle,
            member_pronouns_value: member_pronouns_attr.value_handle,
        })
    }

    fn add_str_characteristic(
        sb: &mut ServiceBuilder,
        uuid: Uuid,
        val: Option<&'static [u8]>,
    ) -> Result<CharacteristicHandles, RegisterError> {
        let attr = Attribute::new(val.unwrap_or(&[0u8; 64])).deferred_read();
        let md = Metadata::new(Properties::new().read());
        Ok(sb.add_characteristic(uuid, attr, md)?.build())
    }

    pub fn on_write(
        &self,
        conn: &nrf_softdevice::ble::Connection,
        handle: u16,
        op: gatt_server::WriteOp,
        offset: usize,
        data: &[u8],
    ) -> Result<(), RawError> {
        if self.selected_member_value == handle {
            let member = match data.len() {
                1 => data[0] as u16,
                2.. => u16::from_le_bytes([data[0], data[1]]),
                _ => return Err(RawError::BleGattsInvalidAttrType),
            };
            self.selected_member
                .store(member, core::sync::atomic::Ordering::Relaxed);
            info!("Selected member: {}", member);
        }

        Ok(())
    }

    pub fn on_deferred_read(
        &self,
        handle: u16,
        offset: usize,
        reply: nrf_softdevice::ble::DeferredReadReply,
    ) -> Result<(), RawError> {
        let badge = match unsafe { crate::BADGE.assume_init_ref() }.try_lock() {
            Ok(badge) => badge,
            Err(_) => {
                warn!("Failed to lock badge");
                return reply.reply(Err(GattError::ATTERR_INSUF_RESOURCES));
            },
        };
        let badge = match badge.system.as_ref() {
            Some(badge) => badge,
            None => {
                info!("No system loaded");
                return reply.reply(Err(GattError::ATTERR_INSUF_RESOURCES));
            },
        };

        match handle {
            x if x == self.name_value => {
                trace!("Read system name");
                reply.reply(Ok(Some(badge.name().as_bytes())))
            },
            x if x == self.member_count_value => {
                trace!("Read system member count");
                let mut buf = [0u8; 2];
                buf.copy_from_slice(&(badge.member_count() as u16).to_le_bytes());
                reply.reply(Ok(Some(&buf)))
            },
            x if x == self.selected_member_value => {
                trace!("Read selected member");
                let mut buf = [0u8; 2];
                buf.copy_from_slice(
                    &self
                        .selected_member
                        .load(core::sync::atomic::Ordering::Relaxed)
                        .to_le_bytes(),
                );
                reply.reply(Ok(Some(&buf)))
            },
            x if x == self.member_name_value => {
                trace!("Read member name");
                let member = self
                    .selected_member
                    .load(core::sync::atomic::Ordering::Relaxed);
                let member = badge.member(member as usize);
                reply.reply(Ok(Some(member.name().as_bytes())))
            },
            x if x == self.member_pronouns_value => {
                trace!("Read member pronouns");
                let member = self
                    .selected_member
                    .load(core::sync::atomic::Ordering::Relaxed);
                let member = badge.member(member as usize);
                reply.reply(Ok(Some(member.pronouns().as_bytes())))
            },
            _ => Err(RawError::BleGapInvalidBleAddr),
        }
    }

    pub fn has_value(&self, handle: u16) -> bool {
        return handle == self.name_value
            || handle == self.member_count_value
            || handle == self.selected_member_value
            || handle == self.member_name_value
            || handle == self.member_pronouns_value;
    }
}
