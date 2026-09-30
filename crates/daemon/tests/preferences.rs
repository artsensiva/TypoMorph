use std::process::{Command, Output};

fn cli(config: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_typomorph"))
        .args(args)
        .env("XDG_CONFIG_HOME", config)
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/nonexistent/typomorph-test-bus",
        )
        .output()
        .unwrap()
}

#[test]
#[cfg(debug_assertions)]
fn persistent_pause_prevents_diagnostic_setup_across_processes() {
    let config = tempfile::tempdir().unwrap();
    assert!(cli(config.path(), &["pause"]).status.success());
    let paused = cli(config.path(), &["run", "--dry-run"]);
    assert!(
        paused.status.success(),
        "{}",
        String::from_utf8_lossy(&paused.stderr)
    );
    assert!(String::from_utf8(paused.stdout)
        .unwrap()
        .contains("no input devices opened"));
    assert!(paused.stderr.is_empty());
    assert!(cli(config.path(), &["sounds", "true"]).status.success());
    let status = cli(config.path(), &["status"]);
    assert!(status.status.success());
    let status = String::from_utf8(status.stdout).unwrap();
    assert!(status.contains("persistent pause: true"));
    assert!(status.contains("sounds enabled: true"));
    assert!(cli(config.path(), &["resume"]).status.success());
    let status = cli(config.path(), &["status"]);
    assert!(String::from_utf8(status.stdout)
        .unwrap()
        .contains("persistent pause: false"));
    // Resume changes preferences only; the production safety gate remains in force.
    let run = cli(config.path(), &["run"]);
    assert!(!run.status.success());
    assert!(String::from_utf8(run.stderr)
        .unwrap()
        .contains("automatic replacement unavailable"));
}

#[test]
fn corrupt_preferences_fail_before_diagnostic_setup_without_echoing_contents() {
    let config = tempfile::tempdir().unwrap();
    let directory = config.path().join("typomorph");
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("settings.json");
    std::fs::write(&path, b"SECRET invalid settings").unwrap();
    let mut commands = vec![vec!["resume"], vec!["sounds", "true"]];
    if cfg!(debug_assertions) {
        commands.push(vec!["run", "--dry-run"]);
    }
    for args in commands {
        let output = cli(config.path(), &args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("settings are invalid"));
        assert!(!error.contains("SECRET"));
        assert!(!error.contains("AT-SPI"));
        assert_eq!(std::fs::read(&path).unwrap(), b"SECRET invalid settings");
    }
}
