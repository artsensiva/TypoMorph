use std::process::Command;

#[test]
fn unsafe_run_refuses_before_bus_license_or_input_setup() {
    let output = Command::new(env!("CARGO_BIN_EXE_typomorph"))
        .arg("run")
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/nonexistent/typomorph-test-bus",
        )
        .env("XDG_CONFIG_HOME", "/nonexistent/typomorph-test-config")
        .output()
        .expect("run CLI without devices");
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("automatic replacement unavailable"),
        "{error}"
    );
    assert!(!error.contains("Listening on"));
    assert!(output.stdout.is_empty());
}

#[test]
fn removed_ai_command_is_not_exposed() {
    let help = Command::new(env!("CARGO_BIN_EXE_typomorph"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(!String::from_utf8(help.stdout)
        .unwrap()
        .contains("improve-prompt"));
    let removed = Command::new(env!("CARGO_BIN_EXE_typomorph"))
        .arg("improve-prompt")
        .output()
        .unwrap();
    assert!(!removed.status.success());
}

#[test]
#[cfg(not(debug_assertions))]
fn release_binary_has_no_live_input_diagnostic_switch() {
    let help = Command::new(env!("CARGO_BIN_EXE_typomorph"))
        .args(["run", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(!String::from_utf8(help.stdout)
        .unwrap()
        .contains("--dry-run"));
    let attempt = Command::new(env!("CARGO_BIN_EXE_typomorph"))
        .args(["run", "--dry-run"])
        .output()
        .unwrap();
    assert!(!attempt.status.success());
    assert!(String::from_utf8(attempt.stderr)
        .unwrap()
        .contains("unexpected argument"));
}
