//! type enums used for USB controll

#[repr(u8)]
pub enum Request {
    ButtonPress = 0x00,
    GetMemberCount = 0x02,
}
