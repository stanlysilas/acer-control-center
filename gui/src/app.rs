
use eframe::egui;
use egui::{Color32, RichText, Stroke};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::{about, components, instance, keyboard_idle, libacercontrol};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page { Dashboard, Cooling, Keyboard, Performance, Battery, Settings }

impl Page {
    const ALL: [Self; 6] = [Self::Dashboard, Self::Cooling, Self::Keyboard, Self::Performance, Self::Battery, Self::Settings];
    fn title(self) -> &'static str { match self {
        Self::Dashboard => "Dashboard", Self::Cooling => "Cooling", Self::Keyboard => "Keyboard RGB",
        Self::Performance => "Performance", Self::Battery => "Battery", Self::Settings => "Settings",
    }}
    fn description(self) -> &'static str { match self {
        Self::Dashboard => "Hardware status and device overview",
        Self::Cooling => "Fan control and thermal telemetry",
        Self::Keyboard => "Backlight color, effects, and inactivity behavior",
        Self::Performance => "System performance controls",
        Self::Battery => "Battery care and charging options",
        Self::Settings => "Application preferences and system information",
    }}
}

const APP_BG: Color32 = Color32::from_rgb(17, 20, 24);
const SIDEBAR_BG: Color32 = Color32::from_rgb(20, 24, 29);
const SURFACE: Color32 = Color32::from_rgb(26, 31, 37);
const SURFACE_HOVER: Color32 = Color32::from_rgb(34, 40, 47);
const BORDER: Color32 = Color32::from_rgb(46, 53, 61);
const TEXT: Color32 = Color32::from_rgb(229, 233, 237);
const MUTED: Color32 = Color32::from_rgb(151, 160, 169);
const ACCENT: Color32 = Color32::from_rgb(95, 174, 166);

pub(crate) struct ControlCenter {
    page: Page,
    brightness: u8,
    color: Color32,
    notice: Option<(bool, String, Instant)>,
    fan_mode: components::FanMode,
    fan_mode_confirmed: bool,
    animation_mode: components::AnimationMode,
    animation_mode_confirmed: bool,
    idle_timeout_seconds: u32,
    minimize_to_tray: bool,
    tray_ready: bool,
    idle_controller: Option<keyboard_idle::KeyboardIdleController>,
    about_info: about::AboutInfo,
    update_sender: mpsc::Sender<Result<about::LatestRelease, String>>,
    update_receiver: mpsc::Receiver<Result<about::LatestRelease, String>>,
    update_checking: bool,
    update_status: String,
    latest_release: Option<about::LatestRelease>,
}

impl ControlCenter {
    fn set_notice(&mut self, succeeded: bool, message: String) {
        self.notice = Some((succeeded, message, Instant::now()));
    }

    pub(crate) fn new(background_ready: bool) -> Self {
        let bus = libacercontrol::dbus::AcerControlDBus::new().ok();
        let brightness = bus
            .as_ref()
            .and_then(|bus| bus.keyboard_brightness().ok())
            .unwrap_or(128);
        let color = bus
            .as_ref()
            .and_then(|bus| bus.keyboard_color().ok())
            .map(|(r, g, b)| Color32::from_rgb(r, g, b))
            .unwrap_or(Color32::from_rgb(50, 140, 255));
        let (fan_mode, fan_mode_confirmed) = bus
            .as_ref()
            .and_then(|bus| bus.fan_max_selected().ok())
            .map(|is_max| {
                (
                    if is_max { components::FanMode::Max } else { components::FanMode::Auto },
                    true,
                )
            })
            .unwrap_or((components::FanMode::Auto, false));
        let (animation_mode, animation_mode_confirmed) = bus
            .as_ref()
            .and_then(|bus| bus.breathing_selected().ok())
            .map(|is_breathing| {
                (
                    if is_breathing { components::AnimationMode::Breathing } else { components::AnimationMode::Off },
                    true,
                )
            })
            .unwrap_or((components::AnimationMode::Off, false));
        let idle_timeout_seconds = keyboard_idle::load_timeout();
        let stored_minimize_to_tray = keyboard_idle::load_minimize_to_tray();
        // Keep the saved preference visible even if the tray host is slow to
        // appear during login. Availability is tracked separately by tray_ready.
        let minimize_to_tray = stored_minimize_to_tray;
        let autostart_error = keyboard_idle::set_login_autostart(
            idle_timeout_seconds > 0 || minimize_to_tray,
        )
        .err();
        let tray_ready = background_ready;
        let (update_sender, update_receiver) = mpsc::channel();
        Self {
            page: Page::Dashboard,
            brightness,
            color,
            notice: autostart_error.map(|error| {
                (false, format!("Could not configure login startup for keyboard inactivity: {error}"), Instant::now())
            }).or_else(|| (!tray_ready && stored_minimize_to_tray).then(|| (false, "Minimize to tray remains enabled, but the tray service is unavailable. The window may stay open until the desktop tray is ready.".to_owned(), Instant::now()))),
            fan_mode,
            fan_mode_confirmed,
            animation_mode,
            animation_mode_confirmed,
            idle_timeout_seconds,
            minimize_to_tray,
            tray_ready,
            idle_controller: (!background_ready && idle_timeout_seconds > 0).then(|| {
                keyboard_idle::KeyboardIdleController::new(idle_timeout_seconds, brightness)
            }),
            about_info: about::AboutInfo::collect(),
            update_sender,
            update_receiver,
            update_checking: false,
            update_status: "Updates are checked only when you ask.".to_owned(),
            latest_release: None,
        }
    }

