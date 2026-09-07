use licensing::{validate_offline, FeatureAccess, LicenseStatus};

#[test]
fn invalid_or_missing_entitlement_keeps_premium_features_disabled() {
    let access = FeatureAccess::from_status(None);
    assert!(!access.multi_language_profiles);
    assert!(!access.developer_mode);
}

#[test]
fn offline_entitlement_requires_the_original_license_key() {
    let status = LicenseStatus {
        active: true,
        license_key_hash: "not-the-key-hash".to_string(),
        instance_id_hash: None,
        product_id: None,
        variant_id: None,
        expires_at: None,
        checked_at_unix: 0,
    };
    assert!(!validate_offline("test-key", &status));
}
