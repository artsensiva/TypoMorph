mod tray;
use std::io::{self, BufRead, Read};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use clap::{Args, Parser, Subcommand};
use core_engine::layout::{
    evaluate_layout_candidates, infer_layout_from_text, keycode_to_character, replacement_keycodes,
};
use core_engine::prompt_detector::PromptDetector;
use core_engine::prompt_improver::improve_rule_based;
use core_engine::{LanguageClassifier, RingBuffer, RING_BUFFER_CAPACITY};
use licensing::{FeatureAccess, LemonSqueezyClient, LicenseError, LicenseStore};
use platform_linux::{GnomeShellSwitcher, LayoutSwitcher, MultiEvdevKeyboard, UinputKeyboard};
use prompt_cloud::{Backend, PromptCloudClient};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "typomorph",
    version,
    about = "TypoMorph multilingual input daemon"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(RunArgs),
    TestInput(TestInputArgs),
    ImprovePrompt(ImprovePromptArgs),
    License {
        #[command(subcommand)]
        command: LicenseCommand,
    },
    Status,
}

#[derive(Debug, Args)]
struct ImprovePromptArgs {
    #[arg(long, help = "Read the prompt text from stdin")]
    stdin: bool,
    #[arg(
        long,
        help = "Allow sending the prompt to a cloud AI backend (Anthropic API); never happens without this flag"
    )]
    cloud: bool,
    #[arg(
        long,
        help = "Anthropic API key for the free bring-your-own-key cloud path (defaults to $TYPOMORPH_ANTHROPIC_KEY)"
    )]
    api_key: Option<String>,
}

#[derive(Debug, Args)]
struct TestInputArgs {
    #[arg(long, default_value = "us")]
    layout: String,
    #[arg(long, default_value_t = 0.60)]
    threshold: f64,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(long, default_value = "us")]
    layout: String,
    #[arg(long, help = "Classify and report without opening uinput or D-Bus")]
    dry_run: bool,
}

#[derive(Debug, Subcommand)]
enum LicenseCommand {
    Activate {
        key: String,
        #[arg(long, default_value = "typomorph-linux")]
        instance: String,
    },
}

