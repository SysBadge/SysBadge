use core::sync::atomic::{AtomicBool, Ordering};

use ::sysbadge::usb as sysusb;
use defmt::*;
use embassy_futures::block_on;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::Driver;
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Handler};
use static_cell::make_static;
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

        let iface = func.interface();
        let comm_if = iface.interface_number();

        drop(func);

        let control: &'static mut Control = make_static!(Control { comm_if });

        builder.handler(control);

        Self {
            _d: core::marker::PhantomData,
        }
    }

    pub async fn run(&self) {
        run().await;
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
            },
            sysusb::Request::Reboot => {
                defmt::info!("reboot");
                cortex_m::peripheral::SCB::sys_reset()
            },
            sysusb::Request::SystemUpload => init_system_load(req),
            sysusb::Request::SystemDNLoad => {
                if req.length == 0 {
                    finalize_dnload(req)
                } else {
                    dnload(req, data)
                }
            },
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
                        let system = match badge.system.as_ref() {
                            Some(s) => s,
                            None => return Some(InResponse::Rejected),
                        };
                        let out = system.name().as_bytes();
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
            },
            sysusb::Request::GetMemberCount => {
                defmt::assert!(buf.len() >= 2);

                block_on(async {
                    let badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
                    let system = match badge.system.as_ref() {
                        Some(s) => s,
                        None => return Some(InResponse::Rejected),
                    };
                    let count = system.member_count() as u16;
                    buf[0..2].copy_from_slice(&count.to_le_bytes());
                    Some(InResponse::Accepted(&buf[..2]))
                })
            },
            sysusb::Request::GetMemberName => {
                debug!("Sending member name {}", req.value);

                let offset = req.value as usize;
                block_on(async {
                    let badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
                    let system = match badge.system.as_ref() {
                        Some(s) => s,
                        None => return Some(InResponse::Rejected),
                    };
                    if system.member_count() <= offset {
                        trace!("Member {} not found", offset);
                        Some(InResponse::Rejected)
                    } else {
                        let member = system.member(offset);
                        let name = member.name();
                        let name = name.as_bytes();
                        buf[..name.len()].copy_from_slice(name);
                        Some(InResponse::Accepted(&buf[..name.len()]))
                    }
                })
            },
            sysusb::Request::GetMemberPronouns => {
                debug!("Sending member pronouns {}", req.value);

                let offset = req.value as usize;
                block_on(async {
                    let badge = unsafe { crate::BADGE.assume_init_ref() }.lock().await;
                    let system = match badge.system.as_ref() {
                        Some(s) => s,
                        None => return Some(InResponse::Rejected),
                    };
                    if system.member_count() <= offset {
                        trace!("Member {} not found", offset);
                        Some(InResponse::Rejected)
                    } else {
                        let member = system.member(offset);
                        let pronouns = member.pronouns();
                        let pronouns = pronouns.as_bytes();
                        buf[..pronouns.len()].copy_from_slice(pronouns);
                        Some(InResponse::Accepted(&buf[..pronouns.len()]))
                    }
                })
            },
            sysusb::Request::SystemUpload => {
                defmt::info!("system upload status request");
                if IN_FLIGHT.load(Ordering::Relaxed) {
                    Some(InResponse::Accepted(&[SystemUpdateStatus::Writing as u8]))
                } else if OUT.signaled() {
                    let status = block_on(OUT.wait());
                    buf[0] = status as u8;
                    Some(InResponse::Accepted(&buf[..1]))
                } else {
                    Some(InResponse::Accepted(&[
                        SystemUpdateStatus::NotInUpdateMode as u8
                    ]))
                }
            },
            _ => defmt::todo!(),
        }
    }
}

/// Initialized updating the system
fn init_system_load(req: Request) -> OutResponse {
    debug!("Preparing to update system");
    if let Ok(mut badge) = unsafe { crate::BADGE.assume_init_ref() }.try_lock() {
        badge.set_current(CurrentMenu::Updating);
    } else {
        debug!("Failed to acquire badge lock");
        return OutResponse::Rejected;
    }

    if req.value == 1 {
        // Erase system
        if !try_schedule(InMsg::EraseSystem) {
            debug!("Failed to schedule erase system");
            return OutResponse::Rejected;
        }
    }

    OUT.signal(SystemUpdateStatus::ReadyForUpdate);
    info!("Ready to dnload system");
    // TODO: update display?
    OutResponse::Accepted
}

