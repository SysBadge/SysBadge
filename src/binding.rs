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
    let _ = unsafe {
        alloc::boxed::Box::from_raw(core::ptr::from_raw_parts_mut(buffer.cast(), len) as *mut [u8])
    };
}
