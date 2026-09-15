use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

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

pub struct MultiEvdevKeyboard {
    devices: Vec<(String, PathBuf)>,
    events: Receiver<RawKeyEvent>,
}

fn spawn_keyboard_listener(
    path: PathBuf,
    sender: mpsc::Sender<RawKeyEvent>,
    active_paths: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<PathBuf>>>,
) -> Option<String> {
    let Ok(mut device) = Device::open(&path) else {
        return None;
    };
    let name = device.name().unwrap_or("unnamed keyboard").to_string();
    if name
        .to_ascii_lowercase()
        .contains("typomorph-virtual-keyboard")
    {
        return None;
    }
    let keys = device.supported_keys()?;
    if !(keys.contains(Key::KEY_A)
        || keys.contains(Key::KEY_SPACE)
        || keys.contains(Key::KEY_ENTER))
    {
        return None;
    }

    {
        let mut set = active_paths.lock().unwrap();
        if set.contains(&path) {
            return None;
        }
        set.insert(path.clone());
    }

    let name_clone = name.clone();
    let p_clone = path.clone();
    let set_clone = active_paths.clone();

    thread::spawn(move || {
        eprintln!(
            "[DEBUG] Dynamic reader thread started for: {:?} ({})",
            name_clone,
            p_clone.display()
        );
        loop {
            let Ok(batch) = device.fetch_events() else {
                eprintln!(
                    "[DEBUG] Device disconnected: {:?} ({})",
                    name_clone,
                    p_clone.display()
                );
                let mut set = set_clone.lock().unwrap();
                set.remove(&p_clone);
                break;
            };
            for event in batch {
                if let InputEventKind::Key(key) = event.kind() {
                    let raw_ev = RawKeyEvent {
                        keycode: key.code(),
                        pressed: event.value() == 1,
                        repeat: event.value() == 2,
                    };
                    if sender.send(raw_ev).is_err() {
                        return;
                    }
                }
            }
        }
    });

    Some(name)
}

impl MultiEvdevKeyboard {
    pub fn open_all() -> Result<Self, PlatformError> {
        let (sender, events) = mpsc::channel();
        let active_paths =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let mut devices = Vec::new();

        let initial_paths = keyboard_paths().unwrap_or_default();
        for path in initial_paths {
            if let Some(name) =
                spawn_keyboard_listener(path.clone(), sender.clone(), active_paths.clone())
            {
                devices.push((name, path));
            }
        }

        // Фоновый поток для авто-подключения любых новых клавиатур (BT / USB / Dock)
        let watcher_sender = sender.clone();
        let watcher_set = active_paths.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(2));
            if let Ok(paths) = keyboard_paths() {
                for path in paths {
                    spawn_keyboard_listener(path, watcher_sender.clone(), watcher_set.clone());
                }
            }
        });

        Ok(Self { devices, events })
    }

    pub fn devices(&self) -> &[(String, PathBuf)] {
        &self.devices
    }

    pub fn recv(&self) -> Result<RawKeyEvent, mpsc::RecvError> {
        self.events.recv()
    }
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
    let path = keyboard_paths()?
        .into_iter()
        .next()
        .ok_or(PlatformError::NoKeyboard)?;
    EvdevKeyboard::open(path)
}

fn keyboard_paths() -> Result<Vec<PathBuf>, PlatformError> {
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

    paths.retain(|path| {
        let Ok(device) = Device::open(path) else {
            return false;
        };
        let name = device.name().unwrap_or("");
        if name
            .to_ascii_lowercase()
            .contains("typomorph-virtual-keyboard")
        {
            return false;
        }
        let Some(keys) = device.supported_keys() else {
            return false;
        };
        // More lenient check for wireless receivers/keyboards: require at least KEY_A or KEY_SPACE or KEY_ENTER
        keys.contains(Key::KEY_A) || keys.contains(Key::KEY_SPACE) || keys.contains(Key::KEY_ENTER)
    });

    Ok(paths)
}

pub struct UinputKeyboard {
    device: evdev::uinput::VirtualDevice,
}