    fn action(&mut self, label: &str, action: impl FnOnce() -> Result<(), String>) -> bool {
        let result = action();
        let succeeded = result.is_ok();
        self.notice = Some(match result { Ok(()) => (true, format!("{label} applied"), Instant::now()), Err(e) => (false, format!("{label} failed: {e}"), Instant::now()) });
        succeeded
    }

    fn metric(ui: &mut egui::Ui, label: &str, value: Result<i32, String>, unit: &str) {
        let text = value
            .map(|v| format!("{v}{unit}"))
            .unwrap_or_else(|_| "Unavailable".into());

        egui::Frame::new()
            .fill(SURFACE)
            .stroke(Stroke::new(1.0_f32, BORDER))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(12, 12))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                ui.label(
                    RichText::new(label.to_uppercase())
                        .small()
                        .color(MUTED),
                );

                ui.heading(RichText::new(text).size(21.0).color(TEXT));
            });
    }

    fn metric_grid(ui: &mut egui::Ui, metrics: [(&str, Result<i32, String>, &str); 4]) {
        let available_width = ui.available_width();
        let card_width = (available_width / 2.0).max(1.0);

        for row in 0..2 {
            ui.vertical(|ui| {

                for col in 0..2 {
                    let index = row * 2 + col;
                    let (label, value, unit) = &metrics[index];

                    ui.allocate_ui_with_layout(
                        egui::vec2(card_width, 100.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |card_ui| {
                            card_ui.set_width(card_width);
                            Self::metric(card_ui, label, value.clone(), unit);
                        },
                    );
                }
            });
        }
    }

    pub(crate) fn configure_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.dark_mode = true;
        visuals.override_text_color = Some(TEXT);
        visuals.window_fill = SURFACE;
        visuals.panel_fill = APP_BG;
        visuals.extreme_bg_color = Color32::from_rgb(13, 16, 19);
        visuals.faint_bg_color = Color32::from_rgb(23, 27, 32);
        visuals.selection.bg_fill = Color32::from_rgb(42, 76, 78);
        visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
        visuals.widgets.noninteractive.bg_fill = SURFACE;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(36, 42, 49);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
        visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
        visuals.widgets.active.bg_fill = Color32::from_rgb(42, 76, 78);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
        visuals.widgets.open.bg_fill = SURFACE_HOVER;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, BORDER);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 9.0);
        style.spacing.interact_size = egui::vec2(40.0, 34.0);
        style.visuals.window_corner_radius = egui::CornerRadius::same(10);
        ctx.set_style(style);
    }

    fn keyboard_preview(ui: &mut egui::Ui, color: Color32, brightness: u8) {
        ui.add_space(16.0);
        ui.label(RichText::new("LIVE KEYBOARD PREVIEW").strong());
        let (rect, _) = ui.allocate_exact_size(egui::vec2(680.0, 156.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let glow = |channel: u8| ((channel as u16 * brightness as u16) / 255) as u8;
        let key_color = Color32::from_rgb(glow(color.r()), glow(color.g()), glow(color.b()));
        let rows: [&[&str]; 4] = [
            &["Esc", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"],
            &["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "<-"],
            &["Tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "[", "]"],
            &["Caps", "A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'", "Enter"],
        ];
        for (row_index, row) in rows.iter().enumerate() {
            let gap = 5.0;
            let key_width = ((rect.width() - gap * (row.len() as f32 - 1.0)) / row.len() as f32).min(48.0);
            let row_width = row.len() as f32 * key_width + gap * (row.len() as f32 - 1.0);
            let start_x = rect.center().x - row_width / 2.0;
            let y = rect.top() + row_index as f32 * 34.0;
            for (column, label) in row.iter().enumerate() {
                let key = egui::Rect::from_min_size(
                    egui::pos2(start_x + column as f32 * (key_width + gap), y),
                    egui::vec2(key_width, 27.0),
                );
                painter.rect_filled(key, 5.0, Color32::from_gray(32));
                painter.rect_stroke(key, 5.0, egui::Stroke::new(1.0_f32, key_color), egui::StrokeKind::Inside);
                painter.text(key.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(10.0), key_color);
            }
        }
    }

}

mod ui;

impl Drop for ControlCenter {
    fn drop(&mut self) {
        if !self.minimize_to_tray && self.idle_timeout_seconds == 0 {
            instance::request_background_quit();
        }
    }
}

impl eframe::App for ControlCenter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if instance::take_activation_request() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if instance::take_window_quit_request() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ctx.input(|input| input.viewport().close_requested()) {
            if self.minimize_to_tray && self.tray_ready {
                log::info!("Window closed; background tray service and inactivity controller remain active.");
            } else if self.idle_timeout_seconds > 0 && self.tray_ready {
                log::info!("Window closed; inactivity controller remains active in the tray service.");
            }
        }
        if let Ok(result) = self.update_receiver.try_recv() {
            self.update_checking = false;
            match result {
                Ok(release) => {
                    let current = env!("CARGO_PKG_VERSION");
                    self.update_status = match about::compare_versions(current, &release.tag_name) {
                        Some(std::cmp::Ordering::Less) => format!("Update available: {}", release.tag_name),
                        Some(std::cmp::Ordering::Equal) => format!("You are up to date ({current})."),
                        Some(std::cmp::Ordering::Greater) => format!("This build ({current}) is newer than the latest published release ({}).", release.tag_name),
                        None => format!("Latest published version: {}", release.tag_name),
                    };
                    self.latest_release = Some(release);
                }
                Err(error) => {
                    self.update_status = if error.starts_with("No published release") {
                        error
                    } else {
                        format!("Could not check for updates: {error}")
                    };
                }
            }
        }
        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(230.0)
            .frame(egui::Frame::new().fill(SIDEBAR_BG).stroke(Stroke::new(1.0_f32, BORDER)).inner_margin(egui::Margin::same(14)))
            .show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("ACER").size(11.0).strong().color(ACCENT));
                ui.add_space(3.0);
                ui.label(RichText::new("Control Center").size(18.0).strong().color(TEXT));
                ui.label(RichText::new("DEVICE MANAGEMENT").size(9.0).color(MUTED));
                ui.add_space(22.0);
                for page in Page::ALL {
                    if components::sidebar_button(ui, page.title(), self.page == page) {
                        self.page = page;
                    }
                    ui.add_space(2.0);
                }
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).small().color(MUTED));
            });
        });
        egui::TopBottomPanel::top("page-header")
            .frame(egui::Frame::new().fill(SIDEBAR_BG).stroke(Stroke::new(1.0_f32, BORDER)).inner_margin(egui::Margin::symmetric(26, 16)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(RichText::new(self.page.title()).size(22.0).color(TEXT));
                        ui.label(RichText::new(self.page.description()).small().color(MUTED));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("LINUX DEVICE UTILITY").small().color(MUTED));
                    });
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(APP_BG).inner_margin(egui::Margin::same(24)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.set_max_width(980.0);
                    self.page_content(ui);
                    if self.notice.as_ref().is_some_and(|(_, _, created)| created.elapsed() >= Duration::from_secs(3)) {
                        self.notice = None;
                    }
                    if let Some((ok, message, _)) = &self.notice {
                        ui.add_space(16.0);
                        let color = if *ok { ACCENT } else { Color32::from_rgb(220, 132, 122) };
                        egui::Frame::new().fill(SURFACE).stroke(Stroke::new(1.0_f32, BORDER)).corner_radius(8.0).inner_margin(12).show(ui, |ui| {
                            ui.colored_label(color, message);
                        });
                    }
                });
            });
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}
