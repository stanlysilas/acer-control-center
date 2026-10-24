use std::ffi::CStr;
use std::os::raw::c_char;

use crate::ffi;

const VENDOR_SIZE: usize = 128;
const PRODUCT_SIZE: usize = 128;
const BOARD_SIZE: usize = 128;
const BOARD_VENDOR_SIZE: usize = 128;
const CHASSIS_VENDOR_SIZE: usize = 128;
const INTERFACE_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwarePlatform {
    Unknown,
    AlgAl15g53,
}

#[derive(Debug)]
pub struct HardwareInfo {
    pub supported: bool,
    pub platform: HardwarePlatform,
    pub vendor: String,
    pub product_name: String,
    pub board_name: String,
    pub board_vendor: String,
    pub chassis_vendor: String,
    pub interface_name: String,
}

fn c_string(buffer: &[c_char]) -> String {
    unsafe {
        CStr::from_ptr(buffer.as_ptr())
            .to_string_lossy()
            .into_owned()
    }
}

impl HardwareInfo {
    fn from_ffi(info: &ffi::AcerHardwareInfo) -> Self {
        let platform = match info.platform {
            1 => HardwarePlatform::AlgAl15g53,
            _ => HardwarePlatform::Unknown,
        };

        Self {
            supported: info.supported != 0,
            platform,
            vendor: c_string(&info.vendor),
            product_name: c_string(&info.product_name),
            board_name: c_string(&info.board_name),
            board_vendor: c_string(&info.board_vendor),
            chassis_vendor: c_string(&info.chassis_vendor),
            interface_name: c_string(&info.interface_name),
        }
    }
}

pub fn detect() -> Result<HardwareInfo, String> {
    let context = unsafe { ffi::acer_open() };

    if context.is_null() {
        return Err("Failed to open Acer Control Center context".to_string());
    }

    let mut info = ffi::AcerHardwareInfo {
        supported: 0,
        platform: 0,
        vendor: [0; VENDOR_SIZE],
        product_name: [0; PRODUCT_SIZE],
        board_name: [0; BOARD_SIZE],
        board_vendor: [0; BOARD_VENDOR_SIZE],
        chassis_vendor: [0; CHASSIS_VENDOR_SIZE],
        interface_name: [0; INTERFACE_SIZE],
    };

    let result = unsafe {
        ffi::acer_hardware_detect(context, &mut info)
    };

    unsafe {
        ffi::acer_close(context);
    }

    if result < 0 {
        return Err(format!("Hardware detection failed: {}", result));
    }

    Ok(HardwareInfo::from_ffi(&info))
}
