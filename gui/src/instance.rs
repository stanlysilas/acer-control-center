use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub const APP_ID: &str = "acer-control-center";
static ACTIVATE_REQUESTED: AtomicBool = AtomicBool::new(false);
static WINDOW_QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);
static BACKGROUND_ACTIVATE_REQUESTED: AtomicBool = AtomicBool::new(false);
static BACKGROUND_QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

// Ensure one GUI process per logged-in user. A second launch asks the first
// instance to focus its window and exits immediately.
pub fn ensure_single_instance(background: bool) -> Result<bool, String> {
    let socket_path = socket_path(background)?;
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create runtime directory: {error}"))?;
    }

    for attempt in 0..2 {
        match UnixListener::bind(&socket_path) {
            Ok(listener) => {
                #[cfg(unix)]
                std::fs::set_permissions(
                    &socket_path,
                    std::os::unix::fs::PermissionsExt::from_mode(0o600),
                )
                .map_err(|error| format!("Could not secure activation socket: {error}"))?;

                thread::Builder::new()
                    .name("app-activation-listener".into())
                    .spawn(move || {
                        for incoming in listener.incoming() {
                            match incoming {
                                Ok(mut stream) => {
                                    let mut command = String::new();
                                    let _ = std::io::Read::read_to_string(&mut stream, &mut command);
                                    match (background, command.trim()) {
                                        (true, "activate") => BACKGROUND_ACTIVATE_REQUESTED.store(true, Ordering::Release),
                                        (true, "quit") => BACKGROUND_QUIT_REQUESTED.store(true, Ordering::Release),
                                        (false, "activate") => ACTIVATE_REQUESTED.store(true, Ordering::Release),
                                        (false, "quit") => WINDOW_QUIT_REQUESTED.store(true, Ordering::Release),
                                        _ => {}
                                    }
                                }
                                Err(error) => log::warn!("Activation listener stopped: {error}"),
                            }
                        }
                    })
                    .map_err(|error| format!("Could not start activation listener: {error}"))?;

                return Ok(true);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
                match UnixStream::connect(&socket_path) {
                    Ok(mut stream) => {
                        if !background {
                            stream
                                .write_all(b"activate\n")
                                .map_err(|error| format!("Could not activate existing window: {error}"))?;
                        }
                        return Ok(false);
                    }
                    Err(_) if attempt == 0 => {
                        // The previous process may have crashed and left its socket behind.
                        let _ = std::fs::remove_file(&socket_path);
                    }
                    Err(error) => {
                        return Err(format!("Another app instance may be starting: {error}"));
                    }
                }
            }
            Err(error) => return Err(format!("Could not create app activation socket: {error}")),
        }
    }

    Err("Could not establish the app activation socket.".into())
}

pub fn start_background_service() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut child = std::process::Command::new(executable)
        .arg("--background")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not start background service: {error}"))?;

    for _ in 0..40 {
        if child.try_wait().map_err(|error| error.to_string())?.is_some() {
            // A second start may have exited because the service is already running.
            if connect_and_send(true, "noop").is_ok() {
                return Ok(());
            }
            return Err("The tray and timeout background service could not start.".to_owned());
        }
        if connect_and_send(true, "noop").is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err("The background service did not become ready in time.".to_owned())
}

pub fn activate_or_launch_window() -> Result<(), String> {
    if connect_and_send(false, "activate").is_ok() {
        return Ok(());
    }
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    std::process::Command::new(executable)
        .arg("--window")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open Acer Control Center: {error}"))
}

pub fn request_background_quit() {
    let _ = connect_and_send(true, "quit");
}

pub fn request_window_quit() {
    let _ = connect_and_send(false, "quit");
}

pub fn take_activation_request() -> bool {
    ACTIVATE_REQUESTED.swap(false, Ordering::AcqRel)
}

pub fn take_window_quit_request() -> bool {
    WINDOW_QUIT_REQUESTED.swap(false, Ordering::AcqRel)
}

pub fn take_background_activation_request() -> bool {
    BACKGROUND_ACTIVATE_REQUESTED.swap(false, Ordering::AcqRel)
}

pub fn take_background_quit_request() -> bool {
    BACKGROUND_QUIT_REQUESTED.swap(false, Ordering::AcqRel)
}

fn socket_path(background: bool) -> Result<PathBuf, String> {
    let suffix = if background { "background" } else { "window" };
    if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        return Ok(PathBuf::from(runtime_dir).join(format!("acer-control-center-{suffix}.sock")));
    }

    let uid = unsafe { libc::getuid() };
    let private_dir = std::env::temp_dir().join(format!("acer-control-center-{uid}"));
    std::fs::create_dir_all(&private_dir)
        .map_err(|error| format!("Could not create private runtime directory: {error}"))?;
    #[cfg(unix)]
    std::fs::set_permissions(
        &private_dir,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .map_err(|error| format!("Could not secure private runtime directory: {error}"))?;
    Ok(private_dir.join(format!("acer-control-center-{suffix}.sock")))
}

fn connect_and_send(background: bool, command: &str) -> Result<(), String> {
    let path = socket_path(background)?;
    let mut stream = UnixStream::connect(path).map_err(|error| error.to_string())?;
    stream.write_all(format!("{command}\n").as_bytes()).map_err(|error| error.to_string())
}
