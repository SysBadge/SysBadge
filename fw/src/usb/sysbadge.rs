use defmt::export::usize;
use defmt::*;
use embassy_futures::block_on;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::Driver;
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Handler};
use embedded_storage_async::nor_flash::NorFlash;
use static_cell::make_static;

use ::sysbadge::usb as sysusb;
use sysbadge::badge::CurrentMenu;
use sysbadge::system::Member;
use sysbadge::System;

pub struct SysBadgeClass<'d, D: Driver<'d>> {
    _d: core::marker::PhantomData<&'d D>,
}

impl<'d, D: Driver<'d>> SysBadgeClass<'d, D> {
    pub fn new(builder: &mut Builder<'d, D>) -> Self {
        defmt::assert!(builder.control_buf_len() >= 64);

        let mut func = builder.function(0x0f, 0x00, 0x00);

        let mut iface = func.interface();
        let comm_if = iface.interface_number();

        drop(func);

        let control: &'static mut Control = make_static!(Control { comm_if });
        builder.handler(control);

        Self {
            _d: core::marker::PhantomData,
        }
    }
}

struct Control {
    comm_if: InterfaceNumber,
}

impl Handler for Control {
    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient, req.index)
            != (
                RequestType::Vendor,
                Recipient::Interface,
                self.comm_if.0 as u16,
            )
        {
            return None;
        }

        let request = match sysusb::Request::try_from(req.request) {
            Ok(req) => req,
            Err(_) => return Some(OutResponse::Rejected),
        };

        Some(match request {
            sysusb::Request::ButtonPress => {
                defmt::info!("button press");
                defmt::todo!();
            }
            sysusb::Request::Reboot => {
                defmt::info!("reboot");
                cortex_m::peripheral::SCB::sys_reset()
            }
            sysusb::Request::SystemUpload => block_on(init_system_load(req)),
            sysusb::Request::SystemDNLoad => {
                if req.length == 0 {
                    block_on(finalize_dnload(req))
                } else {
                    block_on(dnload(req, data))
                }
            }
            _ => defmt::todo!(),
        })
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if (req.request_type, req.recipient, req.index)
            != (
                RequestType::Vendor,
                Recipient::Interface,
                self.comm_if.0 as u16,
            )
        {
            return None;
        }

        let request = match sysusb::Request::try_from(req.request) {
            Ok(req) => req,
            Err(_) => return Some(InResponse::Rejected),
        };

        match request {
            sysusb::Request::GetSystemName => {
                defmt::info!("get system name");

                match req.value {
                    0 /* System name */ => block_on(async {
                        let badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
                        let out = badge.system.name().as_bytes();
                        if buf.len() < out.len() {
                            return Some(InResponse::Rejected);
                        }
                        buf[..out.len()].copy_from_slice(out);
                        Some(InResponse::Accepted(&buf[..out.len()]))
                    }),
                    1 /* PK HID */ => block_on(async {
                        let badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
                        defmt::todo!()
                    }),
                    _ => Some(InResponse::Rejected)
                }
            }
            sysusb::Request::GetMemberCount => {
                defmt::assert!(buf.len() >= 2);

                let count = block_on(async {
                    let badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
                    badge.system.member_count() as u16
                });
                buf[0..2].copy_from_slice(&count.to_le_bytes());

                Some(InResponse::Accepted(&buf[..2]))
            }
            sysusb::Request::GetMemberName => {
                debug!("Sending member name {}", req.value);

                let offset = req.value as usize;
                block_on(async {
                    let badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
                    if badge.system.member_count() <= offset {
                        trace!("Member {} not found", offset);
                        Some(InResponse::Rejected)
                    } else {
                        let member = badge.system.member(offset);
                        let name = member.name();
                        let name = name.as_bytes();
                        buf[..name.len()].copy_from_slice(name);
                        Some(InResponse::Accepted(&buf[..name.len()]))
                    }
                })
            }
            sysusb::Request::GetMemberPronouns => {
                debug!("Sending member pronouns {}", req.value);

                let offset = req.value as usize;
                block_on(async {
                    let badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
                    if badge.system.member_count() <= offset {
                        trace!("Member {} not found", offset);
                        Some(InResponse::Rejected)
                    } else {
                        let member = badge.system.member(offset);
                        let pronouns = member.pronouns();
                        let pronouns = pronouns.as_bytes();
                        buf[..pronouns.len()].copy_from_slice(pronouns);
                        Some(InResponse::Accepted(&buf[..pronouns.len()]))
                    }
                })
            }
            _ => defmt::todo!(),
        }
    }
}

/// Initialized updating the system
async fn init_system_load(req: Request) -> OutResponse {
    let mut badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
    if badge.current() == &CurrentMenu::Updating {
        return OutResponse::Rejected;
    }
    badge.set_current(CurrentMenu::Updating);

    extern "C" {
        static __ssystem_start: u8;
        static __ssystem_end: u8;
    }

    let system_offset = unsafe { &__ssystem_start as *const u8 as usize as u32 };
    let system_end = unsafe { &__ssystem_end as *const u8 as usize as u32 };

    // Only erase if reqeusted
    if req.value == 1 {
        use embedded_storage_async::nor_flash::NorFlash;
        if let Err(e) = unsafe { crate::FLASH.assume_init_ref() }
            .lock()
            .await
            .erase(system_offset, system_end)
            .await
        {
            warn!("Failed to erase flash: {:?}", e);
            return OutResponse::Rejected;
        }
    }

    // TODO: update display?
    OutResponse::Accepted
}

/// Finalize dnload loading the new system into the badge instance.
async fn finalize_dnload(_req: Request) -> OutResponse {
    info!("Finished donloading new system");
    let mut badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
    // check badge is in dnload mode
    if badge.current() != &CurrentMenu::Updating {
        return OutResponse::Rejected;
    }
    match unsafe { ::sysbadge::system::SystemReader::from_linker_symbols() } {
        Ok(system) => badge.system = system,
        Err(e) => {
            defmt::error!("Failed to read system");
            return OutResponse::Rejected;
        }
    }

    // Unblock
    badge.set_current(CurrentMenu::SystemName);

    OutResponse::Accepted
}

/// Load bytes into flash
async fn dnload(req: Request, data: &[u8]) -> OutResponse {
    if ((req.length % 4) != 0) || ((req.value % 4) != 0) {
        info!("Cannot dnload non-word-aligned data");
        return OutResponse::Rejected;
    }
    // check badge is in dnload mode
    if unsafe { crate::BADGE.assume_init_ref() }
        .lock()
        .await
        .current()
        != &CurrentMenu::Updating
    {
        return OutResponse::Rejected;
    }

    extern "C" {
        static __ssystem_start: u8;
        static __ssystem_end: u8;
    }

    let system_offset = unsafe { &__ssystem_start as *const u8 as usize as u32 };
    let system_end = unsafe { &__ssystem_end as *const u8 as usize as u32 };

    if (system_offset + req.value as u32 + req.length as u32) > system_end {
        info!("Cannot dnload data outside of system");
        return OutResponse::Rejected;
    }

    debug!("Flashing {} bytes at offset {}", req.length, req.value);

    use embedded_storage_async::nor_flash::NorFlash;
    let mut flash = unsafe { crate::FLASH.assume_init_ref() }.lock().await;
    if let Err(e) = flash
        .write(
            system_offset + req.value as u32,
            &data[..req.length as usize],
        )
        .await
    {
        warn!("Failed to write to flash: {:?}", e);
        return OutResponse::Rejected;
    }

    OutResponse::Accepted
}
