use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;

use sysbadge::system::file;
use sysbadge::system::file::binding as cfile;

#[test]
fn test_file_open() {
    let path = CString::new("tests/exmpl.sysdf").unwrap();

    let mut file = MaybeUninit::<file::File>::uninit();

    // open file
    let ret = unsafe { cfile::sb_file_open(path.as_ptr(), file.as_mut_ptr()) };
    assert_eq!(ret, 0, "Failed to open file: {}", ret);

    // verify file
    assert_eq!(unsafe { cfile::sb_file_verify(file.as_ptr()) }, true);

    // reader full header
    let header = unsafe { cfile::sb_file_get_header(file.as_ptr()) };
    let header = unsafe { &*header };
    assert_eq!(header.flags, file::Flags::default());
    assert_eq!(header.version, 1);

    // read system name
    let system_name = unsafe { cfile::sb_file_system_name(file.as_ptr()) };
    let system_name = unsafe { CStr::from_ptr(system_name) };
    assert_eq!(system_name.to_str().unwrap(), "PluralKit Example System");

    // get json
    let json = unsafe { cfile::sb_file_json(file.as_ptr()) };
    assert!(!json.is_null());
    unsafe { sysbadge::binding::sb_free_string(json) };

    // manually drop the file
    unsafe { cfile::sb_file_free(file.as_mut_ptr()) };
}
