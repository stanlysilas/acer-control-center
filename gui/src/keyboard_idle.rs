use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use zbus::blocking::{Connection, Proxy};

use crate::libacercontrol::dbus::AcerControlDBus;

const FADE_DURATION: Duration = Duration::from_millis(900);
const TICK: Duration = Duration::from_millis(50);

enum Command {
    SetTimeout(u32),
    SetBrightness(u8),
    Stop,
}

pub struct KeyboardIdleController {
    sender: mpsc::Sender<Command>,
    status: Arc<Mutex<String>>,
}

impl KeyboardIdleController {
    pub fn new(timeout_seconds: u32, brightness: u8) -> Self {
        let (sender, receiver) = mpsc::channel();
        let status = Arc::new(Mutex::new("Idle dimming is off".to_owned()));
        let worker_status = Arc::clone(&status);

        thread::spawn(move || {
            run_worker(receiver, worker_status, timeout_seconds, brightness);
        });

        Self { sender, status }
    }

    pub fn set_timeout(&self, seconds: u32) {
        let _ = self.sender.send(Command::SetTimeout(seconds));
    }

    pub fn set_brightness(&self, brightness: u8) {
        let _ = self.sender.send(Command::SetBrightness(brightness));
    }

    pub fn status(&self) -> String {
        self.status
            .lock()
            .map(|value| value.clone())
            .unwrap_or_else(|_| "Idle dimming status is unavailable".to_owned())
    }
}

impl Drop for KeyboardIdleController {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Stop);
    }
}

pub fn load_timeout() -> u32 {
    std::fs::read_to_string(settings_path())
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|value| *value == 0 || (5..=30).contains(value) && value % 5 == 0)
        .unwrap_or(0)
}

pub fn save_timeout(seconds: u32) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, format!("{seconds}\n"))
        .map_err(|error| error.to_string())?;
    std::fs::rename(temporary, path).map_err(|error| error.to_string())
}

pub fn load_minimize_to_tray() -> bool {
    std::fs::read_to_string(minimize_to_tray_path())
        .ok()
        .is_some_and(|value| value.trim() == "true")
}

pub fn save_minimize_to_tray(enabled: bool) -> Result<(), String> {
    let path = minimize_to_tray_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, if enabled { "true\n" } else { "false\n" })
        .map_err(|error| error.to_string())?;
    std::fs::rename(temporary, path).map_err(|error| error.to_string())
}

pub fn set_login_autostart(enabled: bool) -> Result<(), String> {
    let autostart_dir = config_home().join("autostart");
    let entry = autostart_dir.join("acer-control-center-idle.desktop");

    if !enabled {
        match std::fs::remove_file(entry) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
        return Ok(());
    }

    std::fs::create_dir_all(&autostart_dir).map_err(|error| error.to_string())?;
    let executable = if std::path::Path::new("/usr/bin/acer-control-center").exists() {
        "/usr/bin/acer-control-center".to_owned()
    } else {
        std::env::current_exe()
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned()
    };
    let escaped_executable = executable
        .replace('%', "%%")
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let contents = format!(
        "[Desktop Entry]\nType=Application\nName=Acer Control Center Background Service\nComment=Provide system tray controls and keyboard inactivity dimming\nExec=\"{escaped_executable}\" --background\nTerminal=false\nNoDisplay=true\nX-GNOME-Autostart-enabled=true\n"
    );
    let temporary = entry.with_extension("desktop.tmp");
    std::fs::write(&temporary, contents).map_err(|error| error.to_string())?;
    std::fs::rename(temporary, entry).map_err(|error| error.to_string())
}

fn settings_path() -> PathBuf {
    config_home().join("acer-control-center").join("keyboard-idle-timeout")
}

fn minimize_to_tray_path() -> PathBuf {
    config_home().join("acer-control-center").join("minimize-to-tray")
}

