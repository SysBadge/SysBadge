/// Free a CString allocated by libSysBadge.
#[cfg(feature = "alloc")]
#[export_name = "sb_free_string"]
pub unsafe extern "C" fn sb_free_string(string: *mut core::ffi::c_char) {
    let _ = unsafe { alloc::ffi::CString::from_raw(string) };
}
