use alloc::format;
use alloc::string::ToString;
use core::alloc::Layout;
use core::ffi::CStr;
use core::mem::MaybeUninit;
use core::ptr;

/// Flash representaion of a member
// INVARIANTS:
// - `name` has to be valid utf8
// - `pronouns` has to be valid utf8
#[repr(C)]
pub struct Member {
    name: *mut str,
    pronouns: *mut str,
}

#[cfg(feature = "simulator")]
impl Member {
    pub fn new_str(name: &str, pronouns: &str) -> Self {
        use core::alloc::Layout;

        let layout_name = Layout::array::<u8>(name.len()).unwrap();
        let layout_pronouns = Layout::array::<u8>(pronouns.len()).unwrap();
        let (layout, pronouns_offset) = layout_name.extend(layout_pronouns).unwrap();

        let ptr = unsafe {
            let ptr = alloc::alloc::alloc(layout);
            ptr::copy_nonoverlapping(name.as_ptr(), ptr, name.len());
            ptr::copy_nonoverlapping(pronouns.as_ptr(), ptr.add(pronouns_offset), pronouns.len());
            ptr
        };

        Self {
            name: ptr::from_raw_parts_mut(ptr.cast(), name.len()),
            pronouns: ptr::from_raw_parts_mut(
                unsafe { ptr.add(pronouns_offset).cast() },
                pronouns.len(),
            ),
        }
    }
}

impl Member {
    #[inline(always)]
    pub fn name(&self) -> &str {
        // SAFETY: type invariant
        unsafe { &*self.name }
    }

    #[inline(always)]
    pub fn pronouns(&self) -> &str {
        // SAFETY: type invariant
        unsafe { &*self.pronouns }
    }
}

#[cfg(feature = "simulator")]
impl Drop for Member {
    fn drop(&mut self) {
        // TEST if really from simulator memory
        let name_len = core::ptr::metadata(self.name);
        if unsafe { (self.name as *const u8).add(name_len) } == (self.pronouns as *const u8) {
            unsafe {
                alloc::alloc::dealloc(
                    self.name.cast(),
                    Layout::from_size_align_unchecked(name_len + ptr::metadata(self.pronouns), 1),
                )
            }
        }
    }
}

/// System definition as in the flash.
// INVARIANTS:
// - `name` and `members` have to be valid fat pointers
// - `name` has to point to a valid utf8 string
// - `members` has to point to a valid member array
#[repr(C)]
pub struct SystemUf2 {
    name: *const str,
    members: *const [Member],
    crc16: u16,
}

#[cfg(feature = "simulator")]
impl SystemUf2 {
    pub const ZERO: Self = Self {
        name: unsafe { ptr::from_raw_parts_mut(ptr::null_mut(), 0) },
        members: unsafe { ptr::from_raw_parts_mut(ptr::null_mut(), 0) },
        crc16: 0,
    };

    /// This leaks the memory
    pub fn new_from_box(
        name: alloc::boxed::Box<str>,
        members: alloc::boxed::Box<[Member]>,
    ) -> Self {
        Self {
            name: alloc::boxed::Box::leak(name),
            members: alloc::boxed::Box::leak(members),
            crc16: 0,
        }
    }
}

impl SystemUf2 {
    #[inline(always)]
    pub fn name(&self) -> &str {
        // SAFETY: type invariant
        unsafe { &*self.name }
    }

    #[inline(always)]
    pub fn members(&self) -> &[Member] {
        // SAFETY: held by type invariant
        unsafe { &*self.members }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        core::ptr::metadata(self.members)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

pub(crate) struct DrawableMember<'a, C>
where
    C: PixelColor + From<BinaryColor>,
{
    member: &'a Member,
    bounds: Rectangle,
    select: super::Select,
    _color: core::marker::PhantomData<C>,
}

impl<'a, C> DrawableMember<'a, C>
where
    C: PixelColor + From<BinaryColor>,
{
    pub fn new(member: &'a Member, bounds: Rectangle, select: super::Select) -> Self {
        Self {
            member,
            bounds,
            select,
            _color: core::marker::PhantomData,
        }
    }

    fn pronoun<D>(&self, target: &mut D) -> DrawResult<D, Point>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    {
        let (pos, align, font) = match self.bounds.size.height {
            x if x > 100 => (
                Point::new(5, (self.bounds.size.height - 20) as i32),
                Alignment::Left,
                &embedded_graphics::mono_font::ascii::FONT_10X20,
            ),
            x if x > 50 => (
                Point::new(5, (self.bounds.size.height - 15) as i32),
                Alignment::Left,
                &embedded_graphics::mono_font::ascii::FONT_8X13,
            ),
            x if x > 40 => (
                Point::new((self.bounds.size.width - 5) as i32, 15),
                Alignment::Right,
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
            MonoTextStyle::new(font, super::BINARY_COLOR_ON.into()),
            align,
        )
        .draw(target)
    }

    fn name<D>(&self, target: &mut D) -> DrawResult<D, Point>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    {
        let (pos, font) = match self.bounds.size.height {
            x if x > 40 => (
                Point::new(5, 25),
                //&embedded_graphics::mono_font::ascii::FONT_10X20,
                &profont::PROFONT_24_POINT,
            ),
            x if x > 20 => (
                Point::new(5, 20),
                &embedded_graphics::mono_font::ascii::FONT_8X13,
            ),
            _ => (
                Point::new(5, 20),
                &embedded_graphics::mono_font::ascii::FONT_6X10,
            ),
        };

        Text::with_alignment(
            self.member.name(),
            self.bounds.top_left + pos,
            MonoTextStyle::new(font, super::BINARY_COLOR_ON.into()),
            Alignment::Left,
        )
        .draw(target)
    }
}

impl<'a, C> Drawable for DrawableMember<'a, C>
where
    C: PixelColor + From<BinaryColor>,
{
    type Color = C;
    type Output = Rectangle;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    {
        let bound_style =
            PrimitiveStyle::with_stroke(super::BINARY_COLOR_ON.into(), self.select.stroke_with());
        self.bounds.draw_styled(&bound_style, target)?;

        self.name(target)?;
        if !self.member.pronouns().is_empty() {
            self.pronoun(target)?;
        }

        Ok(self.bounds)
    }
}