fn config_home() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn run_worker(
    receiver: mpsc::Receiver<Command>,
    status: Arc<Mutex<String>>,
    mut timeout_seconds: u32,
    mut base_brightness: u8,
) {
    let mut desktop_idle = DesktopIdleMonitor::default();
    let mut hardware: Option<AcerControlDBus> = None;
    let mut is_idle = false;
    let mut idle_confirmations = 0_u8;
    let mut active_confirmations = 0_u8;
    let mut current = base_brightness as f32;
    let mut transition_from = current;
    let mut transition_to = current;
    let mut transition_started = Instant::now();
    let mut last_sent = Some(base_brightness);
    let mut next_poll = Instant::now();
    let mut last_monitor_error = Instant::now() - Duration::from_secs(30);

    loop {
        match receiver.recv_timeout(TICK) {
            Ok(Command::SetTimeout(seconds)) => {
                timeout_seconds = seconds;
                if seconds == 0 {
                    is_idle = false;
                    idle_confirmations = 0;
                    active_confirmations = 0;
                    set_status(&status, "Idle dimming is off");
                }
                next_poll = Instant::now();
            }
            Ok(Command::SetBrightness(value)) => {
                base_brightness = value;
                last_sent = Some(value);
                current = value as f32;
                transition_from = current;
                transition_to = if is_idle { 0.0 } else { current };
                transition_started = Instant::now();
            }
            Ok(Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                if let Some(bus) = hardware.as_ref() {
                    let _ = bus.set_keyboard_brightness_transient(base_brightness);
                }
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }

        let now = Instant::now();
        if timeout_seconds > 0 && now >= next_poll {
            match desktop_idle.seconds_since_activity() {
                Ok(idle_seconds) => {
                    if is_idle {
                        if idle_seconds <= 1 {
                            active_confirmations = active_confirmations.saturating_add(1);
                            if active_confirmations >= 2 {
                                is_idle = false;
                                idle_confirmations = 0;
                            }
                        } else {
                            active_confirmations = 0;
                        }
                    } else if idle_seconds >= timeout_seconds {
                        idle_confirmations = idle_confirmations.saturating_add(1);
                        if idle_confirmations >= 2 {
                            is_idle = true;
                            active_confirmations = 0;
                        }
                    } else {
                        idle_confirmations = 0;
                    }
                    set_status(
                        &status,
                        if is_idle {
                            "Dimming after inactivity"
                        } else {
                            "Idle dimming is ready"
                        },
                    );
                }
                Err(error) => {
                    if now.duration_since(last_monitor_error) >= Duration::from_secs(8) {
                        set_status(
                            &status,
                            &format!("Desktop idle detection unavailable: {error}"),
                        );
                        last_monitor_error = now;
                    }
                }
            }
            next_poll = now + Duration::from_millis(500);
        }

        let target = if timeout_seconds > 0 && is_idle {
            0.0
        } else {
            base_brightness as f32
        };
        if target != transition_to {
            transition_from = current;
            transition_to = target;
            transition_started = now;
        }

        let progress = (now.duration_since(transition_started).as_secs_f32()
            / FADE_DURATION.as_secs_f32())
            .clamp(0.0, 1.0);
        current = transition_from + (transition_to - transition_from) * progress;
        let output = current.round().clamp(0.0, 255.0) as u8;
        if last_sent != Some(output) {
            if hardware.is_none() {
                match AcerControlDBus::new() {
                    Ok(bus) => hardware = Some(bus),
                    Err(error) => {
                        set_status(&status, &format!("Keyboard control unavailable: {error}"));
                        continue;
                    }
                }
            }
            let result = hardware
                .as_ref()
                .expect("hardware connection initialized")
                .set_keyboard_brightness_transient(output);
            match result {
                Ok(()) => last_sent = Some(output),
                Err(error) => {
                    hardware = None;
                    set_status(&status, &format!("Could not fade keyboard lighting: {error}"));
                }
            }
        }
    }
}

fn set_status(status: &Mutex<String>, message: &str) {
    if let Ok(mut current) = status.lock() {
        if current.as_str() != message {
            *current = message.to_owned();
        }
    }
}

#[derive(Default)]
struct DesktopIdleMonitor {
    session: Option<Connection>,
}

impl DesktopIdleMonitor {
    fn seconds_since_activity(&mut self) -> Result<u32, String> {
        if self.session.is_none() {
            self.session = Some(Connection::session().map_err(|error| error.to_string())?);
        }
        let session = self.session.as_ref().expect("session initialized");

        for (service, path) in [
            (
                "org.freedesktop.ScreenSaver",
                "/org/freedesktop/ScreenSaver",
            ),
            ("org.freedesktop.ScreenSaver", "/ScreenSaver"),
            ("org.gnome.ScreenSaver", "/org/gnome/ScreenSaver"),
        ] {
            if let Ok(proxy) = Proxy::new(
                session,
                service,
                path,
                "org.freedesktop.ScreenSaver",
            ) {
                if let Ok(idle_seconds) = proxy.call::<_, _, u32>("GetSessionIdleTime", &()) {
                    return Ok(idle_seconds);
                }
            }
        }

        let mutter = Proxy::new(
            session,
            "org.gnome.Mutter.IdleMonitor",
            "/org/gnome/Mutter/IdleMonitor/Core",
            "org.gnome.Mutter.IdleMonitor",
        )
        .map_err(|error| error.to_string())?;
        let idle_millis = mutter
            .call::<_, _, u64>("GetIdletime", &())
            .map_err(|error| error.to_string())?;
        Ok((idle_millis / 1000).min(u32::MAX as u64) as u32)
    }
}
