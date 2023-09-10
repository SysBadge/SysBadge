use crate::system::{Member, MemberUF2, SystemUf2, U32PtrRepr};
use crate::System;
use alloc::borrow::Cow;
use core::mem::MaybeUninit;
use core::{mem, ptr};

/// Owned system utilizing a vec to hold members.
pub struct SystemVec {
    /// Name of the system
    pub name: alloc::string::String,
    /// Vector of members
    pub members: alloc::vec::Vec<MemberStrings>,
}

impl SystemVec {
    pub fn new(name: alloc::string::String) -> Self {
        Self {
            name,
            members: alloc::vec::Vec::new(),
        }
    }
}

impl SystemVec {
    pub fn get_bin(&self, offset: u32) -> alloc::vec::Vec<u8> {
        let mut buf = alloc::vec::Vec::new();

        let mut system = SystemUf2::ZERO;
        system.name = U32PtrRepr::from_raw_parts(
            next_after::<u8>(offset + mem::size_of::<SystemUf2>() as u32),
            self.name.len() as u32,
        );
        system.members = U32PtrRepr::from_raw_parts(
            next_after::<MemberUF2>(system.name.addr() + system.name.metadata()),
            self.members.len() as u32,
        );

        buf.extend(core::iter::repeat(0).take((system.members.addr() - offset) as usize));
        unsafe {
            ptr::copy_nonoverlapping(
                &system as *const SystemUf2 as *const u8,
                buf.as_mut_ptr(),
                mem::size_of::<SystemUf2>(),
            );
            // Writing name bytes
            ptr::copy_nonoverlapping(
                self.name.as_ptr(),
                buf.as_mut_ptr().add((system.name.addr() - offset) as usize),
                self.name.len(),
            );
        }

        self.write_members(&mut buf, system.members.addr());

        buf
    }

    fn write_members(&self, buf: &mut alloc::vec::Vec<u8>, mut offset: u32) {
        let mut curr_mem_buf_offset = buf.len();
        let member_bytes = mem::size_of::<MemberUF2>() * self.members.len();
        let mut curr_str_buf_offset = buf.len() + member_bytes;
        offset += member_bytes as u32;

        buf.extend(core::iter::repeat(0).take(member_bytes));

        for member in &self.members {
            let bytes = Self::write_member(
                offset,
                curr_mem_buf_offset,
                curr_str_buf_offset,
                member,
                buf,
            );
            curr_str_buf_offset += bytes as usize;
            offset += bytes;

            curr_mem_buf_offset += mem::size_of::<MemberUF2>();
        }
    }

    fn write_member(
        offset: u32,
        mem_buf_offset: usize,
        str_buf_offset: usize,
        member: &MemberStrings,
        buf: &mut alloc::vec::Vec<u8>,
    ) -> u32 {
        let name = member.name();
        let name_len = name.len();
        let pronouns = member.pronouns();
        let pronouns_len = pronouns.len();

        let mut bytes = name_len + pronouns_len;
        buf.extend(core::iter::repeat(0).take(bytes));

        let mut member = MemberUF2::ZERO;
        member.name = U32PtrRepr::from_raw_parts(offset, name_len as u32);
        member.pronouns = U32PtrRepr::from_raw_parts(offset + name_len as u32, pronouns_len as u32);

        // write member info
        unsafe {
            ptr::copy_nonoverlapping(
                &member as *const MemberUF2 as *const u8,
                buf.as_mut_ptr().add(mem_buf_offset),
                mem::size_of::<MemberUF2>(),
            );
        }

        // write strings
        unsafe {
            ptr::copy_nonoverlapping(
                name.as_ptr(),
                buf.as_mut_ptr().add(str_buf_offset),
                name_len,
            );
            ptr::copy_nonoverlapping(
                pronouns.as_ptr(),
                buf.as_mut_ptr().add(str_buf_offset + name_len),
                pronouns_len,
            );
        }

        return bytes as u32;
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
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
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
