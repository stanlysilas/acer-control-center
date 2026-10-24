use super::*;

impl ControlCenter {
    pub(super) fn page_content(&mut self, ui: &mut egui::Ui) {
        match self.page {
            Page::Dashboard => {
                ui.label(RichText::new("LIVE TELEMETRY").small().strong().color(MUTED));
                ui.add_space(10.0);
                Self::metric_grid(ui, [
                    ("CPU temperature", libacercontrol::telemetry::cpu_temperature(), " °C"),
                    ("GPU temperature", libacercontrol::telemetry::gpu_temperature(), " °C"),
                    ("CPU fan", libacercontrol::telemetry::cpu_fan_rpm(), " RPM"),
                    ("GPU fan", libacercontrol::telemetry::gpu_fan_rpm(), " RPM"),
                ]);
                ui.add_space(16.0);
                ui.label(RichText::new("Choose a section from the navigation to manage cooling, lighting, and device settings.").color(MUTED));
            }
            Page::Cooling => {
                ui.label(RichText::new("FAN MODE").small().strong().color(MUTED));
                ui.add_space(8.0);
                ui.label("Fan mode");
                let selected = components::segmented_choice(
                    ui,
                    self.fan_mode,
                    &[(components::FanMode::Auto, "Auto"), (components::FanMode::Max, "Max")],
                );
                if let Some(mode) = selected {
                    let success = match mode {
                        components::FanMode::Auto => self.action("Auto fan", || libacercontrol::dbus::AcerControlDBus::new()?.set_fan_auto()),
                        components::FanMode::Max => self.action("Maximum fan", || libacercontrol::dbus::AcerControlDBus::new()?.set_fan_max()),
                    };
                    if success {
                        self.fan_mode = mode;
                        self.fan_mode_confirmed = true;
                    }
                }
                ui.add_space(8.0);
                if self.fan_mode_confirmed {
                    ui.label(format!("Last selected mode: {}", match self.fan_mode { components::FanMode::Auto => "Auto", components::FanMode::Max => "Max" }));
                } else {
                    ui.label("Auto is assumed at startup; this firmware interface does not report the current mode.");
                }
                ui.add_space(20.0);
                Self::metric_grid(ui, [
                    ("CPU fan", libacercontrol::telemetry::cpu_fan_rpm(), " RPM"),
                    ("GPU fan", libacercontrol::telemetry::gpu_fan_rpm(), " RPM"),
                    ("CPU temperature", libacercontrol::telemetry::cpu_temperature(), " °C"),
                    ("GPU temperature", libacercontrol::telemetry::gpu_temperature(), " °C"),
                ]);
            }
            Page::Keyboard => {
                ui.label(RichText::new("BACKLIGHT").small().strong().color(MUTED));
                ui.add_space(10.0);
                let mut changed = false;
                ui.horizontal(|ui| { ui.label("Brightness"); changed |= ui.add(egui::Slider::new(&mut self.brightness, 0..=255)).changed(); ui.label(format!("{} / 255", self.brightness)); });
                ui.horizontal(|ui| { ui.label("Color"); changed |= egui::color_picker::color_edit_button_srgba(ui, &mut self.color, egui::color_picker:: Alpha::Opaque).changed(); });
                if changed {
                    let (r,g,b) = (self.color.r(), self.color.g(), self.color.b());
                    let level = self.brightness;
                    if self.action("Keyboard lighting", move || {
                        let bus = libacercontrol::dbus::AcerControlDBus::new()?;
                        bus.set_keyboard_brightness(level)?;
                        bus.set_keyboard_color(r,g,b)
                    }) {
                        if let Some(controller) = &self.idle_controller {
                            controller.set_brightness(level);
                        }
                        self.animation_mode = components::AnimationMode::Off;
                        self.animation_mode_confirmed = true;
                    }
                }
                ui.add_space(16.0);
                ui.label("Keyboard animation");
                let selected = components::segmented_choice(
                    ui,
                    self.animation_mode,
                    &[(components::AnimationMode::Off, "Off"), (components::AnimationMode::Breathing, "Breathing")],
                );
                if let Some(mode) = selected {
                    let success = match mode {
                        components::AnimationMode::Off => self.action("Keyboard animation", || libacercontrol::dbus::AcerControlDBus::new()?.stop_animation()),
                        components::AnimationMode::Breathing => self.action("Keyboard animation", || libacercontrol::dbus::AcerControlDBus::new()?.start_breathing()),
                    };
                    if success {
                        self.animation_mode = mode;
                        self.animation_mode_confirmed = true;
                    }
                }
                if self.animation_mode_confirmed {
                    ui.label(format!("Restored animation setting: {}", match self.animation_mode { components::AnimationMode::Off => "Off", components::AnimationMode::Breathing => "Breathing" }));
                } else {
                    ui.label("Off is assumed at startup; the daemon could not report the saved animation setting.");
                }
                ui.add_space(18.0);
                ui.label(RichText::new("FADE AFTER INACTIVITY").strong());
                let mut idle_timeout = self.idle_timeout_seconds;
                egui::ComboBox::from_id_salt("keyboard-idle-timeout")
                    .selected_text(if idle_timeout == 0 {
                        "Off".to_owned()
                    } else {
                        format!("{} seconds", idle_timeout)
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut idle_timeout, 0, "Off");
                        for seconds in (5..=30).step_by(5) {
                            ui.selectable_value(
                                &mut idle_timeout,
                                seconds,
                                format!("{seconds} seconds"),
                            );
                        }
                    });
                if idle_timeout != self.idle_timeout_seconds {
                    let previous = self.idle_timeout_seconds;
                    let result = keyboard_idle::save_timeout(idle_timeout)
                        .and_then(|()| {
                            keyboard_idle::set_login_autostart(
                                idle_timeout > 0 || self.minimize_to_tray,
                            )
                        });
                    match result {
                        Ok(()) => {
                            if idle_timeout > 0 && !self.tray_ready {
                                match instance::start_background_service() {
                                    Ok(()) => {
                                        self.tray_ready = true;
                                        self.idle_controller = None;
                                    }
                                    Err(error) => {
                                        self.idle_controller = Some(keyboard_idle::KeyboardIdleController::new(idle_timeout, self.brightness));
                                        self.set_notice(false, format!("Inactivity dimming is active only while this window stays open because the tray service could not start: {error}"));
                                    }
                                }
                            } else if idle_timeout == 0 && !self.minimize_to_tray {
                                instance::request_background_quit();
                                self.tray_ready = false;
                                self.idle_controller = None;
                            } else if let Some(controller) = &self.idle_controller {
                                controller.set_timeout(idle_timeout);
                            }
                            self.idle_timeout_seconds = idle_timeout;
                            if self.notice.as_ref().is_none_or(|(ok, _, _)| *ok) {
                                self.set_notice(true, if idle_timeout > 0 {
                                "Inactivity timeout saved. The tray service keeps it running after the window closes.".to_owned()
                            } else {
                                "Inactivity timeout disabled and login startup removed.".to_owned()
                                });
                            }
                        }
                        Err(error) => {
                            let _ = keyboard_idle::save_timeout(previous);
                            self.set_notice(false, format!("Could not save inactivity settings: {error}"));
                        }
                    }
                }
                ui.label("The keyboard fades out after inactivity and fades back in when you return.");
                let timeout_status = self.idle_controller.as_ref().map(|controller| controller.status())
                    .unwrap_or_else(|| if self.idle_timeout_seconds > 0 { "Inactivity dimming runs in the background service".to_owned() } else { "Idle dimming is off".to_owned() });
                ui.label(RichText::new(timeout_status).weak().small());
                Self::keyboard_preview(ui, self.color, self.brightness);
            }
            Page::Performance => { ui.label(RichText::new("Performance profiles are planned for a future release.").color(MUTED)); }
            Page::Battery => { ui.label(RichText::new("Battery charge management is planned for a future release.").color(MUTED)); }
            Page::Settings => {
                ui.add_space(12.0);
                let previous = self.minimize_to_tray;
                let mut minimize_to_tray = previous;
                ui.checkbox(&mut minimize_to_tray, "Minimize to system tray when closing the window");
                if !self.tray_ready {
                    ui.label(RichText::new("Enabling this starts the background tray service for this session.").weak());
                }
                if minimize_to_tray != previous {
                    let result = keyboard_idle::save_minimize_to_tray(minimize_to_tray)
                        .and_then(|()| {
                            keyboard_idle::set_login_autostart(
                                minimize_to_tray || self.idle_timeout_seconds > 0,
                            )
                        });
                    match result {
                        Ok(()) => {
                            self.minimize_to_tray = minimize_to_tray;
                            if minimize_to_tray {
                                match instance::start_background_service() {
                                    Ok(()) => {
                                        self.tray_ready = true;
                                        self.idle_controller = None;
                                    }
                                    Err(error) => {
                                        self.set_notice(false, format!("Minimize to tray is saved and will be retried at login, but the tray service could not start: {error}"));
                                    }
                                }
                            } else if self.idle_timeout_seconds == 0 {
                                instance::request_background_quit();
                                self.tray_ready = false;
                            }
                            if self.tray_ready || !minimize_to_tray {
                                self.set_notice(true, if minimize_to_tray {
                                    "Closing the window will close its window while the background service remains in the system tray.".to_owned()
                                } else {
                                    "Closing the window will exit Acer Control Center when no background timeout is active.".to_owned()
                                });
                            }
                        }
                        Err(error) => {
                            let _ = keyboard_idle::save_minimize_to_tray(previous);
                            let _ = keyboard_idle::set_login_autostart(
                                previous || self.idle_timeout_seconds > 0,
                            );
                            self.set_notice(false, format!("Could not save tray setting: {error}"));
                        }
                    }
                }
                ui.label(RichText::new("Login startup stays enabled while tray minimization or keyboard inactivity dimming is on. The background service starts hidden with a tray icon.").weak());
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button("Refresh status").clicked() {
                        self.about_info = about::AboutInfo::collect();
                    }
                    ui.label(RichText::new("Version and hardware information").weak());
                });
                egui::Grid::new("system-status")
                    .num_columns(2)
                    .striped(true)
                    .spacing([24.0, 8.0])
                    .show(ui, |ui| {
                        for (label, value) in &self.about_info.rows {
                            ui.label(RichText::new(label).weak());
                            ui.label(value);
                            ui.end_row();
                        }
                    });
                ui.add_space(20.0);
                ui.separator();
                ui.heading("Updates");
                ui.label("Check GitHub for a newer version. Download and installation stay under your control.");
                if ui.add_enabled(!self.update_checking, egui::Button::new("Check for updates")).clicked() {
                    self.update_checking = true;
                    self.update_status = "Checking GitHub Releases…".to_owned();
                    self.latest_release = None;
                    let sender = self.update_sender.clone();
                    std::thread::spawn(move || {
                        let _ = sender.send(about::check_latest_release());
                    });
                }
                ui.label(&self.update_status);
                if let Some(release) = &self.latest_release {
                    ui.hyperlink_to("View release and downloads", &release.html_url);
                } else if !self.update_checking
                    && self.update_status != "Updates are checked only when you ask."
                    && self.update_status != "Checking GitHub Releases…"
                {
                    ui.hyperlink_to("Open project releases", about::RELEASES_PAGE_URL);
                }
            }
        }
    }
}
