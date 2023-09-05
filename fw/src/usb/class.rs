use core::mem::MaybeUninit;
use defmt::*;
use embassy_futures::block_on;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::Driver;
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Handler};

use crate::{UsbControl, UsbResponse, USB_RESP};
use sysbadge::{usb as sysusb, CurrentMenu};

pub struct State {
    control: MaybeUninit<Control>,
    badge: &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>,
}

impl State {
    pub fn new(
        badge: &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>,
    ) -> Self {
        Self {
            control: MaybeUninit::uninit(),
            badge,
        }
    }
}

pub struct SysbadgeClass<'d, D: Driver<'d>> {
    //read_ep: D::EndpointIn,
    //write_ep: D::EndpointOut,
    _d: core::marker::PhantomData<&'d D>,
    //control: Control,
}

impl<'d, D: Driver<'d>> SysbadgeClass<'d, D> {
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State, max_packet_size: u16) -> Self {
        defmt::assert!(builder.control_buf_len() >= 4);

        let mut func = builder.function(0x0f, 0x00, 0x00);

        // control interface
        let mut iface = func.interface();
        let comm_if = iface.interface_number();
        let mut alt = iface.alt_setting(0x0f, 0x00, 0x00, None);

        let comm_ep = alt.endpoint_interrupt_in(8, 255);

        drop(func);

        let control = state.control.write(Control {
            comm_if,
            badge: state.badge,
        });
        builder.handler(control);

        Self {
            _d: core::marker::PhantomData,
        }
    }
}

struct Control {
    comm_if: InterfaceNumber,
    badge: &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>,
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

        match sysusb::Request::try_from(req.request) {
            Ok(sysusb::Request::ButtonPress) => Some(match (req.value as u8).try_into() {
                Ok(button) => {
                    block_on(async {
                        crate::CHANNEL.send(button).await;
                    });
                    OutResponse::Accepted
                }
                Err(_) => OutResponse::Rejected,
            }),
            Ok(sysusb::Request::SetState)
                if req.length == core::mem::size_of::<sysbadge::CurrentMenu>() as u16 =>
            {
                debug!("Received state");
                defmt::assert!(data.len() >= core::mem::size_of::<sysbadge::CurrentMenu>());

                let state = CurrentMenu::from_bytes(data);

                let state = block_on(async {
                    let mut badge = self.badge.lock().await;
                    badge.set_current(state);
                });

                Some(OutResponse::Accepted)
            }
            Ok(sysusb::Request::UpdateDisplay) => {
                debug!("Received update display");
                block_on(async {
                    let mut badge = self.badge.lock().await;
                    unwrap!(badge.draw());
                    unwrap!(badge.display.update(), "Failed to update display");
                });
                Some(OutResponse::Accepted)
            }
            _ => Some(OutResponse::Rejected),
        }
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

        match sysusb::Request::try_from(req.request) {
            Ok(sysusb::Request::GetSystemName) => {
                debug!("Sending system name");

                let offset = req.value as usize;

                let name = block_on(async {
                    let badge = self.badge.lock().await;
                    let name = badge.system.name();
                    if offset >= name.len() {
                        return None;
                    }

                    let len = core::cmp::min(buf.len(), name.len() - offset);
                    Some(&name.as_bytes()[offset..len])
                });

                if let Some(name) = name {
                    buf[..name.len()].copy_from_slice(name);
                    Some(InResponse::Accepted(&buf[..name.len()]))
                } else {
                    Some(InResponse::Rejected)
                }
            }
            Ok(sysusb::Request::GetMemberCount) if req.length == 2 => {
                debug!("Sending member count");
                defmt::assert!(buf.len() >= 2);

                let count = block_on(async {
                    let badge = self.badge.lock().await;
                    badge.system.len() as u16
                });
                buf[0..2].copy_from_slice(&count.to_le_bytes());

                Some(InResponse::Accepted(&buf[..2]))
            }
            Ok(sysusb::Request::GetMemberName) => {
                debug!("Sending member name {}", req.value);

                let offset = req.value as usize;
                let name = block_on(async {
                    let badge = self.badge.lock().await;
                    if badge.system.len() <= offset {
                        trace!("Member {} not found", offset);
                        None
                    } else {
                        Some(badge.system.members()[offset].name())
                    }
                });

                if let Some(name) = name {
                    buf[..name.len()].copy_from_slice(name.as_bytes());
                    Some(InResponse::Accepted(&buf[..name.len()]))
                } else {
                    Some(InResponse::Rejected)
                }
            }
            Ok(sysusb::Request::GetMemberPronouns) => {
                debug!("Sending member pronouns {}", req.value);

                let offset = req.value as usize;
                let pronouns = block_on(async {
                    let badge = self.badge.lock().await;
                    if badge.system.len() <= offset {
                        trace!("Member {} not found", offset);
                        None
                    } else {
                        Some(badge.system.members()[offset].pronouns())
                    }
                });

                if let Some(pronouns) = pronouns {
                    buf[..pronouns.len()].copy_from_slice(pronouns.as_bytes());
                    Some(InResponse::Accepted(&buf[..pronouns.len()]))
                } else {
                    Some(InResponse::Rejected)
                }
            }
            Ok(sysusb::Request::GetState)
                if req.length == core::mem::size_of::<sysbadge::CurrentMenu>() as u16 =>
            {
                debug!("Sending state");
                defmt::assert!(buf.len() >= core::mem::size_of::<sysbadge::CurrentMenu>());

                let len = block_on(async {
                    let badge = self.badge.lock().await;
                    let state = badge.current().as_bytes();
                    buf[..state.len()].copy_from_slice(state);
                    state.len()
                });

                Some(InResponse::Accepted(&buf[..len]))
            }
            _ => Some(InResponse::Rejected),
        }
    }
}
