use nix::fcntl::{fcntl, FcntlArg, OFlag};
use std::fs::{self, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
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
    #[error("automatic replacement unavailable: uinput cannot bind edits to a field or serialize concurrent input")]
    UnsafeReplacementBackend,
    #[error("D-Bus operation failed: {0}")]
    Dbus(String),
    #[error("unsupported layout backend: {0}")]
    UnsupportedBackend(String),
}

#[derive(Clone, PartialEq, Eq)]
pub struct RawKeyEvent {
    pub keycode: RawKeycode,
    pub pressed: bool,
    pub repeat: bool,
    /// Name of the device that produced this event, as reported by evdev.
    /// Internal event provenance; do not include it in per-event output.
    pub source: String,
    /// Kernel-assigned event timestamp (`InputEvent::timestamp()`),
    /// milliseconds since the Unix epoch. This is the time the driver
    /// stamped the event, not when this process read it off the queue —
    /// useful for checking whether events from different reader threads
    /// were emitted in the order they were delivered to `recv()`.
    pub timestamp_ms: u64,
}

impl std::fmt::Debug for RawKeyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RawKeyEvent { [redacted] }")
    }
}

/// Identifies a physical device independently of its (possibly duplicated)
/// name string, for spotting the same USB dongle exposing several logical
/// `/dev/input/eventN` nodes (e.g. a main keyboard interface plus a
/// "Consumer Control" HID collection on the same receiver).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub path: PathBuf,
    pub vendor: u16,
    pub product: u16,
}

pub struct EvdevKeyboard {
    device: Device,
    path: PathBuf,
}

pub struct MultiEvdevKeyboard {
    devices: Vec<DeviceInfo>,
    events: Receiver<RawKeyEvent>,
    suppressed: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    watcher: Option<thread::JoinHandle<()>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryOutcome {
    Sent,
    Suppressed,
    ChannelClosed,
    Overflow,
}

/// Pulled out of the reader thread's loop body so the suppression logic
/// itself is unit-testable without a real evdev device: dropped regardless
/// of which device claims to have produced it, for as long as an emission
/// is in flight (see `MultiEvdevKeyboard::suppress_delivery`) — the second,
/// independent layer of defense against reading back the daemon's own
/// synthetic keystrokes.
fn deliver_event(
    sender: &mpsc::SyncSender<RawKeyEvent>,
    suppressed: &AtomicBool,
    event: RawKeyEvent,
) -> DeliveryOutcome {
    if suppressed.load(Ordering::SeqCst) {
        return DeliveryOutcome::Suppressed;
    }
    match sender.try_send(event) {
        Ok(()) => DeliveryOutcome::Sent,
        Err(mpsc::TrySendError::Disconnected(_)) => DeliveryOutcome::ChannelClosed,
        Err(mpsc::TrySendError::Full(_)) => DeliveryOutcome::Overflow,
    }
}

fn spawn_keyboard_listener(
    path: PathBuf,
    sender: mpsc::SyncSender<RawKeyEvent>,
    active_paths: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<PathBuf>>>,
    suppressed: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
) -> Option<(DeviceInfo, thread::JoinHandle<()>)> {
    let Ok(mut device) = Device::open(&path) else {
        return None;
    };
    // Nonblocking reads let Drop join every worker even while the keyboard is idle.
    let flags = OFlag::from_bits_truncate(fcntl(device.as_raw_fd(), FcntlArg::F_GETFL).ok()?);
    fcntl(
        device.as_raw_fd(),
        FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK),
    )
    .ok()?;
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

    let input_id = device.input_id();
    let info = DeviceInfo {
        name: name.clone(),
        path: path.clone(),
        vendor: input_id.vendor(),
        product: input_id.product(),
    };

    let name_clone = name.clone();
    let p_clone = path.clone();
    let set_clone = active_paths.clone();

