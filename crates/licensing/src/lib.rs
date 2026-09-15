use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const DEFAULT_LICENSE_ENDPOINT: &str = "https://api.lemonsqueezy.com/v1/licenses/activate";

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("license key cannot be empty")]
    EmptyKey,
    #[error("license configuration directory is unavailable")]
    ConfigDirectoryUnavailable,
    #[error("license configuration I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("license configuration is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("license service request failed: {0}")]
    Network(String),
    #[error("license service rejected the key: {0}")]
    Rejected(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub active: bool,
    pub license_key_hash: String,
    pub instance_id_hash: Option<String>,
    pub product_id: Option<u64>,
    pub variant_id: Option<u64>,
    pub expires_at: Option<String>,
    pub checked_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct StoredLicense {
    status: LicenseStatus,
    integrity_hash: String,
}

impl StoredLicense {
    fn new(status: LicenseStatus) -> Self {
        let integrity_hash =
            hash_bytes(&serde_json::to_vec(&status).expect("status is serializable"));
        Self {
            status,
            integrity_hash,
        }
    }

    fn is_integrity_valid(&self) -> bool {
        let expected =
            hash_bytes(&serde_json::to_vec(&self.status).expect("status is serializable"));
        expected == self.integrity_hash
    }
}

#[derive(Debug, Clone)]
pub struct LicenseStore {
    path: PathBuf,
}

impl LicenseStore {
    pub fn default_path() -> Result<PathBuf, LicenseError> {
        let config_dir = dirs::config_dir().ok_or(LicenseError::ConfigDirectoryUnavailable)?;
        Ok(config_dir.join("typomorph").join("license.json"))
    }

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Option<LicenseStatus>, LicenseError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&self.path)?;
        let stored: StoredLicense = serde_json::from_slice(&bytes)?;
        if !stored.is_integrity_valid() {
            return Ok(None);
        }
        Ok(Some(stored.status))
    }

    pub fn save(&self, status: &LicenseStatus) -> Result<(), LicenseError> {
        let parent = self
            .path
            .parent()
            .ok_or(LicenseError::ConfigDirectoryUnavailable)?;
        fs::create_dir_all(parent)?;
        let mut options = OpenOptions::new();
        options.create(true).truncate(true).write(true);
        let mut file = options.open(&self.path)?;
        file.write_all(
            serde_json::to_string_pretty(&StoredLicense::new(status.clone()))?.as_bytes(),
        )?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        set_private_permissions(&file)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureAccess {
    pub developer_mode: bool,
}

impl FeatureAccess {
    pub fn from_status(status: Option<&LicenseStatus>) -> Self {
        let licensed = status.is_some_and(|value| value.active);
        Self {
            developer_mode: licensed,
        }
    }
}

pub trait LicenseTransport {
    fn post_form(&self, endpoint: &str, fields: &[(&str, &str)]) -> Result<String, LicenseError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UreqTransport;

impl LicenseTransport for UreqTransport {
    fn post_form(&self, endpoint: &str, fields: &[(&str, &str)]) -> Result<String, LicenseError> {
        ureq::post(endpoint)
            .send_form(fields)
            .map_err(|error| LicenseError::Network(error.to_string()))?
            .into_string()
            .map_err(|error| LicenseError::Network(error.to_string()))
    }
}

pub struct LemonSqueezyClient<T = UreqTransport> {
    endpoint: String,
    transport: T,
}

impl LemonSqueezyClient<UreqTransport> {
    pub fn new() -> Result<Self, LicenseError> {
        Ok(Self {
            endpoint: DEFAULT_LICENSE_ENDPOINT.to_string(),
            transport: UreqTransport,
        })
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Result<Self, LicenseError> {
        Ok(Self {
            endpoint: endpoint.into(),
            transport: UreqTransport,
        })
    }
}

impl<T: LicenseTransport> LemonSqueezyClient<T> {
    pub fn with_transport(endpoint: impl Into<String>, transport: T) -> Self {
        Self {
            endpoint: endpoint.into(),
            transport,
        }
    }

    pub fn activate(
        &self,
        license_key: &str,
        instance_name: &str,
    ) -> Result<LicenseStatus, LicenseError> {
        if license_key.trim().is_empty() {
            return Err(LicenseError::EmptyKey);
        }
        let response: LemonResponse = serde_json::from_str(&self.transport.post_form(
            &self.endpoint,
            &[
                ("license_key", license_key),
                ("instance_name", instance_name),
            ],
        )?)?;
        if !response.activated {
            return Err(LicenseError::Rejected(
                response
                    .error
                    .unwrap_or_else(|| "activation failed".to_string()),
            ));
        }
        Ok(LicenseStatus {
            active: true,
            license_key_hash: hash_text(license_key),
            instance_id_hash: response
                .license_key
                .as_ref()
                .and_then(|key| key.id.map(|id| hash_text(&id.to_string()))),
            product_id: response.meta.as_ref().and_then(|meta| meta.product_id),
            variant_id: response.meta.as_ref().and_then(|meta| meta.variant_id),
            expires_at: response
                .license_key
                .as_ref()
                .and_then(|key| key.expires_at.clone()),
            checked_at_unix: unix_now(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct LemonResponse {
    activated: bool,
    error: Option<String>,
    license_key: Option<LemonLicenseKey>,
    meta: Option<LemonMeta>,
}

#[derive(Debug, Deserialize)]
struct LemonLicenseKey {
    id: Option<u64>,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LemonMeta {
    product_id: Option<u64>,
    variant_id: Option<u64>,
}

pub fn validate_offline(license_key: &str, status: &LicenseStatus) -> bool {
    !license_key.trim().is_empty()
        && status.active
        && status.license_key_hash == hash_text(license_key)
}

fn hash_text(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

fn hash_bytes(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(unix)]
fn set_private_permissions(file: &File) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_permissions(_file: &File) -> Result<(), std::io::Error> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_for(key: &str) -> LicenseStatus {
        LicenseStatus {
            active: true,
            license_key_hash: hash_text(key),
            instance_id_hash: None,
            product_id: Some(1),
            variant_id: Some(2),
            expires_at: None,
            checked_at_unix: 1,
        }
    }

    #[test]
    fn offline_validation_and_feature_gates_require_the_same_key() {
        let status = status_for("test-key");
        assert!(validate_offline("test-key", &status));
        assert!(!validate_offline("other-key", &status));
        assert!(!FeatureAccess::from_status(None).developer_mode);
        assert!(FeatureAccess::from_status(Some(&status)).developer_mode);
    }

    #[test]
    fn license_store_round_trips_hashed_status_with_private_permissions() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("license.json");
        let store = LicenseStore::new(&path);
        let status = status_for("test-key");
        store.save(&status).expect("save status");
        let loaded = store.load().expect("load status").expect("status exists");
        assert_eq!(loaded, status);
        let file_text = fs::read_to_string(path).expect("read config");
        assert!(!file_text.contains("test-key"));
        assert!(file_text.contains("license_key_hash"));
    }

    struct MockTransport;

    impl LicenseTransport for MockTransport {
        fn post_form(
            &self,
            _endpoint: &str,
            fields: &[(&str, &str)],
        ) -> Result<String, LicenseError> {
            assert_eq!(
                fields,
                [("license_key", "test-key"), ("instance_name", "test")]
            );
            Ok(r#"{"activated":true,"license_key":{"id":42,"expires_at":null},"meta":{"product_id":7,"variant_id":9}}"#.to_string())
        }
    }

    #[test]
    fn online_activation_can_use_a_mock_transport() {
        let client = LemonSqueezyClient::with_transport("https://license.test", MockTransport);
        let status = client
            .activate("test-key", "test")
            .expect("activation succeeds");
        assert!(status.active);
        assert_eq!(status.product_id, Some(7));
        assert!(validate_offline("test-key", &status));
    }
}
