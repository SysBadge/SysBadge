use alloc::ffi::CString;

use super::{GenericDownloader, Source};
use crate::system::SystemVec;

/// Create a new generic downloader.
#[export_name = "sb_downloader_new"]
pub unsafe extern "C" fn sb_downloader_generic_new(
    downloader: *mut *mut GenericDownloader,
    useragent: *const core::ffi::c_char,
) -> core::ffi::c_int {
    let useragent = if useragent.is_null() {
        None
    } else {
        match unsafe { std::ffi::CStr::from_ptr(useragent) }.to_str() {
            Ok(useragent) => Some(useragent),
            Err(_) => return -(crate::binding::StatusCode::InvalidArgument as core::ffi::c_int),
        }
    };

    let new = match useragent {
        Some(useragent) => GenericDownloader::new_with_useragent(useragent),
        None => GenericDownloader::new(),
    };

    let new = Box::new(new);
    unsafe { downloader.write(Box::leak(new)) };
    0
}

/// Get the useragent of a generic downloader.
///
/// This returns a newly allocated string, which has to be freed using [`sb_free_string`].
///
/// [`sb_free_string`]: crate::binding::sb_free_string
#[export_name = "sb_downloader_useragent"]
pub unsafe extern "C" fn sb_downloader_generic_useragent(
    downloader: *const GenericDownloader,
) -> *mut core::ffi::c_char {
    let downloader: &GenericDownloader = unsafe { &*downloader };
    let useragent = &downloader.useragent;
    let useragent = CString::new(useragent.as_str()).unwrap();
    useragent.into_raw()
}

/// Get a system from a generic downloader.
///
/// This returns a newly allocated system, which has to be freed using [`sb_system_free`].
#[export_name = "sb_downloader_get"]
#[cfg(feature = "tokio")]
pub unsafe extern "C" fn sb_downloader_get(
    downloader: *const GenericDownloader,
    source: Source,
    id: *const core::ffi::c_char,
    system: *mut *mut SystemVec,
) -> core::ffi::c_int {
    let id = match unsafe { std::ffi::CStr::from_ptr(id) }.to_str() {
        Ok(id) => id,
        Err(_) => return -(crate::binding::StatusCode::InvalidArgument as core::ffi::c_int),
    };

    let downloader = unsafe { &*downloader };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let system_new = match rt.block_on(downloader.get(source, id)) {
        Ok(system) => system,
        Err(_) => return -(crate::binding::StatusCode::FailedToWrite as core::ffi::c_int),
    };

    let system_new = Box::new(system_new);
    unsafe { system.write(Box::leak(system_new)) };
    0
}

/// Set the useragent of a generic downloader.
#[export_name = "sb_downloader_set_useragent"]
pub unsafe extern "C" fn sb_downloader_generic_set_useragent(
    downloader: *mut GenericDownloader,
    useragent: *const core::ffi::c_char,
) -> core::ffi::c_int {
    let useragent = match unsafe { std::ffi::CStr::from_ptr(useragent) }.to_str() {
        Ok(useragent) => useragent,
        Err(_) => return -(crate::binding::StatusCode::InvalidArgument as core::ffi::c_int),
    };

    let downloader = unsafe { &mut *downloader };
    downloader.useragent = useragent.to_string();
    0
}

/// Free a generic downloader.
#[export_name = "sb_downloader_free"]
pub unsafe extern "C" fn sb_downloader_generic_free(downloader: *mut GenericDownloader) {
    drop(unsafe { Box::from_raw(downloader) });
}
