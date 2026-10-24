use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct AcerContext {
    _private: [u8; 0],
}

// #[repr(C)]
// pub struct AcerHardwareInfo {
//     pub supported: c_int,
//     pub platform: c_int,
//     pub vendor: [c_char; 128],
//     pub product_name: [c_char; 128],
//     pub board_name: [c_char; 128],
//     pub board_vendor: [c_char; 128],
//     pub chassis_vendor: [c_char; 128],
//     pub interface_name: [c_char; 32],
// }

#[link(name = "acercontrol")]
unsafe extern "C" {
    pub fn acer_open() -> *mut AcerContext;

    pub fn acer_close(ctx: *mut AcerContext);

    pub fn acercontrol_version() -> *const c_char;

    pub fn acer_driver_version(
        ctx: *mut AcerContext,
        buffer: *mut c_char,
        len: usize,
    ) -> c_int;

    pub fn acer_driver_hardware_check(
        ctx: *mut AcerContext,
        supported: *mut c_int,
    ) -> c_int;

    pub fn acer_driver_interface(
        ctx: *mut AcerContext,
        buffer: *mut c_char,
        len: usize,
    ) -> c_int;

    pub fn acer_driver_abi_major() -> c_int;

    pub fn acer_driver_abi_minor() -> c_int;

    pub fn acer_cpu_fan_rpm(
        ctx: *mut AcerContext,
        rpm: *mut c_int,
    ) -> c_int;

    pub fn acer_gpu_fan_rpm(
        ctx: *mut AcerContext,
        rpm: *mut c_int,
    ) -> c_int;

    // pub fn acer_system_fan_rpm(
    //     ctx: *mut AcerContext,
    //     rpm: *mut c_int,
    // ) -> c_int;

    // pub fn acer_hardware_detect(
    //     ctx: *mut AcerContext,
    //     info: *mut AcerHardwareInfo,
    // ) -> c_int;

    pub fn acer_cpu_temperature(
        ctx: *mut AcerContext,
        temperature: *mut c_int,
    ) -> c_int;

    pub fn acer_gpu_temperature(
        ctx: *mut AcerContext,
        temperature: *mut c_int,
    ) -> c_int;
}
