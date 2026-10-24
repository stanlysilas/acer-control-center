use std::os::raw::c_int;

use super::context::Context;
use super::ffi;

pub fn cpu_fan_rpm() -> Result<i32, String> {
    let ctx = Context::new().map_err(|e| e.to_string())?;

    let mut rpm: c_int = 0;

    let ret = unsafe {
        ffi::acer_cpu_fan_rpm(
            ctx.as_ptr(),
            &mut rpm,
        )
    };

    if ret < 0 {
        return Err("Failed to read CPU fan RPM".into());
    }

    Ok(rpm)
}

pub fn gpu_fan_rpm() -> Result<i32, String> {
    let ctx = Context::new().map_err(|e| e.to_string())?;

    let mut rpm: c_int = 0;

    let ret = unsafe {
        ffi::acer_gpu_fan_rpm(
            ctx.as_ptr(),
            &mut rpm,
        )
    };

    if ret < 0 {
        return Err("Failed to read GPU fan RPM".into());
    }

    Ok(rpm)
}

// pub fn system_fan_rpm() -> Result<i32, String> {
//     let ctx = Context::new().map_err(|e| e.to_string())?;

//     let mut rpm: c_int = 0;

//     let ret = unsafe {
//         ffi::acer_system_fan_rpm(
//             ctx.as_ptr(),
//             &mut rpm,
//         )
//     };

//     if ret < 0 {
//         return Err("Failed to read System fan RPM".into());
//     }

//     Ok(rpm)
// }

pub fn cpu_temperature() -> Result<i32, String> {
    let ctx = Context::new().map_err(|e| e.to_string())?;

    let mut temperature: c_int = 0;

    let ret = unsafe {
        ffi::acer_cpu_temperature(
            ctx.as_ptr(),
            &mut temperature,
        )
    };

    if ret < 0 {
        return Err("Failed to read CPU temperature".into());
    }

    Ok(temperature)
}

pub fn gpu_temperature() -> Result<i32, String> {
    let ctx = Context::new().map_err(|e| e.to_string())?;

    let mut temperature: c_int = 0;

    let ret = unsafe {
        ffi::acer_gpu_temperature(
            ctx.as_ptr(),
            &mut temperature,
        )
    };

    if ret < 0 {
        return Err("Failed to read GPU temperature".into());
    }

    Ok(temperature)
}
