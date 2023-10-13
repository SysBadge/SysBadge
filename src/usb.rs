//! type enums used for USB controll

use int_enum::IntEnum;

pub const VID: u16 = 0x33ff;
pub const PID: u16 = 0x4025;

pub type IntEnumError<T> = int_enum::IntEnumError<T>;

/// USB Request types
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum Request {
    /// Request to press a button
    ButtonPress = 0x00,
    /// Request the currently loaded system name
    GetSystemName = 0x01,
    /// Request the currently loaded system member count
    GetMemberCount = 0x02,
    /// Request the name of a member of the currently loaded system
    GetMemberName = 0x03,
    /// Request the pronouns of a member of the currently loaded system
    GetMemberPronouns = 0x04,
    GetState = 0x05,
    SetState = 0x06,
    /// Request the SysBadge to update its display
    UpdateDisplay = 0x07,
    /// Request the SysBadge to return its version
    GetVersion = 0x08,
    /// Request the SysBadge to reboot
    Reboot = 0x09,
    /// Request the SysBadge to enter update mode
    SystemUpload = 0x0A,
    /// Upload a system chunk to the SysBadge
    SystemDNLoad = 0x0B,
}

/// Version types
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum, Default)]
pub enum VersionType {
    /// Request the SemVer version of the SysBadge.
    #[default]
    SemVer = 0x10,
}

/// Reboot selection
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum, Default)]
pub enum BootSel {
    /// Boot the SysBadge into the application
    #[default]
    Application = 0x00,
    /// Boot the SysBadge into the bootloader
    Bootloader = 0x01,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum, Default)]
pub enum SystemUpdateStatus {
    /// The SysBadge is not in update mode.
    #[default]
    NotInUpdateMode = 0x00,
    /// Currently erasing
    Erasing = 0x01,
    /// Erased and ready for update
    ErasedForUpdate = 0x02,
    /// Ready for update
    ReadyForUpdate = 0x03,
    /// Wirint a chung
    Writing = 0x04,
    /// Chunk written
    Written = 0x05,
    /// Error while erasing
    EraseError = 0x06,
    /// Error while writing
    WriteError = 0x07,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum, Default)]
pub enum SystemIdType {
    /// The file was not generated via a know provider
    #[default]
    None = 0x00,
    /// The file was generated with data from PluralKit
    PluralKit = 0x01,
    /// The file was generated with data from PronounsCC
    PronounsCC = 0x02,
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum SystemId {
    None,
    PluralKit(alloc::string::String),
    PronounsCC(alloc::string::String),
}

#[cfg(feature = "alloc")]
impl SystemId {
    #[inline]
    pub fn new(id: SystemIdType, str: alloc::string::String) -> Self {
        match id {
            SystemIdType::None => SystemId::None,
            SystemIdType::PluralKit => SystemId::PluralKit(str),
            SystemIdType::PronounsCC => SystemId::PronounsCC(str),
        }
    }
}

#[cfg(feature = "alloc")]
impl AsRef<SystemIdType> for SystemId {
    fn as_ref(&self) -> &SystemIdType {
        match self {
            SystemId::None => &SystemIdType::None,
            SystemId::PluralKit(_) => &SystemIdType::PluralKit,
            SystemId::PronounsCC(_) => &SystemIdType::PronounsCC,
        }
    }
}

#[cfg(feature = "alloc")]
impl From<SystemId> for SystemIdType {
    fn from(id: SystemId) -> Self {
        *id.as_ref()
    }
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for SystemId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SystemId::None => write!(f, "None"),
            SystemId::PluralKit(s) => write!(f, "PluralKit: {}", s),
            SystemId::PronounsCC(s) => write!(f, "PronounsCC: {}", s),
        }
    }
}
