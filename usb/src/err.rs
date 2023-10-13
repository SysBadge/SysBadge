#[derive(Debug)]
pub enum Error {
    Usb(rusb::Error),
    Utf8(std::string::FromUtf8Error),
    NoDevice,
    Unaligned,
    Io(std::io::Error),
    IntEnumError(u8),
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

/*impl<T> From<sysbadge::usb::IntEnumError<T> for Error {
    fn from(err: sysbadge::usb::IntEnumError<T>) -> Self {
        Self::IntEnumError(err.value)
    }
}*/

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Usb(err) => write!(f, "USB error: {}", err),
            Self::Utf8(err) => write!(f, "UTF-8 error: {}", err),
            Self::NoDevice => write!(f, "No device found"),
            Self::Unaligned => write!(f, "Unaligned access"),
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::IntEnumError(val) => write!(f, "Int enum error {}", val),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Usb(err) => Some(err),
            Self::Utf8(err) => Some(err),
            Self::NoDevice => None,
            Self::Unaligned => None,
            Self::Io(err) => Some(err),
            Self::IntEnumError(val) => None,
        }
    }
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
