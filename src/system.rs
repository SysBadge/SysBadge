use alloc::format;
use core::mem::MaybeUninit;
use core::{mem, ptr};

pub trait Member {
    fn name(&self) -> &str;
    fn pronouns(&self) -> &str;
}

pub trait System {
    fn name(&self) -> &str;
    fn member_count(&self) -> usize;
    fn member(&self, index: usize) -> &dyn Member;
}

impl<'a, S: System> System for &S {
    fn name(&self) -> &str {
        (*self).name()
    }

    fn member_count(&self) -> usize {
        (*self).member_count()
    }

    fn member(&self, index: usize) -> &dyn Member {
        (*self).member(index)
    }
}

pub struct SystemVec {
    pub name: alloc::string::String,
    pub members: alloc::vec::Vec<MemberStrings>,
}

impl SystemVec {
    pub fn new(name: alloc::string::String) -> Self {
        Self {
            name,
            members: alloc::vec::Vec::new(),
        }
    }

    /*pub fn import_bin(bin: &[u8], offset: u32) -> Self {
        let mut ret = Self {
            name: alloc::string::String::new(),
            members: alloc::vec::Vec::new(),
        };

        let system: &SystemUf2 = unsafe { &*(bin.as_ptr() as *const _) };
        let name_len = ptr::metadata(system.name) as usize;
        let name_offset = system.name as usize - offset as usize;
        let member_count = ptr::metadata(system.members) as usize;
        let member_offset = system.members as usize - offset as usize;
        drop(system);
        todo!()
    }*/

    pub fn get_bin(&self, offset: u32) -> alloc::vec::Vec<u8> {
        let mut ret = alloc::vec::Vec::new();

        let mut system = MaybeUninit::zeroed();

        let name_addr = next_after::<u8>(mem::size_of::<SystemUf2>() as u32);
        let name_len = self.name.len() as u32;
        let member_addr = next_after::<MemberUF2>(name_addr + name_len);
        unsafe {
            // writing name address
            ptr::copy_nonoverlapping(
                (offset + name_addr).to_le_bytes().as_ptr(),
                system.as_mut_ptr() as *mut u8,
                4,
            );
            // writing name length
            ptr::copy_nonoverlapping(
                name_len.to_le_bytes().as_ptr(),
                (system.as_mut_ptr() as *mut u8).add(4),
                4,
            );

            // write member offset
            ptr::copy_nonoverlapping(
                (offset + member_addr).to_le_bytes().as_ptr(),
                (system.as_mut_ptr() as *mut u8).add(8),
                4,
            );
            // write member count
            ptr::copy_nonoverlapping(
                self.members.len().to_le_bytes().as_ptr(),
                (system.as_mut_ptr() as *mut u8).add(12),
                4,
            );
        }

        let system: SystemUf2 = unsafe { system.assume_init() };
        ret.extend(core::iter::repeat(0).take(member_addr as usize));
        unsafe {
            // writing system information
            ptr::copy_nonoverlapping(
                &system as *const SystemUf2 as *const u8,
                ret.as_mut_ptr(),
                mem::size_of::<SystemUf2>(),
            );
            // writing name
            ptr::copy_nonoverlapping(
                self.name.as_ptr(),
                ret.as_mut_ptr().add(name_addr as usize),
                name_len as usize,
            );
        }

        self.write_members(&mut ret, offset);

        ret
    }

    fn write_members(&self, buf: &mut alloc::vec::Vec<u8>, offset: u32) {
        let mut start_addr = buf.len();
        let member_bytes = mem::size_of::<MemberUF2>() * self.members.len();
        let mut member_end = (start_addr + member_bytes) as u32;
        buf.extend(core::iter::repeat(0).take(member_bytes));

        for member in &self.members {
            member_end += Self::write_member(member_end + offset, start_addr, member, buf);

            start_addr += mem::size_of::<MemberUF2>();
        }
    }

    fn write_member(
        offset: u32,
        member_offset: usize,
        member: &MemberStrings,
        buf: &mut alloc::vec::Vec<u8>,
    ) -> u32 {
        let name_len = member.name.len() as u32;
        let pronouns_len = member.pronouns.len() as u32;
        let start_addr = buf.len();
        buf.extend(core::iter::repeat(0).take((name_len + pronouns_len) as usize));

        // write member pointers
        unsafe {
            let member_ptr = buf.as_mut_ptr().add(member_offset);

            // write name pointer
            ptr::copy_nonoverlapping(offset.to_le_bytes().as_ptr(), member_ptr, 4);
            // write name len
            ptr::copy_nonoverlapping(name_len.to_le_bytes().as_ptr(), member_ptr.add(4), 4);

            // write pronouns pointer
            ptr::copy_nonoverlapping(
                (offset + name_len).to_le_bytes().as_ptr(),
                member_ptr.add(8),
                4,
            );
            // write pronouns len
            ptr::copy_nonoverlapping(pronouns_len.to_le_bytes().as_ptr(), member_ptr.add(12), 4);
        }

        // write strings
        unsafe {
            ptr::copy_nonoverlapping(
                member.name.as_ptr(),
                buf.as_mut_ptr().add(start_addr),
                name_len as usize,
            );
            ptr::copy_nonoverlapping(
                member.pronouns.as_ptr(),
                buf.as_mut_ptr().add(start_addr + name_len as usize),
                pronouns_len as usize,
            );
        }

        name_len + pronouns_len
    }
}

