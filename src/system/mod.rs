mod uf2;
pub use uf2::*;

use alloc::borrow::Cow;
use core::mem::MaybeUninit;
use core::{mem, ptr};

pub trait Member {
    fn name(&self) -> &str;
    fn pronouns(&self) -> &str;
}

impl<M: Member> Member for &M {
    fn name(&self) -> &str {
        (*self).name()
    }

    fn pronouns(&self) -> &str {
        (*self).pronouns()
    }
}

pub trait System {
    fn name(&self) -> Cow<'_, str>;
    fn member_count(&self) -> usize;
    fn member(&self, index: usize) -> &dyn Member;
}

impl<S: System> System for &S {
    fn name(&self) -> Cow<'_, str> {
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
