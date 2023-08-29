#![no_std]
#![feature(ptr_metadata)]

extern crate alloc;

use crate::system::{DrawableMember, SystemUf2};
use core::hint::unreachable_unchecked;
use embedded_graphics::geometry::AnchorY;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::{Alignment, Text};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

pub mod system;

pub type DrawResult<D, T = ()> = Result<T, <D as DrawTarget>::Error>;

pub const HEIGHT: u32 = uc8151::HEIGHT;
pub const WIDTH: u32 = uc8151::WIDTH;

#[cfg(not(feature = "invert"))]
const BINARY_COLOR_OFF: BinaryColor = BinaryColor::Off;

#[cfg(feature = "invert")]
const BINARY_COLOR_OFF: BinaryColor = BinaryColor::On;

#[cfg(not(feature = "invert"))]
const BINARY_COLOR_ON: BinaryColor = BinaryColor::On;

#[cfg(feature = "invert")]
const BINARY_COLOR_ON: BinaryColor = BinaryColor::Off;

fn inc_wrapping<T>(cur: T, max: T) -> T
where
    T: core::ops::Add<T, Output = T>,
    T: Eq,
    T: TryFrom<u8>,
{
    if cur == max {
        unsafe { T::try_from(0).unwrap_unchecked() }
    } else {
        cur + unsafe { T::try_from(1).unwrap_unchecked() }
    }
}

fn dec_wrapping<T>(cur: T, max: T) -> T
where
    T: core::ops::Sub<T, Output = T>,
    T: Eq,
    T: TryFrom<u8>,
{
    if cur == unsafe { T::try_from(0).unwrap_unchecked() } {
        max
    } else {
        cur - unsafe { T::try_from(1).unwrap_unchecked() }
    }
}

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

#[derive(Eq, PartialEq, Debug, Clone, Copy, Default)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Select {
    #[default]
    None,
    Select,
    Edit,
}

impl Select {
    pub(crate) fn stroke_with(&self) -> u32 {
        match self {
            Self::None => 1,
            Self::Select => 2,
            Self::Edit => 4,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct MemberCell {
    id: u16,
}

#[derive(Eq, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct CurrentMembers {
    members: [MemberCell; 4],
    sel: (u8, Select),
    len: u8,
}

impl core::default::Default for CurrentMembers {
    fn default() -> Self {
        Self {
            len: 1,
            sel: (0, Select::Select),
            members: core::default::Default::default(),
        }
    }
}

impl CurrentMembers {
    fn sel_for_cell(&self, idx: u8) -> Select {
        if self.sel.0 == idx {
            self.sel.1
        } else {
            Select::None
        }
    }

    fn button_press(&mut self, button: Button, members: usize) {
        match button {
            Button::Up | Button::Down if self.sel.1 == Select::None => {
                self.sel.1 = Select::Select;
                if button == Button::Up {
                    self.sel.0 = 0;
                } else {
                    self.sel.0 = self.len - 1;
                }
            }
            Button::Up | Button::Down if self.sel.1 == Select::Select => {
                if button == Button::Up {
                    self.sel.0 = dec_wrapping(self.sel.0, self.len - 1);
                } else {
                    self.sel.0 = inc_wrapping(self.sel.0, self.len - 1);
                }
            }
            Button::Up | Button::Down if self.sel.1 == Select::Edit => {
                if button == Button::Up {
                    self.members[self.sel.0 as usize].id =
                        inc_wrapping(self.members[self.sel.0 as usize].id, members as u16 - 1);
                } else {
                    self.members[self.sel.0 as usize].id =
                        dec_wrapping(self.members[self.sel.0 as usize].id, members as u16 - 1);
                }
            }
            Button::C if self.sel.1 == Select::Edit => {
                self.len -= 1;
                if self.sel.0 != self.sel.0 {
                    self.members[self.sel.0 as usize] = self.members[self.len as usize];
                }
                self.sel.1 = Select::None;
            }
            Button::C => {
                if self.len < 4 {
                    self.len += 1;
                }
            }
            Button::B => {
                if self.sel.1 == Select::Select {
                    self.sel.1 = Select::Edit;
                } else {
                    self.sel.1 = Select::Select;
                }
            }
            _ => {
                #[cfg(feature = "defmt")]
                defmt::warn!("Unhandled member button press: {:?}", button)
            }
        }
    }

    fn draw<D>(&self, system: &SystemUf2, target: &mut D) -> DrawResult<D>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    {
        debug_assert!(self.len != 0 && self.len <= 4);

        match self.len {
            1 => {
                DrawableMember::new(
                    &system.members()[self.members[0].id as usize],
                    target.bounding_box(),
                    self.sel_for_cell(0),
                )
                .draw(target)?;
            }
            2 => {
                self.draw_two((0, 1), system, target.bounding_box(), target)?;
            }
            3 => {
                DrawableMember::new(
                    &system.members()[self.members[0].id as usize],
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 3, AnchorY::Top),
                    self.sel_for_cell(0),
                )
                .draw(target)?;
                DrawableMember::new(
                    &system.members()[self.members[1].id as usize],
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 3, AnchorY::Center),
                    self.sel_for_cell(1),
                )
                .draw(target)?;
                DrawableMember::new(
                    &system.members()[self.members[2].id as usize],
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 3, AnchorY::Bottom),
                    self.sel_for_cell(2),
                )
                .draw(target)?;
            }
            4 => {
                self.draw_two(
                    (0, 1),
                    system,
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 2, AnchorY::Top),
                    target,
                )?;
                self.draw_two(
                    (2, 3),
                    system,
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 2, AnchorY::Bottom),
                    target,
                )?;
            }
            _ => unsafe { unreachable_unchecked() },
        }

        Ok(())
    }

    fn draw_two<D>(
        &self,
        idx: (u8, u8),
        system: &SystemUf2,
        bounds: Rectangle,
        target: &mut D,
    ) -> DrawResult<D>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    {
        DrawableMember::new(
            &system.members()[self.members[idx.0 as usize].id as usize],
            bounds.resized_height(bounds.size.height / 2, AnchorY::Top),
            self.sel_for_cell(idx.0),
        )
        .draw(target)?;
        DrawableMember::new(
            &system.members()[self.members[idx.1 as usize].id as usize],
            bounds.resized_height(bounds.size.height / 2, AnchorY::Bottom),
            self.sel_for_cell(idx.1),
        )
        .draw(target)?;

        Ok(())
    }
}

