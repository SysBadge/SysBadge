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
