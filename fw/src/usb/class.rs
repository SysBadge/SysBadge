use core::mem::MaybeUninit;
use defmt::*;
use embassy_futures::block_on;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::Driver;
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Handler};

use crate::{UsbControl, UsbResponse, USB_RESP};
use sysbadge::usb as sysusb;

pub struct State {
    control: MaybeUninit<Control>,
}

impl State {
    pub fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
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

        let control = state.control.write(Control { comm_if });
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

        match req.request {
            x if x == (sysusb::Request::ButtonPress as u8) && req.length == 2 => {
                defmt::assert!(data.len() >= 2);

                Some(match (req.value as u8).try_into() {
                    Ok(button) => {
                        crate::CHANNEL.try_send(button).unwrap();
                        OutResponse::Accepted
                    }
                    Err(_) => OutResponse::Rejected,
                })
            }
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        warn!("got into controll function");
        if (req.request_type, req.recipient, req.index)
            != (
                RequestType::Vendor,
                Recipient::Interface,
                self.comm_if.0 as u16,
            )
        {
            return None;
        }

        match req.request {
            x if x == (sysusb::Request::GetMemberCount as u8) && req.length == 2 => {
                debug!("Sending member count");
                warn!("implement with actual data");
                defmt::assert!(buf.len() >= 2);

                /*crate::USB.try_send(UsbControl::GetMemberCount).unwrap();
                if let UsbResponse::MemberCount(count) = USB_RESP.try_receive().unwrap() {
                    buf[0..2].copy_from_slice(&count.to_le_bytes());
                }*/

                Some(InResponse::Accepted(&buf[..2]))
            }
            _ => Some(InResponse::Rejected),
        }
    }
}
