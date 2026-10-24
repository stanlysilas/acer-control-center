use serde::Deserialize;

use crate::libacercontrol::{dbus::AcerControlDBus, driver, telemetry};

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/stanlysilas/acer-control-center/releases/latest";
pub const RELEASES_PAGE_URL: &str =
    "https://github.com/stanlysilas/acer-control-center/releases";

#[derive(Clone)]
pub struct AboutInfo {
    pub rows: Vec<(String, String)>,
}

impl AboutInfo {
    pub fn collect() -> Self {
        let daemon = AcerControlDBus::new().and_then(|bus| bus.version());
        let module_version = driver::version();
        let module_status = module_version
            .as_ref()
            .map(|version| format!("Loaded and responding (v{version})"))
            .unwrap_or_else(|_| "Not detected or not accessible".to_owned());
        let hardware = driver::hardware_supported()
            .map(|supported| if supported { "Supported" } else { "Not supported" }.to_owned())
            .unwrap_or_else(|_| "Could not query driver".to_owned());
        let interface = driver::interface().unwrap_or_else(|_| "Unavailable".to_owned());
        let module_rows = ["acer_keyboard", "acer_wmi", "acer_acpi", "acer_io"]
            .into_iter()
            .map(|name| {
                let path = format!("/sys/module/{name}");
                let version_path = format!("{path}/version");
                let value = if std::path::Path::new(&path).exists() {
                    read_trimmed(&version_path)
                        .map(|version| format!("Loaded (v{version})"))
                        .unwrap_or_else(|| "Loaded".to_owned())
                } else {
                    "Not loaded".to_owned()
                };
                (format!("Module: {name}"), value)
            });

        let mut rows = vec![
            ("Acer Control Center".to_owned(), format!("v{}", env!("CARGO_PKG_VERSION"))),
            (
                "Control daemon".to_owned(),
                daemon
                    .map(|version| format!("Running (v{version})"))
                    .unwrap_or_else(|_| "Not running or unreachable".to_owned()),
            ),
            (
                "Control library".to_owned(),
                driver::library_version()
                    .map(|version| format!("v{version}"))
                    .unwrap_or_else(|_| "Unavailable".to_owned()),
            ),
            ("Kernel".to_owned(), kernel_version()),
            ("Kernel driver".to_owned(), module_status),
            (
                "Driver ABI".to_owned(),
                format!("{}.{}", driver::abi_major(), driver::abi_minor()),
            ),
            ("Hardware support".to_owned(), hardware),
            ("Firmware interface".to_owned(), interface),
            ("Laptop model".to_owned(), laptop_model()),
            (
                "Keyboard RGB".to_owned(),
                if std::path::Path::new("/sys/class/leds/rgb:kbd_backlight").exists() {
                    "Available".to_owned()
                } else {
                    "Not detected".to_owned()
                },
            ),
            ("CPU fan".to_owned(), metric(telemetry::cpu_fan_rpm(), " RPM")),
            ("GPU fan".to_owned(), metric(telemetry::gpu_fan_rpm(), " RPM")),
            (
                "CPU temperature".to_owned(),
                metric(telemetry::cpu_temperature(), " °C"),
            ),
            (
                "GPU temperature".to_owned(),
                metric(telemetry::gpu_temperature(), " °C"),
            ),
        ];
        rows.extend(module_rows);
        Self { rows }
    }
}

fn metric(result: Result<i32, String>, unit: &str) -> String {
    result
        .map(|value| format!("{value}{unit}"))
        .unwrap_or_else(|_| "Unavailable".to_owned())
}

fn kernel_version() -> String {
    read_trimmed("/proc/sys/kernel/osrelease").unwrap_or_else(|| "Unavailable".to_owned())
}

fn laptop_model() -> String {
    let vendor = read_trimmed("/sys/class/dmi/id/sys_vendor");
    let product = read_trimmed("/sys/class/dmi/id/product_name");
    match (vendor, product) {
        (Some(vendor), Some(product)) => format!("{vendor} {product}"),
        (Some(vendor), None) => vendor,
        (None, Some(product)) => product,
        (None, None) => "Unavailable".to_owned(),
    }
}

fn read_trimmed(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[derive(Deserialize)]
pub struct LatestRelease {
    pub tag_name: String,
    pub html_url: String,
}

pub fn check_latest_release() -> Result<LatestRelease, String> {
    let response = ureq::get(LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Acer-Control-Center")
        .call()
        .map_err(|error| match error {
            ureq::Error::StatusCode(404) => "No published release is available yet.".to_owned(),
            error => format!("GitHub check failed: {error}"),
        })?;
    let release = response
        .into_body()
        .read_json::<LatestRelease>()
        .map_err(|error| format!("Could not read the release information: {error}"))?;
    if release.tag_name.trim().is_empty() || release.html_url.trim().is_empty() {
        return Err("GitHub returned an incomplete release record".to_owned());
    }
    Ok(release)
}

pub fn compare_versions(current: &str, latest: &str) -> Option<std::cmp::Ordering> {
    fn parts(value: &str) -> Option<Vec<u64>> {
        let clean = value.trim().trim_start_matches(['v', 'V']);
        let core = clean.split(['-', '+']).next()?;
        let mut numbers = core
            .split('.')
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        if numbers.is_empty() {
            return None;
        }
        while numbers.len() < 3 {
            numbers.push(0);
        }
        Some(numbers)
    }

    let current = parts(current)?;
    let latest = parts(latest)?;
    Some(current.cmp(&latest))
}
