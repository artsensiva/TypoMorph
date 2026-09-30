use ed25519_dalek::{Signer, SigningKey};
use licensing::*;

fn key() -> SigningKey {
    SigningKey::from_bytes(&[42; 32])
} // Test-only key, never a production trust root.
fn verifier() -> Verifier {
    Verifier::new([("test".into(), key().verifying_key().to_bytes())]).unwrap()
}
fn claims(plan: Plan) -> Claims {
    Claims {
        version: 1,
        audience: "typomorph-desktop".into(),
        account_id: "account-1".into(),
        installation_id: "device-1".into(),
        plan,
        issued_at: 100,
        not_before: 100,
        expires_at: match plan {
            Plan::Trial => Some(100 + TRIAL_SECONDS),
            Plan::Annual => Some(1000),
            Plan::Perpetual => None,
        },
        trial_started_at: (plan == Plan::Trial).then_some(100),
        renewal_grace_until: None,
    }
}
fn token(c: &Claims) -> Vec<u8> {
    let payload = serde_json::to_string(c).unwrap();
    let mut message = SIGNING_DOMAIN.to_vec();
    message.extend_from_slice(payload.as_bytes());
    let signature = key().sign(&message);
    serde_json::to_vec(&SignedEntitlement {
        key_id: "test".into(),
        payload,
        signature_hex: signature
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    })
    .unwrap()
}
fn verify(c: &Claims, now: u64) -> Result<Access, EntitlementError> {
    verifier().verify(&token(c), "account-1", "device-1", now)
}

#[test]
fn confirmed_periods_work_offline_with_exclusive_expiry() {
    for (plan, access, end) in [
        (Plan::Trial, Access::Trial, 100 + TRIAL_SECONDS),
        (Plan::Annual, Access::Annual, 1000),
    ] {
        let c = claims(plan);
        assert_eq!(verify(&c, 100), Ok(access));
        assert_eq!(verify(&c, end - 1), Ok(access));
        assert_eq!(verify(&c, end), Err(EntitlementError::Expired));
    }
    assert_eq!(
        verify(&claims(Plan::Perpetual), u64::MAX),
        Ok(Access::Perpetual)
    );
}
#[test]
fn additional_device_activation_does_not_restart_trial() {
    let mut c = claims(Plan::Trial);
    c.issued_at = 100 + TRIAL_SECONDS - 50;
    c.not_before = c.issued_at;
    assert_eq!(verify(&c, c.issued_at), Ok(Access::Trial));
    assert_eq!(
        verify(&c, 100 + TRIAL_SECONDS),
        Err(EntitlementError::Expired)
    );
    c.expires_at = Some(c.issued_at + TRIAL_SECONDS);
    assert_eq!(
        verify(&c, c.issued_at),
        Err(EntitlementError::InvalidClaims)
    );
}
#[test]
fn grace_requires_separately_signed_failed_renewal_evidence() {
    let mut c = claims(Plan::Annual);
    assert_eq!(verify(&c, 1000), Err(EntitlementError::Expired));
    c.issued_at = 1000;
    c.renewal_grace_until = Some(1000 + GRACE_SECONDS);
    assert_eq!(verify(&c, 1000), Ok(Access::RenewalGrace));
    assert_eq!(
        verify(&c, 1000 + GRACE_SECONDS),
        Err(EntitlementError::Expired)
    );
    c.issued_at = 999;
    assert_eq!(verify(&c, 1000), Err(EntitlementError::InvalidClaims));
}
#[test]
fn signature_and_identity_tampering_are_rejected() {
    let bytes = token(&claims(Plan::Perpetual));
    let mut envelope: SignedEntitlement = serde_json::from_slice(&bytes).unwrap();
    envelope.payload = envelope.payload.replace("account-1", "account-2");
    assert_eq!(
        verifier().verify(
            &serde_json::to_vec(&envelope).unwrap(),
            "account-2",
            "device-1",
            100
        ),
        Err(EntitlementError::InvalidSignature)
    );
    assert_eq!(
        verifier().verify(&bytes, "account-1", "device-2", 100),
        Err(EntitlementError::WrongBinding)
    );
    assert_eq!(
        Verifier::new([])
            .unwrap()
            .verify(&bytes, "account-1", "device-1", 100),
        Err(EntitlementError::UnknownKey)
    );
}
#[test]
fn malformed_future_and_inconsistent_grants_fail_closed() {
    assert_eq!(
        verifier().verify(&vec![b'x'; MAX_TOKEN_BYTES + 1], "a", "b", 100),
        Err(EntitlementError::Malformed)
    );
    assert_eq!(
        verify(&claims(Plan::Annual), 99),
        Err(EntitlementError::NotYetValid)
    );
    let mut c = claims(Plan::Perpetual);
    c.expires_at = Some(1000);
    assert_eq!(verify(&c, 100), Err(EntitlementError::InvalidClaims));
    c = claims(Plan::Trial);
    c.renewal_grace_until = Some(1000);
    assert_eq!(verify(&c, 100), Err(EntitlementError::InvalidClaims));
    c = claims(Plan::Annual);
    c.audience = "another-product".into();
    assert_eq!(verify(&c, 100), Err(EntitlementError::InvalidClaims));
}
#[test]
fn arbitrary_signature_is_not_a_checksum_entitlement() {
    let mut e: SignedEntitlement = serde_json::from_slice(&token(&claims(Plan::Annual))).unwrap();
    e.signature_hex = "00".repeat(64);
    assert_eq!(
        verifier().verify(
            &serde_json::to_vec(&e).unwrap(),
            "account-1",
            "device-1",
            100
        ),
        Err(EntitlementError::InvalidSignature)
    );
}
