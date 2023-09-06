#[derive(Debug)]
pub enum Error {
    Usb(rusb::Error),
    Utf8(std::string::FromUtf8Error),
    NoDevice,
    Io(std::io::Error),
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, F: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Usb(err) => write!(F, "USB error: {}", err),
            Self::Utf8(err) => write!(F, "UTF-8 error: {}", err),
            Self::NoDevice => write!(F, "No device found"),
            Self::Io(err) => write!(F, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Usb(err) => Some(err),
            Self::Utf8(err) => Some(err),
            Self::NoDevice => None,
            Self::Io(err) => Some(err),
        }
    }
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
