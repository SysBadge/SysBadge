#[cfg(feature = "alloc")]
use alloc::string::ToString;

/// Free a CString allocated by libSysBadge.
#[cfg(feature = "alloc")]
#[export_name = "sb_free_string"]
pub unsafe extern "C" fn sb_free_string(string: *mut core::ffi::c_char) {
    let _ = unsafe { alloc::ffi::CString::from_raw(string) };
}

/// Free a buffer allocated by libSysBadge.
#[cfg(feature = "alloc")]
#[export_name = "sb_free_buffer"]
pub unsafe extern "C" fn sb_free_buffer(buffer: *mut u8, len: usize) {
    drop(unsafe {
        alloc::boxed::Box::from_raw(core::ptr::from_raw_parts_mut(buffer.cast(), len) as *mut [u8])
    });
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok = 0,
    InvalidArgument = 100,
    InvalidIndex,
    FailedToCreate,
    FailedToWrite,
}

impl core::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StatusCode::Ok => write!(f, "Ok"),
            StatusCode::InvalidArgument => write!(f, "InvalidArgument"),
            StatusCode::InvalidIndex => write!(f, "InvalidIndex"),
            StatusCode::FailedToCreate => write!(f, "FailedToCreate"),
            StatusCode::FailedToWrite => write!(f, "FailedToWrite"),
        }
    }
}

/// Convert status code into an error debug string.
///
/// This returns a newly allocated string, which has to be freed using [`sb_free_string`].
#[cfg(feature = "alloc")]
#[export_name = "sb_status_code_debug_string"]
pub unsafe extern "C" fn sb_status_code_debug_string(code: StatusCode) -> *mut core::ffi::c_char {
    let str = alloc::ffi::CString::new(alloc::format!("{:?}", code)).unwrap();
    str.into_raw()
}

/// Convert status code into an error string..
///
/// This returns a newly allocated string, which has to be freed using [`sb_free_string`].
#[cfg(feature = "alloc")]
#[export_name = "sb_status_code_string"]
pub unsafe extern "C" fn sb_status_code_string(code: StatusCode) -> *mut core::ffi::c_char {
    let str = alloc::ffi::CString::new(code.to_string()).unwrap();
    str.into_raw()
}
