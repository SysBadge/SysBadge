#[derive(Debug)]
pub enum Error {
    Usb(rusb::Error),
    Utf8(std::string::FromUtf8Error),
    NoDevice,
}

impl From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Self {
        Self::Usb(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Utf8(err)
    }
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
