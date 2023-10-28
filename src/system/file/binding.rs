use super::{File, FileHeader};

/// Open a SysBadge system definition file.
///
/// This creates a new `File` object from a file on disk.
///
/// Has to be freed using [`sb_file_free`].
#[cfg(feature = "std")]
#[export_name = "sb_file_open"]
pub unsafe extern "C" fn sb_file_open(
    path: *const core::ffi::c_char,
    out_file: *mut File,
) -> core::ffi::c_int {
    let path = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(path) => path,
        Err(_) => return -libc::EINVAL,
    };

    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => match e.raw_os_error() {
            Some(e) => return -e,
            None => return -libc::EINVAL,
        },
    };

    let file = match File::read(&mut std::io::BufReader::new(file)) {
        Ok(Some(file)) => file,
        Ok(_) => return -libc::EINVAL,
        Err(_) => return -libc::EINVAL,
    };

    unsafe { out_file.write(file) };

    0
}

/// Get the header of a SysBadge system definition file.
///
/// This only returns a reference, which is valid as long as the file is not freed.
#[export_name = "sb_file_get_header"]
pub unsafe extern "C" fn sb_file_get_header(file: *const File) -> *const FileHeader {
    let file = unsafe { &*file };
    &file.header as *const FileHeader
}

/// Get the system name of a SysBadge system definition file.
///
/// This returns a pointer to the internal string, which is valid as long as the file is not freed.
#[export_name = "sb_file_system_name"]
pub unsafe extern "C" fn sb_file_system_name(file: *const File) -> *const core::ffi::c_char {
    let file = unsafe { &*file };
    let name = &file.header.system_name;
    name.as_ptr() as *const core::ffi::c_char
}

/// Get the json blob of a SysBadge system definition file.
///
/// This returns a newly allocated string, which has to be freed using [`sb_free_string`].
///
/// [`sb_free_string`]: crate::binding::sb_free_string
#[export_name = "sb_file_json"]
pub unsafe extern "C" fn sb_file_json(file: *const File) -> *mut core::ffi::c_char {
    let file = unsafe { &*file };

    if !file.header.flags.contains(super::Flags::JSON_BLOB) {
        #[cfg(feature = "tracing")]
        tracing::warn!("File does not contain a JSON blob");
        return core::ptr::null_mut();
    }

    let json = match &file.json {
        Some(json) => json,
        None => return core::ptr::null_mut(),
    };

    let json = match std::str::from_utf8(json) {
        Ok(json) => json,
        Err(_) => return core::ptr::null_mut(),
    };

    let json = match alloc::ffi::CString::new(json) {
        Ok(json) => json,
        Err(_) => return core::ptr::null_mut(),
    };

    json.into_raw()
}

/// Verify file checksum if available.
#[export_name = "sb_file_verify"]
pub unsafe extern "C" fn sb_file_verify(file: *const File) -> bool {
    let file = unsafe { &*file };
    file.verify()
}

/// Free a SysBadge system definition file.
#[export_name = "sb_file_free"]
pub unsafe extern "C" fn sb_file_free(file: *mut File) {
    unsafe { core::ptr::drop_in_place(file) };
}