#[derive(Debug, Error)]
enum DaemonError {
    #[error("license operation failed: {0}")]
    License(#[from] LicenseError),
    #[error("platform operation failed: {0}")]
    Platform(#[from] platform_linux::PlatformError),
    #[error("input stream failed: {0}")]
    Input(#[from] std::io::Error),
}

fn main() {
    if let Err(error) = run_cli(Cli::parse()) {
        eprintln!("typomorph: {error}");
        std::process::exit(1);
    }
}

fn run_cli(cli: Cli) -> Result<(), DaemonError> {
    match cli.command {
        Commands::Run(args) => run_daemon(args),
        Commands::TestInput(args) => test_input(args),
        Commands::ImprovePrompt(args) => improve_prompt(args),
        Commands::License { command } => match command {
            LicenseCommand::Activate { key, instance } => activate_license(&key, &instance),
        },
        Commands::Status => print_status(),
    }
}

fn test_input(args: TestInputArgs) -> Result<(), DaemonError> {
    let classifier = LanguageClassifier::new();
    let stdin = io::stdin();
    let mut layout = args.layout;

    eprintln!("interactive input simulation; type text or `KEYS <keycode> ...`, Ctrl-D to exit");
    for line in stdin.lock().lines() {
        let line = line?;
        let characters = if let Some(sequence) = line.strip_prefix("KEYS ") {
            sequence
                .split_whitespace()
                .filter_map(|keycode| keycode.parse::<u16>().ok())
                .filter_map(|keycode| keycode_to_character(keycode, &layout))
                .collect::<String>()
        } else {
            line
        };
        if let Some(inferred_layout) = infer_layout_from_text(&characters) {
            layout = inferred_layout.to_string();
        }
        let mut buffer = RingBuffer::<RING_BUFFER_CAPACITY>::new();
        for character in characters.chars() {
            buffer.push(character);
        }

        let text = buffer.as_string();
        let decision = evaluate_layout_candidates(&text, &layout, &classifier, args.threshold);

        println!(
            "detected={:?} confidence={:.2} switch={} target={:?} corrected={:?}",
            decision.language,
            decision.confidence,
            if decision.switch { "yes" } else { "no" },
            decision.target_layout,
            decision.corrected,
        );
        if decision.switch {
            layout = decision
                .target_layout
                .expect("switch target exists")
                .to_string();
        }
    }
    Ok(())
}

fn improve_prompt(args: ImprovePromptArgs) -> Result<(), DaemonError> {
    if !args.stdin {
        eprintln!("typomorph improve-prompt currently only supports --stdin");
        std::process::exit(2);
    }

    let mut text = String::new();
    io::stdin().lock().read_to_string(&mut text)?;
    let text = text.trim();
    if text.is_empty() {
        eprintln!("no input on stdin");
        return Ok(());
    }

    let signal = PromptDetector::new().detect(text);
    eprintln!(
        "prompt-detected={} confidence={:.2} language={:?}",
        signal.is_prompt, signal.confidence, signal.language
    );

    if !args.cloud {
        println!("{}", improve_rule_based(text));
        eprintln!("mode=local (free, offline, no network)");
        return Ok(());
    }

    let client = PromptCloudClient::new();
    let api_key = args
        .api_key
        .or_else(|| std::env::var("TYPOMORPH_ANTHROPIC_KEY").ok())
        .filter(|key| !key.trim().is_empty());

    if let Some(api_key) = api_key {
        match client.improve(text, Backend::BringYourOwnKey { api_key: &api_key }, false) {
            Ok(improved) => {
                println!("{improved}");
                eprintln!("mode=cloud (bring-your-own-key, free)");
            }
            Err(error) => {
                eprintln!("cloud request failed ({error}); falling back to the local improver");
                println!("{}", improve_rule_based(text));
            }
        }
        return Ok(());
    }

    let store = LicenseStore::new(LicenseStore::default_path()?);
    let is_pro = store.load()?.is_some();
    if !is_pro {
        eprintln!(
            "--cloud requires either TYPOMORPH_ANTHROPIC_KEY (free, bring your own key) or an active Pro license"
        );
        std::process::exit(2);
    }

    match client.improve(text, Backend::Managed, is_pro) {
        Ok(improved) => {
            println!("{improved}");
            eprintln!("mode=cloud (managed, pro)");
        }
        Err(error) => {
            eprintln!("cloud request failed ({error}); falling back to the local improver");
            println!("{}", improve_rule_based(text));
        }
    }
    Ok(())
}

fn activate_license(key: &str, instance: &str) -> Result<(), DaemonError> {
    let client = LemonSqueezyClient::new()?;
    let status = client.activate(key, instance)?;
    let store = LicenseStore::new(LicenseStore::default_path()?);
    store.save(&status)?;
    println!("license activated; premium features enabled");
    Ok(())
}

fn print_status() -> Result<(), DaemonError> {
    let store = LicenseStore::new(LicenseStore::default_path()?);
    let status = store.load()?;
    let access = FeatureAccess::from_status(status.as_ref());
    println!("tier: {}", if status.is_some() { "pro" } else { "free" });
    println!("layout correction: unlimited (free for all languages)");
    println!("developer mode: {}", access.developer_mode);
    Ok(())
}

fn run_daemon(args: RunArgs) -> Result<(), DaemonError> {
    let store = LicenseStore::new(LicenseStore::default_path()?);
    let status = store.load()?;
    let access = FeatureAccess::from_status(status.as_ref());
    let keyboard = MultiEvdevKeyboard::open_all()?;
    let devices = keyboard
        .devices()
        .iter()
        .map(|(name, path)| format!("{} ({})", name, path.display()))
        .collect::<Vec<_>>();
    eprintln!(
        "Listening on {} devices: [{}]",
        devices.len(),
        devices.join(", ")
    );
    let mut buffer = RingBuffer::<RING_BUFFER_CAPACITY>::new();
    let mut scan_codes = Vec::new();
    let classifier = LanguageClassifier::new();
    let prompt_detector = PromptDetector::new();
    let mut current_layout = args.layout;
    let mut emitter = if args.dry_run {
        None
    } else {
        Some(UinputKeyboard::open()?)
    };
    let switcher = if args.dry_run {
        None
    } else {
        Some(GnomeShellSwitcher::connect()?)
    };
    let window_filter = ActiveWindowFilter;
    let is_pro = status.is_some();
    let paused = Arc::new(AtomicBool::new(false));
    tray::spawn_tray(is_pro, Arc::clone(&paused));

    eprintln!(
        "running in {} tier; developer mode {}",
        if status.is_some() { "pro" } else { "free" },
        if access.developer_mode {
            "enabled"
        } else {
            "disabled"
        }
    );

    let mut ctrl_held = false;
    let mut alt_held = false;

    loop {
        if paused.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }
        let event = keyboard.recv().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "all input devices closed",
            )
        })?;

        match event.keycode {
            KEYCODE_LEFT_CTRL | KEYCODE_RIGHT_CTRL => {
                ctrl_held = event.pressed;
                continue;
            }
            KEYCODE_LEFT_ALT | KEYCODE_RIGHT_ALT => {
                alt_held = event.pressed;
                continue;
            }
            KEYCODE_I if event.pressed && !event.repeat && ctrl_held && alt_held => {
                let text = buffer.as_string();
                if !text.is_empty() {
                    let improved = improve_rule_based(&text);
                    eprintln!("Prompt hotkey pressed; improved locally: {improved:?}");
                    notify_send("TypoMorph", &format!("Improved prompt:\n{improved}"));
                }
                continue;
            }
            _ => {}
        }

