//! Offline verification only. No network, hardware fingerprint or signing secret.
//! Trust roots must come from the application, never from an entitlement file.
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

pub const MAX_TOKEN_BYTES: usize = 8192;
pub const TRIAL_SECONDS: u64 = 7 * 24 * 60 * 60;
pub const GRACE_SECONDS: u64 = 7 * 24 * 60 * 60;
pub const SIGNING_DOMAIN: &[u8] = b"TypoMorph offline entitlement v1\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Trial,
    Annual,
    Perpetual,
}

/// This is a wire payload, NOT authorization until verified.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claims {
    pub version: u32,
    pub audience: String,
    pub account_id: String,
    pub installation_id: String,
    pub plan: Plan,
    pub issued_at: u64,
    pub not_before: u64,
    pub expires_at: Option<u64>,
    /// The original account-wide trial start, also for later device activations.
    pub trial_started_at: Option<u64>,
    /// Issued by the service only after a confirmed failed annual renewal.
    pub renewal_grace_until: Option<u64>,
}

/// Signature covers domain + exact UTF-8 payload bytes; do not reserialize first.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedEntitlement {
    pub key_id: String,
    pub payload: String,
    pub signature_hex: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntitlementError {
    #[error("entitlement is malformed or too large")]
    Malformed,
    #[error("entitlement signing key is not trusted")]
    UnknownKey,
    #[error("entitlement signature is invalid")]
    InvalidSignature,
    #[error("entitlement claims are invalid")]
    InvalidClaims,
    #[error("entitlement belongs to another account or installation")]
    WrongBinding,
    #[error("entitlement is not yet valid")]
    NotYetValid,
    #[error("entitlement has expired")]
    Expired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Trial,
    Annual,
    RenewalGrace,
    Perpetual,
}

pub struct Verifier {
    keys: BTreeMap<String, VerifyingKey>,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

impl Verifier {
    pub fn new(
        keys: impl IntoIterator<Item = (String, [u8; 32])>,
    ) -> Result<Self, EntitlementError> {
        let mut trusted = BTreeMap::new();
        for (id, bytes) in keys {
            if !valid_id(&id) {
                return Err(EntitlementError::InvalidClaims);
            }
            let key =
                VerifyingKey::from_bytes(&bytes).map_err(|_| EntitlementError::InvalidSignature)?;
            if key.is_weak() || trusted.insert(id, key).is_some() {
                return Err(EntitlementError::InvalidSignature);
            }
        }
        Ok(Self { keys: trusted })
    }

    /// Caller supplies current UTC seconds and its authenticated account/device.
    /// This does not solve hostile local clock rollback or immediate offline revocation.
    pub fn verify(
        &self,
        token: &[u8],
        account: &str,
        installation: &str,
        now: u64,
    ) -> Result<Access, EntitlementError> {
        if token.is_empty() || token.len() > MAX_TOKEN_BYTES {
            return Err(EntitlementError::Malformed);
        }
        let envelope: SignedEntitlement =
            serde_json::from_slice(token).map_err(|_| EntitlementError::Malformed)?;
        let key = self
            .keys
            .get(&envelope.key_id)
            .ok_or(EntitlementError::UnknownKey)?;
        let signature = decode_signature(&envelope.signature_hex)?;
        let mut message = Vec::with_capacity(SIGNING_DOMAIN.len() + envelope.payload.len());
        message.extend_from_slice(SIGNING_DOMAIN);
        message.extend_from_slice(envelope.payload.as_bytes());
        key.verify_strict(&message, &Signature::from_bytes(&signature))
            .map_err(|_| EntitlementError::InvalidSignature)?;
        let claims: Claims =
            serde_json::from_str(&envelope.payload).map_err(|_| EntitlementError::InvalidClaims)?;
        if claims.version != 1
            || claims.audience != "typomorph-desktop"
            || !valid_id(&claims.account_id)
            || !valid_id(&claims.installation_id)
        {
            return Err(EntitlementError::InvalidClaims);
        }
        if claims.account_id != account || claims.installation_id != installation {
            return Err(EntitlementError::WrongBinding);
        }
        if claims.issued_at > now || claims.not_before > now {
            return Err(EntitlementError::NotYetValid);
        }
        match claims.plan {
            Plan::Perpetual => {
                if claims.expires_at.is_some()
                    || claims.trial_started_at.is_some()
                    || claims.renewal_grace_until.is_some()
                {
                    return Err(EntitlementError::InvalidClaims);
                }
                Ok(Access::Perpetual)
            }
            Plan::Trial => {
                let start = claims
                    .trial_started_at
                    .ok_or(EntitlementError::InvalidClaims)?;
                let end = start
                    .checked_add(TRIAL_SECONDS)
                    .ok_or(EntitlementError::InvalidClaims)?;
                if claims.expires_at != Some(end)
                    || claims.renewal_grace_until.is_some()
                    || claims.not_before < start
                    || claims.not_before >= end
                    || claims.issued_at < start
                {
                    return Err(EntitlementError::InvalidClaims);
                }
                if now >= end {
                    Err(EntitlementError::Expired)
                } else {
                    Ok(Access::Trial)
                }
            }
            Plan::Annual => {
                let end = claims.expires_at.ok_or(EntitlementError::InvalidClaims)?;
                if claims.trial_started_at.is_some() || end <= claims.not_before {
                    return Err(EntitlementError::InvalidClaims);
                }
                if let Some(grace) = claims.renewal_grace_until {
                    if end.checked_add(GRACE_SECONDS) != Some(grace) || claims.issued_at < end {
                        return Err(EntitlementError::InvalidClaims);
                    }
                    if now >= grace {
                        return Err(EntitlementError::Expired);
                    }
                    return Ok(Access::RenewalGrace);
                }
                if now >= end {
                    Err(EntitlementError::Expired)
                } else {
                    Ok(Access::Annual)
                }
            }
        }
    }
}

fn decode_signature(hex: &str) -> Result<[u8; 64], EntitlementError> {
    if hex.len() != 128 || !hex.is_ascii() {
        return Err(EntitlementError::Malformed);
    }
    let mut result = [0; 64];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .map_err(|_| EntitlementError::Malformed)?;
    }
    Ok(result)
}
