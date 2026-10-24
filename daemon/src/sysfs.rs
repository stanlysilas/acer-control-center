use std::fs;

use crate::backend::{HardwareBackend, HardwareCapabilities};

const BRIGHTNESS: &str =
    "/sys/class/leds/rgb:kbd_backlight/brightness";

const MULTI_INTENSITY: &str =
    "/sys/class/leds/rgb:kbd_backlight/multi_intensity";

pub struct SysfsBackend;

impl SysfsBackend {
    pub fn new() -> Self {
        Self
    }
}

impl HardwareBackend for SysfsBackend {
    fn capabilities(&self) -> HardwareCapabilities {
        HardwareCapabilities {
            rgb_keyboard: false,
            rgb_animations: false,
            breathing: false,
            wave: false,
        }
    }

    fn get_keyboard_brightness(&self) -> Result<u8, String> {
        let value = fs::read_to_string(BRIGHTNESS)
            .map_err(|e| e.to_string())?;

        value
            .trim()
            .parse::<u8>()
            .map_err(|e| e.to_string())
    }

    fn set_keyboard_brightness(&self, value: u8) -> Result<(), String> {
        fs::write(BRIGHTNESS, value.to_string())
            .map_err(|e| e.to_string())
    }

    fn get_keyboard_color(&self) -> Result<(u8, u8, u8), String> {
        let text = fs::read_to_string(MULTI_INTENSITY)
            .map_err(|e| e.to_string())?;

        let values: Vec<u8> = text
            .split_whitespace()
            .filter_map(|v| v.parse::<u8>().ok())
            .collect();

        if values.len() != 3 {
            return Err("Invalid RGB value".into());
        }

        Ok((values[0], values[1], values[2]))
    }

    fn set_keyboard_color(
        &self,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), String> {
        fs::write(
            MULTI_INTENSITY,
            format!("{} {} {}", r, g, b),
        )
        .map_err(|e| e.to_string())
    }

    fn get_cpu_temperature(&self) -> Result<i32, String> {
        Err("CPU temperature is not supported by SysfsBackend".into())
    }

    fn get_gpu_temperature(&self) -> Result<i32, String> {
        Err("GPU temperature is not supported by SysfsBackend".into())
    }

    fn set_fan_max(&self) -> Result<(), String> {
        Err("Maximum fan mode is not supported by SysfsBackend".into())
    }

    fn set_fan_auto(&self) -> Result<(), String> {
        Err("Automatic fan mode is not supported by SysfsBackend".into())
    }
}
