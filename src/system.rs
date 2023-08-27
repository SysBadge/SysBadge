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
}
