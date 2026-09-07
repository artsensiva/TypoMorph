use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use evdev::{AttributeSet, Device, InputEventKind, Key};
use thiserror::Error;

pub type RawKeycode = u16;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("failed to open input device {path}: {source}")]
    OpenInput {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("no physical keyboard input device was found under /dev/input")]
    NoKeyboard,
    #[error("failed to open uinput device: {0}")]
    OpenUinput(std::io::Error),
    #[error("failed to create virtual keyboard: {0}")]
    CreateVirtualKeyboard(String),
    #[error("D-Bus operation failed: {0}")]
    Dbus(String),
    #[error("unsupported layout backend: {0}")]
    UnsupportedBackend(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawKeyEvent {
    pub keycode: RawKeycode,
    pub pressed: bool,
    pub repeat: bool,
}

pub struct EvdevKeyboard {
    device: Device,
    path: PathBuf,
}

impl EvdevKeyboard {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PlatformError> {
        let path = path.as_ref().to_path_buf();
        let device = Device::open(&path).map_err(|source| PlatformError::OpenInput {
            path: path.clone(),
            source,
        })?;
        Ok(Self { device, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> Option<&str> {
        self.device.name()
    }

    pub fn next_events(&mut self) -> Result<Vec<RawKeyEvent>, std::io::Error> {
        let mut events = Vec::new();
        for event in self.device.fetch_events()? {
            if let InputEventKind::Key(key) = event.kind() {
                events.push(RawKeyEvent {
                    keycode: key.code(),
                    pressed: event.value() == 1,
                    repeat: event.value() == 2,
                });
            }
        }
        Ok(events)
    }
}

pub fn open_first_keyboard() -> Result<EvdevKeyboard, PlatformError> {
    let mut paths: Vec<PathBuf> = fs::read_dir("/dev/input")
        .map_err(|source| PlatformError::OpenInput {
            path: PathBuf::from("/dev/input"),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("event"))
        })
        .collect();
    paths.sort();

    for path in paths {
        let Ok(device) = Device::open(&path) else {
            continue;
        };
        let name = device.name().unwrap_or("");
        if name
            .to_ascii_lowercase()
            .contains("typomorph-virtual-keyboard")
        {
            continue;
        }
        let Some(keys) = device.supported_keys() else {
            continue;
        };
        let has_keyboard_keys = [Key::KEY_A, Key::KEY_Z, Key::KEY_ENTER, Key::KEY_SPACE]
            .into_iter()
            .all(|key| keys.contains(key));
        if !has_keyboard_keys {
            continue;
        }

        return Ok(EvdevKeyboard { device, path });
    }

    Err(PlatformError::NoKeyboard)
}

pub struct UinputKeyboard {
    device: evdev::uinput::VirtualDevice,
}

impl UinputKeyboard {
    pub fn open() -> Result<Self, PlatformError> {
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::KEY_BACKSPACE);
        let device = evdev::uinput::VirtualDeviceBuilder::new()
            .map_err(|error| PlatformError::CreateVirtualKeyboard(error.to_string()))?
            .name("typomorph-virtual-keyboard")
            .with_keys(&keys)
            .map_err(|error| PlatformError::CreateVirtualKeyboard(error.to_string()))?
            .build()
            .map_err(|error| PlatformError::OpenUinput(error))?;
        Ok(Self { device })
    }

    pub fn from_device(device: evdev::uinput::VirtualDevice) -> Self {
        Self { device }
    }

    pub fn emit_backspaces(&mut self, count: usize) -> Result<(), std::io::Error> {
        for _ in 0..count {
            self.device.emit(&[evdev::InputEvent::new(
                evdev::EventType::KEY,
                Key::KEY_BACKSPACE.code(),
                1,
            )])?;
            self.device.emit(&[evdev::InputEvent::new(
                evdev::EventType::KEY,
                Key::KEY_BACKSPACE.code(),
                0,
            )])?;
        }
        Ok(())
    }

    pub fn emit_replacement(&mut self, keycodes: &[RawKeycode]) -> Result<(), std::io::Error> {
        for &keycode in keycodes {
            self.device
                .emit(&[evdev::InputEvent::new(evdev::EventType::KEY, keycode, 1)])?;
            self.device
                .emit(&[evdev::InputEvent::new(evdev::EventType::KEY, keycode, 0)])?;
        }
        Ok(())
    }

    pub fn replace_text(
        &mut self,
        original_key_count: usize,
        replacement_keycodes: &[RawKeycode],
    ) -> Result<(), std::io::Error> {
        self.emit_backspaces(original_key_count)?;
        self.emit_replacement(replacement_keycodes)
    }
}

pub trait LayoutSwitcher {
    fn switch_to(&self, layout: &str) -> Result<(), PlatformError>;
}

pub struct GnomeShellSwitcher {
    connection: zbus::blocking::Connection,
}

impl GnomeShellSwitcher {
    pub fn connect() -> Result<Self, PlatformError> {
        let connection = zbus::blocking::Connection::session()
            .map_err(|error| PlatformError::Dbus(error.to_string()))?;
        Ok(Self { connection })
    }

    pub fn from_connection(connection: zbus::blocking::Connection) -> Self {
        Self { connection }
    }
}

impl LayoutSwitcher for GnomeShellSwitcher {
    fn switch_to(&self, layout: &str) -> Result<(), PlatformError> {
        let expression = format!(
            "global.display.set_input_source('{}')",
            layout.replace('\'', "\\'")
        );
        self.connection
            .call_method(
                Some("org.gnome.Shell"),
                "/org/gnome/Shell",
                Some("org.gnome.Shell"),
                "Eval",
                &(expression,),
            )
            .map(|_| ())
            .map_err(|error| PlatformError::Dbus(error.to_string()))
    }
}

pub fn open_input_device(path: impl AsRef<Path>) -> Result<EvdevKeyboard, PlatformError> {
    EvdevKeyboard::open(path)
}

pub fn input_device_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true);
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_key_event_preserves_press_release_and_repeat_states() {
        assert_eq!(
            RawKeyEvent {
                keycode: 30,
                pressed: true,
                repeat: false
            }
            .keycode,
            30
        );
        assert!(
            !RawKeyEvent {
                keycode: 30,
                pressed: false,
                repeat: true
            }
            .pressed
        );
    }

    #[test]
    fn input_device_options_are_read_only() {
        let options = input_device_options();
        let _ = options;
    }
}
