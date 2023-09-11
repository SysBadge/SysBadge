use crate::system::Member;
use crate::System;
use alloc::borrow::Cow;
use core::ptr;

pub const MAGIC: u32 = 0xa2b5;
pub const VERSION_1: u16 = 0x0001;

pub trait U32Pointee: ptr::Pointee {
    type Metadata: Copy + Send + Sync + Ord + core::hash::Hash + Unpin;
}

/*impl<P: core::ptr::Pointee<Metadata=()>> U32Pointee for P {
    type Metadata = ();
}*/

impl<P: ptr::Pointee<Metadata = usize> + ?Sized> U32Pointee for P {
    type Metadata = u32;
}

#[repr(C)]
pub struct U32PtrRepr<T: U32Pointee + ?Sized> {
    addr: u32,
    metadata: <T as U32Pointee>::Metadata,
}

impl<T: U32Pointee + ?Sized> U32PtrRepr<T> {
    pub const fn from_raw_parts(addr: u32, metadata: <T as U32Pointee>::Metadata) -> Self {
        Self { addr, metadata }
    }

    pub const fn addr(&self) -> u32 {
        self.addr
    }

    pub const fn metadata(&self) -> <T as U32Pointee>::Metadata {
        self.metadata
    }
}

/*impl<T: U32Pointee<Metadata=()>> U32PtrRepr<T> {
    fn get(&self) -> *const T {
        self.addr as *const T
    }
}*/

impl<T: U32Pointee<Metadata = u32> + ptr::Pointee<Metadata = usize> + ?Sized> U32PtrRepr<T> {
    pub fn get(&self) -> *const T {
        ptr::from_raw_parts(self.addr as *const (), self.metadata as usize)
    }
}

/// Flash representaion of a member
// INVARIANTS:
// - `name` has to be valid utf8
// - `pronouns` has to be valid utf8
#[repr(C)]
pub struct MemberUF2 {
    pub(crate) name: U32PtrRepr<str>,
    pub(crate) pronouns: U32PtrRepr<str>,
}

impl MemberUF2 {
    pub const ZERO: Self = Self {
        name: U32PtrRepr::from_raw_parts(0, 0),
        pronouns: U32PtrRepr::from_raw_parts(0, 0),
    };

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
    pub(crate) magic: u32,
    pub(crate) version: u16,
    pub(crate) reserved: u16,
    pub(crate) name: U32PtrRepr<str>,
    pub(crate) members: U32PtrRepr<[MemberUF2]>,
    pub(crate) reserved_bytes: [u8; 38],
    pub(crate) crc16: u16,
}

impl SystemUf2 {
    pub const ZERO: Self = Self {
        magic: MAGIC.to_le(),
        version: VERSION_1.to_le(),
        reserved: 0,
        name: U32PtrRepr::from_raw_parts(0, 0),
        members: U32PtrRepr::from_raw_parts(0, 0),
        reserved_bytes: [0; 38],
        crc16: 0,
    };

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

    // Version 1 only checks the crc16 of self, not any members or strings
    #[inline]
    fn build_crc16_1(&self) -> u16 {
        let mut crc: crc16::State<crc16::BUYPASS> = crc16::State::new();
        let bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                core::mem::size_of::<Self>() - 2,
            )
        };
        crc.update(bytes);
        crc.get()
    }

    fn build_crc16(&self) -> u16 {
        match self.version {
            VERSION_1 => self.build_crc16_1(),
            _ => unimplemented!(),
        }
    }

    fn check_crc16(&self) -> bool {
        let got = self.build_crc16();

        #[cfg(all(debug_assertions, feature = "defmt"))]
        defmt::trace!("crc16: {=u16}, got: {=u16}", self.crc16, got);

        #[cfg(debug_assertions)]
        if self.crc16 != got {
            panic!("Failed to verify crc16");
        }

        self.crc16 == got
    }

    pub fn finish(&mut self) {
        self.crc16 = self.build_crc16();
    }
}

impl System for SystemUf2 {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.name())
    }

    fn member_count(&self) -> usize {
        self.len()
    }

    fn member(&self, index: usize) -> &dyn Member {
        &self.members()[index]
    }

    fn is_valid(&self) -> bool {
        self.magic == MAGIC && self.version == VERSION_1 && self.check_crc16()
    }
}

unsafe impl Send for SystemUf2 {}
unsafe impl Sync for SystemUf2 {}
