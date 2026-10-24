use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use super::context::Context;
use super::ffi;

pub fn version() -> Result<String, String> {
    let ctx = Context::new().map_err(|e| e.to_string())?;

    let mut buffer = [0 as c_char; 128];

    let ret = unsafe {
        ffi::acer_driver_version(
            ctx.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len(),
        )
    };

    if ret < 0 {
        return Err("Failed to read driver version".into());
    }

    Ok(unsafe {
        CStr::from_ptr(buffer.as_ptr())
            .to_string_lossy()
            .into_owned()
    })
}

pub fn library_version() -> Result<String, String> {
    let version = unsafe { ffi::acercontrol_version() };
    if version.is_null() {
        return Err("Library did not report a version".into());
    }

    Ok(unsafe { CStr::from_ptr(version) }
        .to_string_lossy()
        .into_owned())
}

pub fn interface() -> Result<String, String> {
    let ctx = Context::new().map_err(|e| e.to_string())?;

    let mut buffer = [0 as c_char; 128];

    let ret = unsafe {
        ffi::acer_driver_interface(
            ctx.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len(),
        )
    };

    if ret < 0 {
        return Err("Failed to read interface".into());
    }

    Ok(unsafe {
        CStr::from_ptr(buffer.as_ptr())
            .to_string_lossy()
            .into_owned()
    })
}

pub fn hardware_supported() -> Result<bool, String> {
    let ctx = Context::new().map_err(|e| e.to_string())?;

    let mut supported: c_int = 0;

    let ret = unsafe {
        ffi::acer_driver_hardware_check(
            ctx.as_ptr(),
            &mut supported,
        )
    };

    if ret < 0 {
        return Err("Hardware check failed".into());
    }

    Ok(supported != 0)
}

pub fn abi_major() -> i32 {
    unsafe { ffi::acer_driver_abi_major() }
}

pub fn abi_minor() -> i32 {
    unsafe { ffi::acer_driver_abi_minor() }
}