        if !event.pressed || event.repeat {
            continue;
        }
        let Some(character) = keycode_to_character(event.keycode, &current_layout) else {
            continue;
        };
        eprintln!("Key pressed: {} / {:?}", event.keycode, character);
        // TODO: buffer is only ever reset on a word boundary (whitespace/punctuation),
        // never on a pause. A stray keystroke typed seconds earlier, with no boundary
        // character after it, stays in the buffer and attaches to the next word (e.g.
        // an isolated 's' typed 14s before "привет" becomes "спривет"). Consider also
        // resetting on inactivity (e.g. 3-5s since the last keystroke) to avoid this
        // class of stray-leading-character bug. Separate from the synthetic-echo fix
        // in this same commit — not yet implemented.
        if character.is_whitespace() {
            eprintln!(
                "Detected word boundary, analyzing buffer: {:?}",
                buffer.as_string()
            );
            if buffer.len() < 3 {
                buffer = RingBuffer::new();
                scan_codes.clear();
                continue;
            }

            let decision =
                evaluate_layout_candidates(&buffer.as_string(), &current_layout, &classifier, 0.60);
            if !decision.switch {
                buffer = RingBuffer::new();
                scan_codes.clear();
                continue;
            }

            let prompt_signal = prompt_detector.detect(&buffer.as_string());
            if prompt_signal.is_prompt {
                notify_send(
                    "TypoMorph",
                    "This looks like an AI prompt — press Ctrl+Alt+I to improve it instead of switching layout.",
                );
            }

            let target_layout = decision.target_layout.expect("switch target exists");
            if access.developer_mode && window_filter.should_bypass() {
                buffer = RingBuffer::new();
                scan_codes.clear();
                continue;
            }
            eprintln!(
                "Triggering layout swap: {} -> {}, backspacing {} chars",
                current_layout,
                target_layout,
                buffer.len()
            );

            // Suppressed for the full duration of the swap + emission so that
            // nothing arriving in this window — an echo of our own synthetic
            // keystrokes, or a coincidental real one — reaches the next
            // recv(). Always resumed via `emit_result`, even on error: an
            // early `?` here would leave delivery suppressed forever.
            keyboard.suppress_delivery();
            let emit_result: Result<(), DaemonError> = (|| {
                if let Some(switcher) = switcher.as_ref() {
                    switcher.switch_to(target_layout)?;
                    sleep(Duration::from_millis(15));
                }
                if let Some(emitter) = emitter.as_mut() {
                    let replacement = replacement_keycodes(&decision.corrected, target_layout);
                    emitter.replace_text(scan_codes.len(), &replacement)?;
                }
                Ok(())
            })();
            keyboard.resume_delivery();
            emit_result?;
            current_layout = target_layout.to_string();
            buffer = RingBuffer::new();
            scan_codes.clear();
            continue;
        }

        buffer.push(character);
        scan_codes.push(event.keycode);
    }
}

#[derive(Debug, Default)]
struct ActiveWindowFilter;

impl ActiveWindowFilter {
    fn should_bypass(&self) -> bool {
        let class = xdotool_property("getwindowclassname");
        let title = xdotool_property("getwindowname");
        is_developer_window(&class) || is_developer_window(&title)
    }
}

fn xdotool_property(property: &str) -> String {
    let output = Command::new("xdotool")
        .args(["getactivewindow", property])
        .output();
    output
        .ok()
        .filter(|result| result.status.success())
        .map(|result| String::from_utf8_lossy(&result.stdout).to_lowercase())
        .unwrap_or_default()
}

// Standard Linux evdev keycodes (linux/input-event-codes.h).
const KEYCODE_LEFT_CTRL: u16 = 29;
const KEYCODE_LEFT_ALT: u16 = 56;
const KEYCODE_RIGHT_CTRL: u16 = 97;
const KEYCODE_RIGHT_ALT: u16 = 100;
const KEYCODE_I: u16 = 23;

fn notify_send(summary: &str, body: &str) {
    let _ = Command::new("notify-send").args([summary, body]).status();
}

fn is_developer_window(value: &str) -> bool {
    let normalized = value.to_lowercase();
    ["gnome-terminal", "alacritty", "kitty", "code", "clion"]
        .iter()
        .any(|marker| normalized.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Layout-correction logic itself (keymaps, evaluate_layout_candidates,
    // target_layout, Hindi non-correction) now lives in and is tested by
    // core_engine::layout; these tests cover only what's still daemon-local.

    #[test]
    fn developer_window_markers_bypass_layout_switching() {
        assert!(is_developer_window("Alacritty"));
        assert!(is_developer_window("Visual Studio Code"));
        assert!(!is_developer_window("Firefox"));
    }
}