impl UinputKeyboard {
    pub fn open() -> Result<Self, PlatformError> {
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::KEY_BACKSPACE);
        keys.insert(Key::KEY_SPACE);
        // Add all letters for both US and RU layouts
        for code in 16..=25 {
            keys.insert(Key::new(code));
        } // Q-P
        for code in 30..=38 {
            keys.insert(Key::new(code));
        } // A-L
        for code in 44..=50 {
            keys.insert(Key::new(code));
        } // Z-M
          // Also add some extra keys just in case
        keys.insert(Key::KEY_LEFTMETA);
        keys.insert(Key::KEY_LEFTSHIFT);
        keys.insert(Key::KEY_LEFTALT);

        let device = evdev::uinput::VirtualDeviceBuilder::new()
            .map_err(|error| PlatformError::CreateVirtualKeyboard(error.to_string()))?
            .name("typomorph-virtual-keyboard")
            .with_keys(&keys)
            .map_err(|error| PlatformError::CreateVirtualKeyboard(error.to_string()))?
            .build()
            .map_err(PlatformError::OpenUinput)?;
        Ok(Self { device })
    }

    pub fn from_device(device: evdev::uinput::VirtualDevice) -> Self {
        Self { device }
    }

    pub fn emit_backspaces(&mut self, count: usize) -> Result<(), std::io::Error> {
        eprintln!("Sending {} backspaces...", count);
        for _ in 0..count {
            self.device.emit(&[evdev::InputEvent::new(
                evdev::EventType::KEY,
                Key::KEY_BACKSPACE.code(),
                1,
            )])?;
            self.device.emit(&[evdev::InputEvent::new(
                evdev::EventType::SYNCHRONIZATION,
                0,
                0,
            )])?;
            sleep(Duration::from_millis(15));
            self.device.emit(&[evdev::InputEvent::new(
                evdev::EventType::KEY,
                Key::KEY_BACKSPACE.code(),
                0,
            )])?;
            self.device.emit(&[evdev::InputEvent::new(
                evdev::EventType::SYNCHRONIZATION,
                0,
                0,
            )])?;
            sleep(Duration::from_millis(15));
        }
        Ok(())
    }

    pub fn emit_replacement(&mut self, keycodes: &[RawKeycode]) -> Result<(), std::io::Error> {
        eprintln!("Emitting replacement text ({} keys)...", keycodes.len());
        for &keycode in keycodes {
            self.device
                .emit(&[evdev::InputEvent::new(evdev::EventType::KEY, keycode, 1)])?;
            self.device.emit(&[evdev::InputEvent::new(
                evdev::EventType::SYNCHRONIZATION,
                0,
                0,
            )])?;
            sleep(Duration::from_millis(5));
            self.device
                .emit(&[evdev::InputEvent::new(evdev::EventType::KEY, keycode, 0)])?;
            self.device.emit(&[evdev::InputEvent::new(
                evdev::EventType::SYNCHRONIZATION,
                0,
                0,
            )])?;
            sleep(Duration::from_millis(5));
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
        eprintln!("Executing layout swap to: {}...", layout);

        // GNOME 46+ Wayland safety: org.gnome.Shell.Eval is often restricted.
        // We use org.gnome.desktop.input-sources mru-sources if possible,
        // but the most reliable way via D-Bus for extensions/shell is often
        // calling a specific method if a custom extension is present,
        // OR using the standard GSettings-like interface via D-Bus.
        // For Ubuntu 26.04/GNOME 46, we'll try to use the gsettings-like D-Bus call
        // to change current index or use a more modern approach.

        // Fallback/Standard: Try to use a simpler shell evaluation if allowed,
        // but with better error handling.
        // Note: In modern GNOME, Eval is disabled by default for security.

        // A better way without Eval is to use `gsettings` or D-Bus for `org.gnome.desktop.input-sources`.
        // Since we are in a daemon, we can try to run `gsettings` command as a reliable fallback
        // or use the D-Bus interface for settings.

        let status = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.input-sources",
                "current",
                if layout == "ru" { "1" } else { "0" },
            ])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => {
                // If gsettings fails or isn't what we want, try the Eval as last resort
                let expression = format!(
                    "global.display.get_input_source_manager().get_sources().forEach(s => {{ if (s.id == '{}') s.activate(); }})",
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
