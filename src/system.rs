use core::ffi::CStr;

/// Flash representaion of a member
// INVARIANTS:
// - `name` has to be valid utf8 and null terminated
// - `pronouns` has to be valid utf8 and null terminated
#[repr(C)]
pub struct Member {
    name: [u8; 52],
    pronouns: [u8; 20],
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
