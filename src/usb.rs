//! type enums used for USB controll

pub const VID: u16 = 0xc0de;
pub const PID: u16 = 0x1bad;

#[repr(u8)]
pub enum Request {
    ButtonPress = 0x00,
    GetSystemName = 0x01,
    GetMemberCount = 0x02,
    GetMemberName,
    GetMemberPronouns,
    GetState,
    SetState,
    UpdateDisplay,
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
            _ => Err(()),
        }
    }
}
