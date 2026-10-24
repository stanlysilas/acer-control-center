use std::sync::atomic::{AtomicBool, Ordering};

use ksni::blocking::{Handle, TrayMethods};
use ksni::{Icon, MenuItem, Tray};
use crate::{instance, keyboard_idle, libacercontrol::dbus::AcerControlDBus};
use std::time::Duration;

static ACTIVATE_REQUESTED: AtomicBool = AtomicBool::new(false);
static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
struct ControlCenterTray;

impl Tray for ControlCenterTray {
    fn id(&self) -> String {
        "acer-control-center".to_owned()
    }

    fn title(&self) -> String {
        "Acer Control Center".to_owned()
    }

    fn icon_name(&self) -> String {
        "acer-control-center".to_owned()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let Ok(image) = image::load_from_memory(include_bytes!("../resources/icons/acer-control-center.png")) else {
            return Vec::new();
        };
        let rgba = image.resize_exact(64, 64, image::imageops::FilterType::Lanczos3).to_rgba8();
        let mut argb = Vec::with_capacity((64 * 64 * 4) as usize);
        for pixel in rgba.pixels() {
            let [red, green, blue, alpha] = pixel.0;
            argb.extend_from_slice(&[alpha, red, green, blue]);
        }
        vec![Icon { width: 64, height: 64, data: argb }]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        ACTIVATE_REQUESTED.store(true, Ordering::Release);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        use ksni::menu::StandardItem;

        vec![
            StandardItem {
                label: "Open Acer Control Center".to_owned(),
                icon_name: "acer-control-center".to_owned(),
                activate: Box::new(|_| {
                    ACTIVATE_REQUESTED.store(true, Ordering::Release);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".to_owned(),
                icon_name: "application-exit".to_owned(),
                activate: Box::new(|_| {
                    QUIT_REQUESTED.store(true, Ordering::Release);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub struct TrayController {
    _handle: Handle<ControlCenterTray>,
}

impl TrayController {
    pub fn start() -> Result<Self, String> {
        let handle = ControlCenterTray
            .spawn()
            .map_err(|error| format!("Could not create system tray icon: {error}"))?;
        Ok(Self { _handle: handle })
    }
}

pub fn run_background_service() -> Result<(), String> {
    let _tray = TrayController::start()?;
    if !instance::ensure_single_instance(true)? {
        return Ok(());
    }

    let initial_brightness = AcerControlDBus::new()
        .and_then(|bus| bus.keyboard_brightness())
        .unwrap_or(128);
    let idle = keyboard_idle::KeyboardIdleController::new(
        keyboard_idle::load_timeout(),
        initial_brightness,
    );
    let mut last_timeout = keyboard_idle::load_timeout();
    let mut last_brightness = initial_brightness;

    loop {
        if instance::take_background_activation_request() || take_activation_request() {
            if let Err(error) = instance::activate_or_launch_window() {
                log::warn!("Could not activate the control center window: {error}");
            }
        }
        if instance::take_background_quit_request() || take_quit_request() {
            instance::request_window_quit();
            break;
        }

        let timeout = keyboard_idle::load_timeout();
        if timeout != last_timeout {
            idle.set_timeout(timeout);
            last_timeout = timeout;
        }
        if let Ok(bus) = AcerControlDBus::new() {
            if let Ok(brightness) = bus.keyboard_brightness() {
                if brightness != last_brightness {
                    idle.set_brightness(brightness);
                    last_brightness = brightness;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Ok(())
}

pub fn take_activation_request() -> bool {
    ACTIVATE_REQUESTED.swap(false, Ordering::AcqRel)
}

pub fn take_quit_request() -> bool {
    QUIT_REQUESTED.swap(false, Ordering::AcqRel)
}
