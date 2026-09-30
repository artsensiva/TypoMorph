//! Content-free preferences. No input history or account credentials belong here.
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAX_BYTES: u64 = 32 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiLanguage {
    En,
    Ru,
    Uk,
    De,
    Fr,
    Es,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub version: u32,
    pub paused: bool,
    pub sounds_enabled: bool,
    /// None means use a supported OS UI language, otherwise English.
    pub ui_language: Option<UiLanguage>,
    pub autostart_enabled: bool,
    pub check_updates: bool,
    /// Exact OS IDs confirmed by the user, never inferred from a language name.
    pub selected_layouts: Vec<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            paused: false,
            sounds_enabled: false,
            ui_language: None,
            autostart_enabled: false,
            check_updates: true,
            selected_layouts: Vec::new(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.version != 1 || self.selected_layouts.len() > 32 {
            return Err(SettingsError::Invalid);
        }
        let mut unique = std::collections::HashSet::new();
        for id in &self.selected_layouts {
            if id.is_empty()
                || id.len() > 128
                || id.chars().any(char::is_control)
                || !unique.insert(id)
            {
                return Err(SettingsError::Invalid);
            }
        }
        Ok(())
    }
}
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("settings I/O failed")]
    Io(#[from] std::io::Error),
    #[error("settings are invalid or unsupported; existing preferences were not overwritten")]
    Invalid,
    #[error("settings directory unavailable")]
    NoDirectory,
    #[error("another settings update is in progress")]
    Busy,
}
#[derive(Clone)]
pub struct Store {
    path: PathBuf,
}
impl Store {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    pub fn default_path() -> Result<PathBuf, SettingsError> {
        Ok(dirs::config_dir()
            .ok_or(SettingsError::NoDirectory)?
            .join("typomorph")
            .join("settings.json"))
    }
    pub fn load(&self) -> Result<Settings, SettingsError> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Settings::default()),
            Err(e) => return Err(e.into()),
        };
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(SettingsError::Invalid);
        }
        let settings: Settings =
            serde_json::from_slice(&bytes).map_err(|_| SettingsError::Invalid)?;
        settings.validate()?;
        Ok(settings)
    }
    /// Serialize read-modify-write across processes and atomically publish the file.
    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> Result<Settings, SettingsError> {
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let mut options = OpenOptions::new();
        options.create(true).read(true).write(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options.open(self.path.with_extension("lock"))?;
        lock.try_lock_exclusive().map_err(|e| {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                SettingsError::Busy
            } else {
                SettingsError::Io(e)
            }
        })?;
        let mut settings = self.load()?;
        change(&mut settings);
        settings.validate()?;
        let bytes = serde_json::to_vec_pretty(&settings).map_err(|_| SettingsError::Invalid)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(SettingsError::Invalid);
        }
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        staged.write_all(&bytes)?;
        staged.write_all(b"\n")?;
        staged.as_file().sync_all()?;
        staged
            .persist(&self.path)
            .map_err(|e| SettingsError::Io(e.error))?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        drop(lock);
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_installation_has_no_implicit_layouts_sounds_or_autostart() {
        let d = tempfile::tempdir().unwrap();
        let s = Store::new(d.path().join("prefs.json"));
        assert_eq!(s.load().unwrap(), Settings::default());
        assert!(s.load().unwrap().selected_layouts.is_empty());
        assert!(!s.load().unwrap().sounds_enabled && !s.load().unwrap().autostart_enabled);
    }
    #[test]
    fn pause_and_global_sound_preference_survive_reopening() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("prefs.json");
        let s = Store::new(&p);
        s.update(|x| x.paused = true).unwrap();
        s.update(|x| x.sounds_enabled = true).unwrap();
        let restored = Store::new(&p).load().unwrap();
        assert!(restored.paused && restored.sounds_enabled);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&p).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    #[test]
    fn corrupted_or_future_settings_are_not_overwritten() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("prefs.json");
        let s = Store::new(&p);
        for bytes in [
            b"{invalid SECRET".to_vec(),
            vec![b'x'; MAX_BYTES as usize + 1],
        ] {
            fs::write(&p, &bytes).unwrap();
            assert!(s.update(|x| x.paused = false).is_err());
            assert_eq!(fs::read(&p).unwrap(), bytes);
        }
        let future = Settings {
            version: 2,
            ..Settings::default()
        };
        fs::write(&p, serde_json::to_vec(&future).unwrap()).unwrap();
        assert!(s.load().is_err());
    }
    #[test]
    fn invalid_changes_leave_previous_preferences_intact() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("prefs.json");
        let s = Store::new(&p);
        s.update(|x| x.paused = true).unwrap();
        let before = fs::read(&p).unwrap();
        assert!(s
            .update(|x| x.selected_layouts = vec!["us".into(), "us".into()])
            .is_err());
        assert_eq!(before, fs::read(&p).unwrap());
    }
    #[test]
    fn competing_writer_is_refused_and_lock_releases_on_failure() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("prefs.json");
        let s = Store::new(&p);
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(p.with_extension("lock"))
            .unwrap();
        lock.lock_exclusive().unwrap();
        assert!(matches!(s.update(|_| {}), Err(SettingsError::Busy)));
        drop(lock);
        assert!(s.update(|x| x.version = 9).is_err());
        assert!(s.update(|x| x.paused = true).unwrap().paused);
    }
}