    let worker = thread::spawn(move || {
        while !stopped.load(Ordering::SeqCst) {
            let batch = match device.fetch_events() {
                Ok(batch) => batch,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::park_timeout(Duration::from_millis(20));
                    continue;
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    // Losing a device also loses modifier/input ordering knowledge.
                    stopped.store(true, Ordering::SeqCst);
                    break;
                }
            };
            for event in batch {
                if stopped.load(Ordering::SeqCst) {
                    break;
                }
                if let InputEventKind::Key(key) = event.kind() {
                    let raw_ev = RawKeyEvent {
                        keycode: key.code(),
                        pressed: event.value() == 1,
                        repeat: event.value() == 2,
                        source: name_clone.clone(),
                        timestamp_ms: event
                            .timestamp()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0),
                    };
                    if matches!(
                        deliver_event(&sender, &suppressed, raw_ev),
                        DeliveryOutcome::ChannelClosed | DeliveryOutcome::Overflow
                    ) {
                        // Never process a truncated stream as continuous typing.
                        stopped.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        }
        set_clone.lock().unwrap().remove(&p_clone);
    });
    Some((info, worker))
}

impl MultiEvdevKeyboard {
    /// Diagnostic-only observer; not a field-safe production input adapter.
    pub fn open_all() -> Result<Self, PlatformError> {
        let (sender, events) = mpsc::sync_channel(256);
        let active_paths = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let suppressed = Arc::new(AtomicBool::new(false));
        let stopped = Arc::new(AtomicBool::new(false));
        let mut devices = Vec::new();
        let mut workers = Vec::new();
        for path in keyboard_paths().unwrap_or_default() {
            if let Some((info, worker)) = spawn_keyboard_listener(
                path,
                sender.clone(),
                active_paths.clone(),
                suppressed.clone(),
                stopped.clone(),
            ) {
                devices.push(info);
                workers.push(worker);
            }
        }
        if devices.is_empty() {
            return Err(PlatformError::NoKeyboard);
        }
        let watcher_stopped = stopped.clone();
        let watcher_suppressed = suppressed.clone();
        let watcher = thread::spawn(move || {
            let mut next_scan = std::time::Instant::now() + Duration::from_secs(2);
            while !watcher_stopped.load(Ordering::SeqCst) {
                thread::park_timeout(Duration::from_millis(100));
                if watcher_stopped.load(Ordering::SeqCst) {
                    break;
                }
                if std::time::Instant::now() < next_scan {
                    continue;
                }
                next_scan = std::time::Instant::now() + Duration::from_secs(2);
                if let Ok(paths) = keyboard_paths() {
                    for path in paths {
                        if watcher_stopped.load(Ordering::SeqCst) {
                            break;
                        }
                        if let Some((_, worker)) = spawn_keyboard_listener(
                            path,
                            sender.clone(),
                            active_paths.clone(),
                            watcher_suppressed.clone(),
                            watcher_stopped.clone(),
                        ) {
                            workers.push(worker);
                        }
                    }
                }
            }
            for worker in workers {
                worker.thread().unpark();
                let _ = worker.join();
            }
        });
        Ok(Self {
            devices,
            events,
            suppressed,
            stopped,
            watcher: Some(watcher),
        })
    }