const fn next_after<T: Sized>(curr: u32) -> u32 {
    let pad = bytes_to_align(mem::align_of::<T>() as u32, curr);
    curr + pad
}

const fn bytes_to_align(align: u32, bytes: u32) -> u32 {
    (align - (bytes % align)) % align
}

impl System for SystemVec {
    fn name(&self) -> &str {
        &self.name
    }

    fn member_count(&self) -> usize {
        self.members.len()
    }

    fn member(&self, index: usize) -> &dyn Member {
        &self.members[index]
    }
}

pub struct MemberStrings {
    pub name: alloc::string::String,
    pub pronouns: alloc::string::String,
}

impl Member for MemberStrings {
    fn name(&self) -> &str {
        &self.name
    }

    fn pronouns(&self) -> &str {
        &self.pronouns
    }
}

trait U32Pointee: ptr::Pointee {
    type Metadata: Copy + Send + Sync + Ord + core::hash::Hash + Unpin;
}

/*impl<P: core::ptr::Pointee<Metadata=()>> U32Pointee for P {
    type Metadata = ();
}*/

impl<P: ptr::Pointee<Metadata = usize> + ?Sized> U32Pointee for P {
    type Metadata = u32;
}

#[repr(C)]
struct U32PtrRepr<T: U32Pointee + ?Sized> {
    addr: u32,
    metadata: <T as U32Pointee>::Metadata,
}

impl<T: U32Pointee + ?Sized> U32PtrRepr<T> {
    pub fn from_raw_parts(addr: u32, metadata: <T as U32Pointee>::Metadata) -> Self {
        Self { addr, metadata }
    }
}

/*impl<T: U32Pointee<Metadata=()>> U32PtrRepr<T> {
    fn get(&self) -> *const T {
        self.addr as *const T
    }
}*/

impl<T: U32Pointee<Metadata = u32> + ptr::Pointee<Metadata = usize> + ?Sized> U32PtrRepr<T> {
    fn get(&self) -> *const T {
        ptr::from_raw_parts(self.addr as *const (), self.metadata as usize)
    }
}

/// Flash representaion of a member
// INVARIANTS:
// - `name` has to be valid utf8
// - `pronouns` has to be valid utf8
#[repr(C)]
pub struct MemberUF2 {
    name: U32PtrRepr<str>,
    pronouns: U32PtrRepr<str>,
}

impl MemberUF2 {
    #[inline(always)]
    pub fn name(&self) -> &str {
        // SAFETY: type invariant
        unsafe { &*self.name.get() }
    }

    #[inline(always)]
    pub fn pronouns(&self) -> &str {
        // SAFETY: type invariant
        unsafe { &*self.pronouns.get() }
    }
}

impl Member for MemberUF2 {
    fn name(&self) -> &str {
        self.name()
    }

    fn pronouns(&self) -> &str {
        self.pronouns()
    }
}

/// System definition as in the flash.
// INVARIANTS:
// - `name` and `members` have to be valid fat pointers
// - `name` has to point to a valid utf8 string
// - `members` has to point to a valid member array
#[repr(C)]
pub struct SystemUf2 {
    name: U32PtrRepr<str>,
    members: U32PtrRepr<[MemberUF2]>,
    crc16: u16,
}

#[cfg(feature = "simulator")]
impl SystemUf2 {
    /*pub const ZERO: Self = Self {
        name: unsafe { ptr::from_raw_parts_mut(ptr::null_mut(), 0) },
        members: unsafe { ptr::from_raw_parts_mut(ptr::null_mut(), 0) },
        crc16: 0,
    };*/
}

impl SystemUf2 {
    #[inline(always)]
    pub fn name(&self) -> &str {
        // SAFETY: type invariant
        unsafe { &*self.name.get() }
    }

    #[inline(always)]
    pub fn members(&self) -> &[MemberUF2] {
        // SAFETY: held by type invariant
        unsafe { &*self.members.get() }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.members.metadata as usize
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl System for SystemUf2 {
    fn name(&self) -> &str {
        self.name()
    }

    fn member_count(&self) -> usize {
        self.len()
    }

    fn member(&self, index: usize) -> &dyn Member {
        &self.members()[index]
    }
}

unsafe impl Send for SystemUf2 {}
unsafe impl Sync for SystemUf2 {}

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
    member: &'a dyn Member,
    bounds: Rectangle,
    select: super::Select,
    _color: core::marker::PhantomData<C>,
}

impl<'a, C> DrawableMember<'a, C>
where
    C: PixelColor + From<BinaryColor>,
{
    pub fn new(member: &'a dyn Member, bounds: Rectangle, select: super::Select) -> Self {
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
