use alloc::boxed::Box;

use super::file::FileWriter;
use super::system_capnp::system;
use super::SystemVec;

/// Create a new system.
///
/// This returns a newly allocated system, which has to be freed using [`sb_system_free`].
#[export_name = "sb_system_new"]
pub unsafe extern "C" fn sb_system_new(
    system: *mut *mut SystemVec,
    name: *const core::ffi::c_char,
) -> core::ffi::c_int {
    let name = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(name) => name.to_string(),
        Err(_) => return -(crate::binding::StatusCode::InvalidArgument as core::ffi::c_int),
    };

    let new = SystemVec::new(name);

    let new = Box::new(new);

    unsafe { system.write(Box::leak(new)) };
    0
}

/// Return the name of the system.
///
/// This returns a newly allocated string, which has to be freed using [`sb_free_string`].
///
/// [`sb_free_string`]: crate::binding::sb_free_string
#[export_name = "sb_system_name"]
pub unsafe extern "C" fn sb_system_name(system: *const SystemVec) -> *mut core::ffi::c_char {
    let system = unsafe { &*system };
    let name = &system.name;
    let name = alloc::ffi::CString::new(name.as_str()).unwrap();
    name.into_raw()
}

#[export_name = "sb_system_member_count"]
pub unsafe extern "C" fn sb_system_member_count(system: *const SystemVec) -> usize {
    let system = unsafe { &*system };
    system.members.len()
}

#[repr(C)]
pub struct CMember {
    pub name: *const core::ffi::c_char,
    pub pronouns: *const core::ffi::c_char,
}

/// Get a member of a system.
///
/// This returns a newly allocated member, which has to be freed using [`sb_system_member_free`].
#[export_name = "sb_system_get_member"]
pub unsafe extern "C" fn sb_system_get_member(
    system: *const SystemVec,
    index: usize,
    member: *mut CMember,
) -> core::ffi::c_int {
    let system = unsafe { &*system };
    if system.members.len() <= index {
        return -(crate::binding::StatusCode::InvalidIndex as core::ffi::c_int);
    }

    let system_member = &system.members[index];
    let name = alloc::ffi::CString::new(system_member.name.as_str()).unwrap();
    let pronouns = alloc::ffi::CString::new(system_member.pronouns.as_str()).unwrap();

    unsafe {
        member.write(CMember {
            name: name.into_raw(),
            pronouns: pronouns.into_raw(),
        });
    }

    0
}

/// Push a member to a system.
///
/// This returns the new member count.
#[export_name = "sb_system_push_member"]
pub unsafe extern "C" fn sb_system_push_member(
    system: *mut SystemVec,
    member: *const CMember,
) -> core::ffi::c_int {
    let name = match unsafe { std::ffi::CStr::from_ptr((*member).name) }.to_str() {
        Ok(name) => name.to_string(),
        Err(_) => return -(crate::binding::StatusCode::InvalidArgument as core::ffi::c_int),
    };
    let pronouns = match unsafe { std::ffi::CStr::from_ptr((*member).pronouns) }.to_str() {
        Ok(pronouns) => pronouns.to_string(),
        Err(_) => return -(crate::binding::StatusCode::InvalidArgument as core::ffi::c_int),
    };
    let member = super::MemberStrings { name, pronouns };

    let system = unsafe { &mut *system };
    system.members.push(member);
    system.members.len() as core::ffi::c_int
}

/// Sort a system.
#[export_name = "sb_system_sort"]
pub unsafe extern "C" fn sb_system_sort(system: *mut SystemVec) {
    let system = unsafe { &mut *system };
    system.sort_members();
}

#[export_name = "sb_system_file_writer_new"]
pub unsafe extern "C" fn sb_system_file_writer_new(
    system: *const SystemVec,
) -> FileWriter<'static> {
    let system = unsafe { &*system };
    FileWriter::new(system)
}

/// Return file bytes.
///
/// This returns a newly allocated buffer, which has to be freed using [`sb_free_buffer`].
#[export_name = "sb_system_file_writer_bytes"]
pub unsafe extern "C" fn sb_system_file_writer_bytes(
    writer: *const FileWriter,
    buffer: *mut *mut u8,
) -> usize {
    let writer = unsafe { &*writer };
    let bytes = writer.to_vec();
    let bytes = bytes.into_boxed_slice();
    let len = bytes.len();
    unsafe { buffer.write(Box::into_raw(bytes).cast()) };
    len
}

/// Write a file.
#[export_name = "sb_system_file_writer_write"]
pub unsafe extern "C" fn sb_system_file_writer_write(
    writer: *const FileWriter,
    path: *const core::ffi::c_char,
) -> core::ffi::c_int {
    let path = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(path) => path,
        Err(_) => return -(crate::binding::StatusCode::InvalidArgument as core::ffi::c_int),
    };
    let file = match std::fs::File::create(path) {
        Ok(file) => file,
        Err(_) => return -(crate::binding::StatusCode::FailedToCreate as core::ffi::c_int),
    };

    let writer = unsafe { &*writer };
    let bytes = writer.to_vec();
    match std::io::Write::write_all(&mut std::io::BufWriter::new(file), &bytes) {
        Ok(_) => bytes.len() as core::ffi::c_int,
        Err(_) => -(crate::binding::StatusCode::FailedToWrite as core::ffi::c_int),
    }
}

/// Free a system.
#[export_name = "sb_system_free"]
pub unsafe extern "C" fn sb_system_free(system: *mut SystemVec) {
    drop(unsafe { Box::from_raw(system) })
}

/// Free a system member.
///
/// Only call this function, if you got the member from a rust function.
#[export_name = "sb_system_member_free"]
pub unsafe extern "C" fn sb_system_member_free(member: *mut CMember) {
    let _ = unsafe { alloc::ffi::CString::from_raw((*member).name as *mut _) };
    let _ = unsafe { alloc::ffi::CString::from_raw((*member).pronouns as *mut _) };
    unsafe { std::ptr::drop_in_place(member) };
}
