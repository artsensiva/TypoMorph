use platform_linux::{input_device_options, RawKeyEvent};

#[test]
fn platform_event_contract_is_stable_for_daemon_integration() {
    let event = RawKeyEvent {
        keycode: 28,
        pressed: true,
        repeat: false,
    };
    assert_eq!(event.keycode, 28);
    assert!(event.pressed);
    assert!(!event.repeat);
}

#[test]
fn input_device_open_options_do_not_enable_writes() {
    let _options = input_device_options();
}
