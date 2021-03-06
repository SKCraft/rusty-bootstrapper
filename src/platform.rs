use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HomeDirError {
    #[error("Home directory does not have a path")]
    NoPath,
    #[error("Home directory is not present")]
    NotPresent,
    #[error("Unknown problem while getting home directory")]
    Unknown,
}

#[cfg(target_os = "windows")]
pub fn home_dir() -> Result<PathBuf, HomeDirError> {
    use std::ffi::OsString;
    use std::os::raw::c_char;
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::ffi::CStrExt;
    use winapi::_core::ptr::{null, null_mut};
    use winapi::shared::winerror::{S_OK, E_FAIL, E_INVALIDARG};
    use winapi::ctypes::c_void;

    unsafe {
        let mut result_ptr: *mut u16 = null_mut();
        let success = winapi::um::shlobj::SHGetKnownFolderPath(
            &winapi::um::knownfolders::FOLDERID_Documents,
            0, null_mut(), &mut result_ptr);

        let res = match success {
            S_OK => {
                let len = (0..).take_while(|&i| *result_ptr.offset(i) != 0)
                    .count();
                let slice = std::slice::from_raw_parts(result_ptr, len);

                let path = OsString::from_wide(slice);
                Ok(PathBuf::from(path))
            },
            E_FAIL => Err(HomeDirError::NoPath),
            E_INVALIDARG => Err(HomeDirError::NotPresent),
            _ => Err(HomeDirError::Unknown)
        };

        winapi::um::combaseapi::CoTaskMemFree(result_ptr as *mut c_void);
        res
    }
}

#[cfg(target_os = "linux")]
pub fn home_dir() -> Result<PathBuf, HomeDirError> {
    use std::ffi::{CStr, OsStr};
    use std::os::unix::ffi::OsStrExt;

    if let Some(path) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(path))
    }

    unsafe {
        let pw = libc::getpwuid(libc::getuid());

        if !pw.is_null() {
            let path_c = CStr::from_ptr((*pw).pw_dir);
            let path = OsStr::from_bytes(path_c.to_bytes());

            return Ok(PathBuf::from(path))
        }
    }

    Err(HomeDirError::NoPath)
}

#[cfg(target_os = "macos")]
pub fn home_dir() -> Result<PathBuf, HomeDirError> {
    use std::ffi::{CStr, OsStr};
    use objc::{class, msg_send};
    use objc::runtime::Object;

    let cls = class!(NSFileManager);
    let obj: *mut Object = msg_send![cls, new];
    let nss: *mut Object = msg_send![obj, NSHomeDirectory];
    let cs_ptr: *const c_char = msg_send![nss, UTF8String];

    let path = unsafe { CStr::from_ptr(cs_ptr).to_owned() };
    let path_os = OsStr::from_bytes(path.to_bytes());

    Ok(PathBuf::from(path_os))
}
