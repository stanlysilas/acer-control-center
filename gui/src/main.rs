mod app;
mod components;
mod about;
mod instance;
mod keyboard_idle;
mod libacercontrol;
mod tray;

use eframe::egui;
use app::ControlCenter;

fn main() {
    env_logger::init();
    let background_start = std::env::args().any(|argument| argument == "--background");
    if background_start {
        let mut last_error = None;
        for attempt in 0..20 {
            match tray::run_background_service() {
                Ok(()) => return,
                Err(error) => {
                    last_error = Some(error);
                    if attempt < 19 {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }
        }
        if let Some(error) = last_error {
            log::warn!("Could not start tray and idle service after retrying: {error}");
            if keyboard_idle::load_timeout() > 0 || keyboard_idle::load_minimize_to_tray() {
                let _ = instance::activate_or_launch_window();
            }
        }
        return;
    }

    match instance::ensure_single_instance(false) {
        Ok(true) => {}
        Ok(false) => return,
        Err(error) => {
            log::error!("Could not start single-instance handling: {error}");
            std::process::exit(1);
        }
    }

    let needs_background_service = keyboard_idle::load_timeout() > 0
        || keyboard_idle::load_minimize_to_tray();
    let background_ready = if needs_background_service {
        match instance::start_background_service() {
            Ok(()) => true,
            Err(error) => {
                log::warn!("Background tray service unavailable: {error}");
                false
            }
        }
    } else {
        false
    };

    let icon_image = image::load_from_memory(include_bytes!("../resources/icons/acer-control-center.png"))
        .expect("Failed to load Acer Control Center app icon")
        .into_rgba8();
    let (icon_width, icon_height) = icon_image.dimensions();
    let icon = egui::IconData {
        rgba: icon_image.into_raw(),
        width: icon_width,
        height: icon_height,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(instance::APP_ID)
            .with_inner_size([1000.0, 720.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(icon)
            .with_visible(true),
        ..Default::default()
    };
    if let Err(error) = eframe::run_native(
        "Acer Control Center",
        options,
        Box::new(move |cc| {
            ControlCenter::configure_theme(&cc.egui_ctx);
            Ok(Box::new(ControlCenter::new(background_ready)))
        }),
    ) {
        log::error!("Acer Control Center exited with an error: {error}");
        std::process::exit(1);
    }
}
