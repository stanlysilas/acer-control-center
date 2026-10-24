use crate::animation::HardwareAnimationBackend;
use crate::backend::{HardwareBackend, HardwareCapabilities};
use crate::ffi;
use crate::state::{StateStore, UserState};
use crate::sysfs::SysfsBackend;

use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct AlgAl15g53Backend {
    inner: SysfsBackend,
    context: *mut ffi::AcerContext,
    current_color: Arc<Mutex<(u8, u8, u8)>>,
    animation_running: Arc<AtomicBool>,
    saved_brightness: AtomicU8,
    fan_max: AtomicBool,
    animation_thread: Mutex<Option<JoinHandle<()>>>,
    state: StateStore,
}

impl AlgAl15g53Backend {
    pub fn new() -> Result<Self, String> {
        let context = unsafe { ffi::acer_open() };

        if context.is_null() {
            return Err("Failed to open Acer Control Center context".into());
        }

        let inner = SysfsBackend::new();
        let state = StateStore::new()?;

        let color = inner
            .get_keyboard_color()
            .unwrap_or((255, 255, 255));

        let brightness = inner
            .get_keyboard_brightness()
            .unwrap_or(100);

        let saved = match state.load() {
            Ok(saved) => saved,
            Err(error) => {
                eprintln!("Could not restore saved preferences: {error}");
                None
            }
        };
        let restored = saved.unwrap_or_else(|| UserState::defaults(color.0, color.1, color.2, brightness));

        if saved.is_some() {
            let _ = inner.set_keyboard_color(restored.r, restored.g, restored.b);
            let _ = inner.set_keyboard_brightness(restored.brightness);
        }

        let backend = Self {
            inner,
            context,
            current_color: Arc::new(Mutex::new((restored.r, restored.g, restored.b))),
            animation_running: Arc::new(AtomicBool::new(false)),
            saved_brightness: AtomicU8::new(restored.brightness),
            fan_max: AtomicBool::new(restored.fan_max),
            animation_thread: Mutex::new(None),
            state,
        };

        if saved.is_some() {
            let fan_result = if restored.fan_max {
                backend.set_fan_max()
            } else {
                backend.set_fan_auto()
            };
            if let Err(error) = fan_result {
                eprintln!("Could not restore the saved fan mode: {error}");
            }
        }
        if restored.breathing {
            if let Err(error) = backend.start_breathing() {
                eprintln!("Could not restore the saved breathing effect: {error}");
            }
        }

        Ok(backend)
    }

    fn stop_animation_internal(&self) {
        self.animation_running
            .store(false, Ordering::SeqCst);

        if let Ok(mut handle) = self.animation_thread.lock() {
            if let Some(thread) = handle.take() {
                let _ = thread.join();
            }
        }
    }

    fn save_current_state(&self) -> Result<(), String> {
        let color = self
            .current_color
            .lock()
            .map_err(|_| {
                "Failed to lock keyboard color state".to_string()
            })?;

        self.state.save(UserState {
            r: color.0,
            g: color.1,
            b: color.2,
            brightness: self.saved_brightness.load(Ordering::Relaxed),
            fan_max: self.fan_max.load(Ordering::Relaxed),
            breathing: self.animation_running.load(Ordering::Relaxed),
        })
    }

    fn scale_color(
        color: (u8, u8, u8),
        intensity: u8,
    ) -> (u8, u8, u8) {
        let scale = intensity as u16;

        (
            ((color.0 as u16 * scale) / 100) as u8,
            ((color.1 as u16 * scale) / 100) as u8,
            ((color.2 as u16 * scale) / 100) as u8,
        )
    }
}

impl Drop for AlgAl15g53Backend {
    fn drop(&mut self) {
        self.animation_running
            .store(false, Ordering::SeqCst);

        if let Ok(mut handle) = self.animation_thread.lock() {
            if let Some(thread) = handle.take() {
                let _ = thread.join();
            }
        }

        if !self.context.is_null() {
            unsafe {
                ffi::acer_close(self.context);
            }
        }
    }
}

unsafe impl Send for AlgAl15g53Backend {}
unsafe impl Sync for AlgAl15g53Backend {}

impl Default for AlgAl15g53Backend {
    fn default() -> Self {
        Self::new()
            .expect("Failed to initialize ALG AL15G-53 backend")
    }
}

impl HardwareBackend for AlgAl15g53Backend {
    fn capabilities(&self) -> HardwareCapabilities {
        HardwareCapabilities {
            rgb_keyboard: true,
            rgb_animations: true,
            breathing: true,
            wave: false,
        }
    }

    fn get_keyboard_brightness(&self) -> Result<u8, String> {
        Ok(self.saved_brightness.load(Ordering::Relaxed))
    }