#[derive(Eq, PartialEq, Debug)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum CurrentMenu {
    SystemName,
    Version,
    Member(CurrentMembers),
}

impl CurrentMenu {
    pub fn change(&mut self, button: Button, members: usize) {
        match self {
            Self::SystemName if button == Button::B => *self = Self::Version,
            Self::Version if button == Button::B => *self = Self::SystemName,
            Self::SystemName if button == Button::C => {
                *self = Self::Member(CurrentMembers::default())
            }
            Self::Member(ref c) if button == Button::C && c.len == 1 && c.sel.1 == Select::Edit => {
                *self = Self::SystemName
            }
            Self::Member(c) => c.button_press(button, members),
            _ => {
                //defmt::warn!("Unhandled button press: {:?}", button)
            }
        }
    }
}

pub struct Sysbadge<'a, D>
where
    D: DrawTarget,
    <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
{
    pub display: D,
    pub system: &'a SystemUf2,
    current: CurrentMenu,
    hash: u16,
}

impl<D> Sysbadge<'static, D>
where
    D: DrawTarget,
    <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
{
    pub fn new(display: D) -> Self {
        let system = unsafe { &*Self::system_start() };
        Self::new_with_system(display, system)
    }

    /// Get system start pointer from linker symbole.
    pub fn system_start() -> *const SystemUf2 {
        extern "C" {
            static mut __ssystem: SystemUf2;
        }

        unsafe { &__ssystem }
    }
}

impl<'a, D> Sysbadge<'a, D>
where
    D: DrawTarget,
    <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
{
    pub fn new_with_system(display: D, system: &'a SystemUf2) -> Self {
        Self {
            display,
            system,
            current: CurrentMenu::SystemName,
            hash: 0,
        }
    }

    pub fn press(&mut self, button: Button) {
        self.current.change(button, self.system.len());

        #[cfg(feature = "defmt")]
        defmt::debug!(
            "Pressed button: {:?}, switched to: {:?}",
            button,
            self.current
        );
    }

    pub fn draw(&mut self) -> DrawResult<D, bool> {
        let hash = self.hash();
        if self.hash == hash {
            return Ok(false);
        }

        self.force_draw()?;
        self.hash = hash;
        Ok(true)
    }

    pub fn force_draw(&mut self) -> DrawResult<D> {
        self.display.clear(BINARY_COLOR_OFF.into())?;
        match self.current {
            CurrentMenu::SystemName => self.draw_system_name(),
            CurrentMenu::Version => self.draw_version(),
            CurrentMenu::Member(ref cur) => cur.draw(self.system, &mut self.display),
        }
    }

    fn hash(&self) -> u16 {
        let mut crc: crc16::State<crc16::BUYPASS> = crc16::State::new();
        crc.update(unsafe {
            core::slice::from_raw_parts(
                &self.current as *const CurrentMenu as *const u8,
                core::mem::size_of::<CurrentMenu>(),
            )
        });
        crc.get()
    }

    fn draw_system_name(&mut self) -> DrawResult<D> {
        Text::with_alignment(
            self.system.name(),
            self.display.bounding_box().center(),
            MonoTextStyle::new(
                &embedded_graphics::mono_font::ascii::FONT_10X20,
                BINARY_COLOR_ON.into(),
            ),
            Alignment::Center,
        )
        .draw(&mut self.display)?;

        Ok(())
    }

    fn draw_version(&mut self) -> DrawResult<D> {
        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_10X20,
            BINARY_COLOR_ON.into(),
        );

        Text::with_alignment(
            "Sysbadge",
            self.display.bounding_box().center().x_axis() + Point::new(0, 20),
            text_style,
            Alignment::Center,
        )
        .draw(&mut self.display)?;

        Text::with_alignment(
            concat!("Version: ", env!("CARGO_PKG_VERSION")),
            Point::new(5, 60),
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;

        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_9X18,
            BINARY_COLOR_ON.into(),
        );
        Text::with_alignment(
            concat!(
                "matrix: ",
                env!("SYSBADGE_MATRIX", "missing matrix configuration")
            ),
            Point::new(5, 105),
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;
        Text::with_alignment(
            concat!("web: ", env!("SYSBADGE_WEB", "missing web configuration")),
            Point::new(5, 120),
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;

        Ok(())
    }

    #[cfg(feature = "simulator")]
    pub fn reset(&mut self) {
        self.current = CurrentMenu::SystemName;
    }
}
