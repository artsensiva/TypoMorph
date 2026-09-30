use platform_linux::{GnomeShellSwitcher, LayoutSwitcher};

// Run only with the supplied GJS fake on a private dbus-run-session bus.
#[test]
#[ignore = "requires integrations/gnome/tests/run_dbus_test.py"]
fn gjs_protocol_confirms_switch_and_rejects_stale_or_missing_source() {
    assert_eq!(
        std::env::var("TYPOMORPH_PRIVATE_BRIDGE_TEST").as_deref(),
        Ok("1")
    );
    let bridge = GnomeShellSwitcher::connect().unwrap();
    assert_eq!(bridge.current_layout().unwrap(), "us");
    bridge.switch_from_to("us", "ru").unwrap();
    assert_eq!(bridge.current_layout().unwrap(), "ru");
    assert!(bridge.switch_from_to("us", "ru").is_err());
    assert_eq!(bridge.current_layout().unwrap(), "ru");
    assert!(bridge.switch_from_to("ru", "de").is_err());
    assert_eq!(bridge.current_layout().unwrap(), "ru");
    bridge.switch_from_to("ru", "us").unwrap();
    assert_eq!(bridge.current_layout().unwrap(), "us");
}
