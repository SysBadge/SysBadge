#![no_std]
#![feature(ptr_metadata)]
#![feature(return_position_impl_trait_in_trait)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

pub use system::System;

pub mod system;
pub mod usb;

pub mod badge;

pub type DrawResult<D, T = ()> = Result<T, <D as embedded_graphics::prelude::DrawTarget>::Error>;

pub const HEIGHT: u32 = uc8151::HEIGHT;
pub const WIDTH: u32 = uc8151::WIDTH;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const MATRIX: &'static str = env!("SYSBADGE_MATRIX", "missing matrix configuration");
pub const WEB: &'static str = env!("SYSBADGE_WEB", "missing web configuration");

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Button {
    A,
    B,
    C,
    D,
    Up,
    Down,
    USER,
}

impl TryFrom<u8> for Button {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Button::A as u8 => Ok(Button::A),
            x if x == Button::B as u8 => Ok(Button::B),
            x if x == Button::C as u8 => Ok(Button::C),
            x if x == Button::D as u8 => Ok(Button::D),
            x if x == Button::Up as u8 => Ok(Button::Up),
            x if x == Button::Down as u8 => Ok(Button::Down),
            x if x == Button::USER as u8 => Ok(Button::USER),
            _ => Err(()),
        }
    }
}
