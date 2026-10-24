#[derive(Debug, Clone, Copy)]
pub struct HardwareCapabilities {
    pub rgb_keyboard: bool,
    pub rgb_animations: bool,
    pub breathing: bool,
    pub wave: bool,
}

pub trait HardwareBackend: Send + Sync {
    fn capabilities(&self) -> HardwareCapabilities;

    fn get_keyboard_brightness(&self) -> Result<u8, String>;
    fn set_keyboard_brightness(&self, value: u8) -> Result<(), String>;
    fn set_keyboard_brightness_transient(&self, value: u8) -> Result<(), String> {
        self.set_keyboard_brightness(value)
    }

    fn get_keyboard_color(&self) -> Result<(u8, u8, u8), String>;
    fn set_keyboard_color(&self, r: u8, g: u8, b: u8) -> Result<(), String>;

    fn get_cpu_temperature(&self) -> Result<i32, String>;
    fn get_gpu_temperature(&self) -> Result<i32, String>;

    fn set_fan_max(&self) -> Result<(), String>;
    fn set_fan_auto(&self) -> Result<(), String>;
    fn fan_max_selected(&self) -> bool {
        false
    }
}
