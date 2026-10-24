use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct UserState {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub brightness: u8,
    pub fan_max: bool,
    pub breathing: bool,
}

impl UserState {
    pub fn defaults(r: u8, g: u8, b: u8, brightness: u8) -> Self {
        Self {
            r,
            g,
            b,
            brightness,
            fan_max: false,
            breathing: false,
        }
    }
}

pub struct StateStore {
    path: PathBuf,
    write_lock: Mutex<()>,
}

impl StateStore {
    pub fn new() -> Result<Self, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "Unable to determine configuration directory".to_string())?
            .join("acer-control-center");

        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create configuration directory: {e}"))?;

        Ok(Self {
            path: config_dir.join("state"),
            write_lock: Mutex::new(()),
        })
    }

    pub fn load(&self) -> Result<Option<UserState>, String> {
        if !self.path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read saved settings: {e}"))?;
        let values: Vec<u8> = contents
            .split_whitespace()
            .map(|value| {
                value
                    .parse::<u8>()
                    .map_err(|_| "Invalid saved setting value".to_string())
            })
            .collect::<Result<_, _>>()?;

        // Four-value files were written by earlier versions and contained only
        // RGB and brightness. Treat their missing settings as firmware Auto/Off.
        let (fan_max, breathing) = match values.as_slice() {
            [_, _, _, _] => (false, false),
            [_, _, _, _, fan, breathing] if *fan <= 1 && *breathing <= 1 => {
                (*fan == 1, *breathing == 1)
            }
            _ => return Err("Invalid saved settings format".into()),
        };

        Ok(Some(UserState {
            r: values[0],
            g: values[1],
            b: values[2],
            brightness: values[3],
            fan_max,
            breathing,
        }))
    }

    pub fn save(&self, state: UserState) -> Result<(), String> {
        let _write_guard = self
            .write_lock
            .lock()
            .map_err(|_| "Failed to lock saved settings".to_string())?;
        let contents = format!(
            "{} {} {} {} {} {}\n",
            state.r,
            state.g,
            state.b,
            state.brightness,
            u8::from(state.fan_max),
            u8::from(state.breathing),
        );

        let temporary = self.path.with_extension("state.tmp");
        fs::write(&temporary, contents)
            .map_err(|e| format!("Failed to write saved settings: {e}"))?;
        fs::rename(&temporary, &self.path)
            .map_err(|e| format!("Failed to commit saved settings: {e}"))
    }
}