/// Finalize dnload loading the new system into the badge instance.
fn finalize_dnload(_req: Request) -> OutResponse {
    if IN_FLIGHT.load(Ordering::Relaxed) {
        debug!("Cannot finalize dnload while in-flight");
        return OutResponse::Rejected;
    }

    info!("Finished donloading new system");
    let badge = unsafe { crate::BADGE.assume_init_ref() }.try_lock();
    if badge.is_err() {
        return OutResponse::Rejected;
    }
    let mut badge = unsafe { badge.unwrap_unchecked() };
    // check badge is in dnload mode
    if badge.current() != &CurrentMenu::Updating {
        debug!("Cannot finalize dnload when not in dnload mode");
        return OutResponse::Rejected;
    }
    match unsafe { ::sysbadge::system::SystemReader::from_linker_symbols() } {
        Ok(system) => badge.system = Some(system),
        Err(_e) => {
            defmt::error!("Failed to read system");
            return OutResponse::Rejected;
        },
    }

    // Unblock
    badge.set_current(CurrentMenu::SystemName);
    info!(
        "Loaded new system: {}",
        badge.system.as_ref().unwrap().name()
    );

    OutResponse::Accepted
}

/// Load bytes into flash
fn dnload(req: Request, data: &[u8]) -> OutResponse {
    if ((req.length % 4) != 0) || ((req.value % 4) != 0) {
        info!("Cannot dnload non-word-aligned data");
        return OutResponse::Rejected;
    }
    // check badge is in dnload mode
    if let Ok(badge) = unsafe { crate::BADGE.assume_init_ref() }.try_lock() {
        if badge.current() != &CurrentMenu::Updating {
            debug!("Cannot dnload data when not in dnload mode");
            return OutResponse::Rejected;
        }
    } else {
        debug!("Failed to acquire badge lock");
        return OutResponse::Rejected;
    }

    let system_offset = unsafe { &__ssystem_start as *const u8 as usize as u32 };
    let system_end = unsafe { &__ssystem_end as *const u8 as usize as u32 };

    if (system_offset + req.value as u32 + req.length as u32) > system_end {
        info!("Cannot dnload data outside of system");
        return OutResponse::Rejected;
    }

    debug!(
        "Flashing {} bytes at offset {:x} ({:x})",
        req.length,
        req.value,
        system_offset + req.value as u32
    );

    let mut buf = [0u8; 64];
    (&mut buf[..req.length as usize]).copy_from_slice(&data[..req.length as usize]);
    let msg = InMsg::WritePart {
        offset: req.value,
        bytes: buf,
    };
    if !try_schedule(msg) {
        debug!("Failed to schedule write");
        return OutResponse::Rejected;
    }

    debug!("scheduled to flashe {} bytes", req.length);
    OutResponse::Accepted
}

// writer task
extern "C" {
    static __ssystem_start: u8;
    static __ssystem_end: u8;
}

enum InMsg {
    EraseSystem,
    WritePart { offset: u16, bytes: [u8; 64] },
}

static IN_FLIGHT: AtomicBool = AtomicBool::new(false);
static IN_CHANNEL: Channel<CriticalSectionRawMutex, InMsg, 1> = Channel::new();
static OUT: Signal<CriticalSectionRawMutex, SystemUpdateStatus> = Signal::new();
use embedded_storage_async::nor_flash::NorFlash;
use sysbadge::usb::SystemUpdateStatus;

fn try_schedule(msg: InMsg) -> bool {
    debug!("Trying to schedule new operation");
    IN_CHANNEL.try_send(msg).is_ok()
}

//#[embassy_executor::task]
pub async fn run() -> ! {
    let system_offset = unsafe { &__ssystem_start as *const u8 as usize as u32 };
    let system_end = unsafe { &__ssystem_end as *const u8 as usize as u32 };
    loop {
        let msg = IN_CHANNEL.recv().await;
        IN_FLIGHT.store(true, Ordering::Relaxed);
        match msg {
            InMsg::EraseSystem => {
                let mut flash = unsafe { crate::FLASH.assume_init_ref() }.lock().await;
                info!("Erasing flash");
                if let Err(e) = flash.erase(system_offset, system_end).await {
                    warn!("Failed to erase flash: {:?}", e);
                    OUT.signal(SystemUpdateStatus::EraseError);
                } else {
                    debug!("Finished erasing flash");
                    OUT.signal(SystemUpdateStatus::ErasedForUpdate);
                }
            },
            InMsg::WritePart { offset, bytes } => {
                info!("Writing {} bytes at offset {:x}", bytes.len(), offset);
                if let Err(e) = unsafe { crate::FLASH.assume_init_ref() }
                    .lock()
                    .await
                    .write(system_offset + offset as u32, &bytes)
                    .await
                {
                    warn!("Failed to write flash: {:?}", e);
                    OUT.signal(SystemUpdateStatus::WriteError);
                } else {
                    debug!("Finished writing flash");
                    OUT.signal(SystemUpdateStatus::Written);
                }
            },
        }
        debug!("Unlocking in-flight");
        IN_FLIGHT.store(false, Ordering::Relaxed);
    }
}
