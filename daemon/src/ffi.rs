use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct AcerContext {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AcerHardwareInfo {
    pub supported: c_int,
    pub platform: c_int,
    pub vendor: [c_char; 128],
    pub product_name: [c_char; 128],
    pub board_name: [c_char; 128],
    pub board_vendor: [c_char; 128],
    pub chassis_vendor: [c_char; 128],
    pub interface_name: [c_char; 32],
}

#[link(name = "acercontrol")]
unsafe extern "C" {
    pub fn acer_open() -> *mut AcerContext;
    pub fn acer_close(ctx: *mut AcerContext);
    pub fn acer_fan_max(ctx: *mut AcerContext) -> c_int;
    pub fn acer_fan_auto(ctx: *mut AcerContext) -> c_int;
    pub fn acer_cpu_temperature(
        ctx: *mut AcerContext,
        temperature: *mut c_int,
    ) -> c_int;

    pub fn acer_gpu_temperature(
        ctx: *mut AcerContext,
        temperature: *mut c_int,
    ) -> c_int;

    pub fn acer_hardware_detect(
        ctx: *mut AcerContext,
        info: *mut AcerHardwareInfo,
    ) -> c_int;
}