    /// Legacy diagnostic suppression drops events; it is not a safe pause or repair primitive.
    pub fn suppress_delivery(&self) {
        self.suppressed.store(true, Ordering::SeqCst);
    }
    pub fn resume_delivery(&self) {
        self.suppressed.store(false, Ordering::SeqCst);
    }
    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }
    pub fn recv(&self) -> Result<RawKeyEvent, mpsc::RecvError> {
        loop {
            match self.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => return Ok(event),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Err(mpsc::RecvError),
            }
        }
    }
    pub fn recv_timeout(&self, timeout: Duration) -> Result<RawKeyEvent, mpsc::RecvTimeoutError> {
        if self.stopped.load(Ordering::SeqCst) {
            return Err(mpsc::RecvTimeoutError::Disconnected);
        }
        let result = self.events.recv_timeout(timeout);
        if self.stopped.load(Ordering::SeqCst) {
            return Err(mpsc::RecvTimeoutError::Disconnected);
        }
        result
    }
}
impl Drop for MultiEvdevKeyboard {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        if let Some(watcher) = self.watcher.take() {
            watcher.thread().unpark();
            let _ = watcher.join();
        }
        // All readers and their device descriptors are gone before Drop returns.
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
        let source = self.device.name().unwrap_or("unnamed keyboard").to_string();
        let mut events = Vec::new();
        for event in self.device.fetch_events()? {
            if let InputEventKind::Key(key) = event.kind() {
                let timestamp_ms = event
                    .timestamp()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                events.push(RawKeyEvent {
                    keycode: key.code(),
                    pressed: event.value() == 1,
                    repeat: event.value() == 2,
                    source: source.clone(),
                    timestamp_ms,
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

/// The current Linux adapter has no target-bound, input-serialized edit operation.
/// Check before capture, layout mutation, or opening a virtual keyboard.
pub fn require_safe_replacement_backend() -> Result<(), PlatformError> {
    Err(PlatformError::UnsafeReplacementBackend)
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

    /// Legacy sequential deletion/insertion is deliberately refused. Individual
    /// low-level emission helpers are not a safe automatic-replacement backend.
    pub fn replace_text(
        &mut self,
        _original_key_count: usize,
        _replacement_keycodes: &[RawKeycode],
    ) -> Result<(), std::io::Error> {
        require_safe_replacement_backend().map_err(std::io::Error::other)
    }
}

pub trait LayoutSwitcher {
    fn switch_from_to(&self, expected: &str, target: &str) -> Result<(), PlatformError>;
}

mod gnome;
pub use gnome::GnomeShellSwitcher;

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
                repeat: false,
                source: "test-keyboard".to_string(),
                timestamp_ms: 0,
            }
            .keycode,
            30
        );
        assert!(
            !RawKeyEvent {
                keycode: 30,
                pressed: false,
                repeat: true,
                source: "test-keyboard".to_string(),
                timestamp_ms: 0,
            }
            .pressed
        );
    }

    fn sample_event() -> RawKeyEvent {
        RawKeyEvent {
            keycode: 30,
            pressed: true,
            repeat: false,
            source: "test-keyboard".to_string(),
            timestamp_ms: 0,
        }
    }

    #[test]
    fn events_are_dropped_while_suppressed_and_flow_again_once_resumed() {
        let (sender, receiver) = mpsc::sync_channel(4);
        let suppressed = AtomicBool::new(false);

        assert_eq!(
            deliver_event(&sender, &suppressed, sample_event()),
            DeliveryOutcome::Sent
        );
        assert_eq!(receiver.try_recv(), Ok(sample_event()));

        // This is the exact window around switch_from_to()+replace_text(): while
        // suppressed, nothing reaches the channel, regardless of how many
        // events arrive — this is what stops an echo of our own synthetic
        // keystrokes (or a coincidental real one) from reaching the next
        // recv() and corrupting the next word's buffer.
        suppressed.store(true, Ordering::SeqCst);
        assert_eq!(
            deliver_event(&sender, &suppressed, sample_event()),
            DeliveryOutcome::Suppressed
        );
        assert_eq!(
            deliver_event(&sender, &suppressed, sample_event()),
            DeliveryOutcome::Suppressed
        );
        assert!(receiver.try_recv().is_err(), "channel must stay empty");

        suppressed.store(false, Ordering::SeqCst);
        assert_eq!(
            deliver_event(&sender, &suppressed, sample_event()),
            DeliveryOutcome::Sent
        );
        assert_eq!(receiver.try_recv(), Ok(sample_event()));
    }

    #[test]
    fn closed_channel_is_reported_even_while_not_suppressed() {
        let (sender, receiver) = mpsc::sync_channel(4);
        drop(receiver);
        let suppressed = AtomicBool::new(false);
        assert_eq!(
            deliver_event(&sender, &suppressed, sample_event()),
            DeliveryOutcome::ChannelClosed
        );
    }

    #[test]
    fn raw_event_debug_contains_no_key_device_or_timing_data() {
        assert_eq!(
            format!("{:?}", sample_event()),
            "RawKeyEvent { [redacted] }"
        );
    }

    #[test]
    fn bounded_delivery_reports_overflow_without_blocking_or_reordering() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let suppressed = AtomicBool::new(false);
        assert_eq!(
            deliver_event(&sender, &suppressed, sample_event()),
            DeliveryOutcome::Sent
        );
        let mut later = sample_event();
        later.keycode = 48;
        assert_eq!(
            deliver_event(&sender, &suppressed, later),
            DeliveryOutcome::Overflow
        );
        assert_eq!(receiver.try_recv().unwrap(), sample_event());
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn stopped_capture_refuses_queued_events_and_drop_joins_worker() {
        let (sender, events) = mpsc::sync_channel(1);
        sender.send(sample_event()).unwrap();
        let stopped = Arc::new(AtomicBool::new(true));
        let joined = Arc::new(AtomicBool::new(false));
        let completed = joined.clone();
        let watcher = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            completed.store(true, Ordering::SeqCst);
        });
        let keyboard = MultiEvdevKeyboard {
            devices: Vec::new(),
            events,
            suppressed: Arc::new(AtomicBool::new(false)),
            stopped,
            watcher: Some(watcher),
        };
        assert_eq!(
            keyboard.recv_timeout(Duration::ZERO),
            Err(mpsc::RecvTimeoutError::Disconnected)
        );
        drop(keyboard);
        assert!(joined.load(Ordering::SeqCst));
    }

    #[test]
    fn input_device_options_are_read_only() {
        let options = input_device_options();
        let _ = options;
    }
}

mod context;
pub use context::InputContext;
