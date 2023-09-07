//! type enums used for USB controll

pub const VID: u16 = 0x33ff;
pub const PID: u16 = 0x4025;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    ButtonPress = 0x00,
    GetSystemName = 0x01,
    GetMemberCount = 0x02,
    GetMemberName,
    GetMemberPronouns,
    GetState,
    SetState,
    UpdateDisplay,
    GetVersion,
}

impl TryFrom<u8> for Request {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == (Request::ButtonPress as u8) => Ok(Request::ButtonPress),
            x if x == (Request::GetSystemName as u8) => Ok(Request::GetSystemName),
            x if x == (Request::GetMemberCount as u8) => Ok(Request::GetMemberCount),
            x if x == (Request::GetMemberName as u8) => Ok(Request::GetMemberName),
            x if x == (Request::GetMemberPronouns as u8) => Ok(Request::GetMemberPronouns),
            x if x == (Request::GetState as u8) => Ok(Request::GetState),
            x if x == (Request::SetState as u8) => Ok(Request::SetState),
            x if x == (Request::UpdateDisplay as u8) => Ok(Request::UpdateDisplay),
            x if x == (Request::GetVersion as u8) => Ok(Request::GetVersion),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionType {
    SemVer = 0x01,
    Matrix = 0x10,
    Web,
}

impl TryFrom<u8> for VersionType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == (VersionType::SemVer as u8) => Ok(VersionType::SemVer),
            x if x == (VersionType::Matrix as u8) => Ok(VersionType::Matrix),
            x if x == (VersionType::Web as u8) => Ok(VersionType::Web),
            _ => Err(()),
        }
    }
}
