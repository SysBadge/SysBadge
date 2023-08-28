use alloc::format;
use core::ffi::CStr;
use core::mem::MaybeUninit;

/// Flash representaion of a member
// INVARIANTS:
// - `name` has to be valid utf8 and null terminated
// - `pronouns` has to be valid utf8 and null terminated
#[repr(C)]
pub struct Member {
    name: [u8; 52],
    pronouns: [u8; 20],
}

#[cfg(feature = "simulator")]
impl Member {
    pub fn new_str(name: &str, pronouns: &str) -> Self {
        let ret = MaybeUninit::zeroed();
        let mut ret: Member = unsafe { ret.assume_init() };

        assert!(name.len() < ret.name.len());
        assert!(pronouns.len() < ret.pronouns.len());

        ret.name[..name.len()].copy_from_slice(name.as_bytes());
        ret.pronouns[..pronouns.len()].copy_from_slice(pronouns.as_bytes());

        ret
    }
}

impl Member {
    pub fn name(&self) -> &str {
        // SAFETY: type invariant
        unsafe {
            CStr::from_bytes_until_nul(&self.name)
                .unwrap_unchecked()
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn pronouns(&self) -> &str {
        // SAFETY: type invariant
        unsafe {
            CStr::from_bytes_until_nul(&self.pronouns)
                .unwrap_unchecked()
                .to_str()
                .unwrap_unchecked()
        }
    }
}

/// System definition as in the flash.
// INVARIANTS:
// - `name` has to be valid utf8 and null terminated
// - `members` has to point to a member array and be valid for `num_members`
#[repr(C)]
pub struct SystemUf2 {
    name: [u8; 100],
    members: *const Member,
    num_members: u16,
    crc16: u16,
}

#[cfg(feature = "simulator")]
impl SystemUf2 {
    /// This leaks the memory
    pub fn new_from_box(name: &str, members: alloc::boxed::Box<[Member]>) -> Self {
        let num_members = members.len() as u16;
        let mut ret = Self {
            name: [0; 100],
            members: alloc::boxed::Box::leak(members).as_ptr(),
            num_members,
            crc16: 0,
        };

        assert!(name.len() < 100);
        ret.name[..name.len()].copy_from_slice(name.as_bytes());

        ret
    }
}

impl SystemUf2 {
    pub fn name(&self) -> &str {
        // SAFETY: type invariant
        unsafe {
            CStr::from_bytes_until_nul(&self.name)
                .unwrap_unchecked()
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn members(&self) -> &[Member] {
        // SAFETY: held by type invariant
        unsafe { core::slice::from_raw_parts(self.members, self.num_members as usize) }
    }

    pub fn len(&self) -> usize {
        self.num_members as usize
    }
}

use crate::DrawResult;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::Drawable;

pub(crate) struct DrawableMember<'a> {
    member: &'a Member,
    bounds: Rectangle,
    select: super::Select,
}

impl<'a> DrawableMember<'a> {
    pub fn new(member: &'a Member, bounds: Rectangle, select: super::Select) -> Self {
        Self {
            member,
            bounds,
            select,
        }
    }

    fn pronoun<D: DrawTarget<Color = <Self as Drawable>::Color>>(
        &self,
        target: &mut D,
    ) -> DrawResult<(), D> {
        let (pos, align, font) = match self.bounds.size.height {
            x if x > 100 => (
                Point::new(5, (self.bounds.size.height - 15) as i32),
                Alignment::Left,
                &embedded_graphics::mono_font::ascii::FONT_10X20,
            ),
            x if x > 40 => (
                Point::new(5, 30),
                Alignment::Left,
                &embedded_graphics::mono_font::ascii::FONT_8X13,
            ),
            _ => (
                Point::new((self.bounds.size.width - 5) as i32, 15),
                Alignment::Right,
                &embedded_graphics::mono_font::ascii::FONT_6X10,
            ),
        };

        Text::with_alignment(
            &format!("({})", self.member.pronouns()),
            self.bounds.top_left + pos,
            MonoTextStyle::new(font, super::BINARY_COLOR_ON),
            align,
        )
        .draw(target)?;

        Ok(())
    }

    fn name<D: DrawTarget<Color = <Self as Drawable>::Color>>(
        &self,
        target: &mut D,
    ) -> DrawResult<(), D> {
        let (pos, font) = match self.bounds.size.height {
            x if x > 40 => (
                Point::new(5, 15),
                &embedded_graphics::mono_font::ascii::FONT_10X20,
            ),
            _ => (
                Point::new(5, 15),
                &embedded_graphics::mono_font::ascii::FONT_6X10,
            ),
        };

        Text::with_alignment(
            self.member.name(),
            self.bounds.top_left + pos,
            MonoTextStyle::new(font, super::BINARY_COLOR_ON),
            Alignment::Left,
        )
        .draw(target)?;

        Ok(())
    }
}

impl<'a> Drawable for DrawableMember<'a> {
    type Color = embedded_graphics::pixelcolor::BinaryColor;
    type Output = Rectangle;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let bound_style =
            PrimitiveStyle::with_stroke(super::BINARY_COLOR_ON, self.select.stroke_with());
        self.bounds.draw_styled(&bound_style, target)?;

        self.name(target)?;
        self.pronoun(target)?;

        Ok(self.bounds)
    }
}