    fn set_keyboard_brightness(
        &self,
        value: u8,
    ) -> Result<(), String> {
        self.stop_animation_internal();

        let color = self
            .current_color
            .lock()
            .map_err(|_| "Failed to lock keyboard color state".to_string())?;
        self.inner.set_keyboard_color(color.0, color.1, color.2)?;
        drop(color);

        self.inner.set_keyboard_brightness(value)?;

        self.saved_brightness.store(value, Ordering::Relaxed);

        self.save_current_state()
    }

    fn set_keyboard_brightness_transient(
        &self,
        value: u8,
    ) -> Result<(), String> {
        self.inner.set_keyboard_brightness(value)
    }

    fn get_keyboard_color(
        &self,
    ) -> Result<(u8, u8, u8), String> {
        self.current_color
            .lock()
            .map(|color| *color)
            .map_err(|_| "Failed to lock keyboard color state".to_string())
    }

    fn set_keyboard_color(
        &self,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), String> {
        self.stop_animation_internal();

        self.inner.set_keyboard_color(r, g, b)?;

        {
            let mut color = self
                .current_color
                .lock()
                .map_err(|_| {
                    "Failed to lock keyboard color state"
                        .to_string()
                })?;

            *color = (r, g, b);
        }

        self.save_current_state()
    }

    fn get_cpu_temperature(&self) -> Result<i32, String> {
        let mut temperature: c_int = 0;

        let result = unsafe {
            ffi::acer_cpu_temperature(
                self.context,
                &mut temperature,
            )
        };

        if result < 0 {
            return Err(
                "Failed to read CPU temperature".into()
            );
        }

        Ok(temperature)
    }

    fn get_gpu_temperature(&self) -> Result<i32, String> {
        let mut temperature: c_int = 0;

        let result = unsafe {
            ffi::acer_gpu_temperature(
                self.context,
                &mut temperature,
            )
        };

        if result < 0 {
            return Err(
                "Failed to read GPU temperature".into()
            );
        }

        Ok(temperature)
    }

    fn set_fan_max(&self) -> Result<(), String> {
        let result = unsafe {
            ffi::acer_fan_max(self.context)
        };

        if result < 0 {
            return Err(
                "Failed to set maximum fan mode".into()
            );
        }

        self.fan_max.store(true, Ordering::Relaxed);
        self.save_current_state()
    }

    fn set_fan_auto(&self) -> Result<(), String> {
        let result = unsafe {
            ffi::acer_fan_auto(self.context)
        };

        if result < 0 {
            return Err(
                "Failed to restore automatic fan mode".into()
            );
        }

        self.fan_max.store(false, Ordering::Relaxed);
        self.save_current_state()
    }

    fn fan_max_selected(&self) -> bool {
        self.fan_max.load(Ordering::Relaxed)
    }
}

impl HardwareAnimationBackend for AlgAl15g53Backend {
    fn start_breathing(&self) -> Result<(), String> {
        self.stop_animation_internal();

        let mut animation_thread = self
            .animation_thread
            .lock()
            .map_err(|_| "Failed to lock animation state".to_string())?;
        self.animation_running.store(true, Ordering::SeqCst);
        if let Err(error) = self.save_current_state() {
            self.animation_running.store(false, Ordering::SeqCst);
            return Err(error);
        }

        let running =
            Arc::clone(&self.animation_running);

        let color =
            Arc::clone(&self.current_color);

        let inner = SysfsBackend::new();

        running.store(true, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            let mut intensity: i32 = 255;
            let mut decreasing = true;

            while running.load(Ordering::SeqCst) {
                let base_color = match color.lock() {
                    Ok(value) => *value,
                    Err(_) => break,
                };

                let animated_color =
                    Self::scale_color(
                        base_color,
                        ((intensity * 100) / 255) as u8,
                    );

                if inner
                    .set_keyboard_color(
                        animated_color.0,
                        animated_color.1,
                        animated_color.2,
                    )
                    .is_err()
                {
                    break;
                }

                if decreasing {
                    intensity -= 8;

                    if intensity <= 0 {
                        intensity = 0;
                        decreasing = false;
                    }
                } else {
                    intensity += 8;

                    if intensity >= 255 {
                        intensity = 255;
                        decreasing = true;
                    }
                }

                thread::sleep(Duration::from_millis(25));
            }
        });

        *animation_thread = Some(handle);

        Ok(())
    }

    fn stop_animation(&self) -> Result<(), String> {
        self.stop_animation_internal();

        let color = self
            .current_color
            .lock()
            .map_err(|_| {
                "Failed to lock keyboard color state"
                    .to_string()
            })?;

        self.inner.set_keyboard_color(
            color.0,
            color.1,
            color.2,
        )?;
        drop(color);

        self.save_current_state()
    }

    fn breathing_selected(&self) -> bool {
        self.animation_running.load(Ordering::Relaxed)
    }
}
