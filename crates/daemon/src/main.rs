mod tray;
use std::io::{self, BufRead};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::{Args, Parser, Subcommand};
use core_engine::context::ContextGuard;
use core_engine::layout::{
    evaluate_layout_candidates, infer_layout_from_text, keycode_to_character,
};
use core_engine::replacement::plan_word_replacement;
use core_engine::{LanguageClassifier, RingBuffer, RING_BUFFER_CAPACITY};
use platform_linux::{
    require_safe_replacement_backend, GnomeShellSwitcher, InputContext, MultiEvdevKeyboard,
};
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
    /// Check layout-backend access without capturing keys or changing the layout.
    CheckLayoutBackend,
    /// Inspect field eligibility using metadata only; never read text or capture keys.
    CheckInputContext,
    TestInput(TestInputArgs),
    Status,
    /// Persist pause without opening input devices.
    Pause,
    /// Clear persistent pause; does not start input capture.
    Resume,
    /// Set the global sound preference (off by default).
    Sounds {
        #[arg(action = clap::ArgAction::Set, value_parser = clap::value_parser!(bool))]
        enabled: bool,
    },
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
    #[cfg_attr(
        debug_assertions,
        arg(
            long,
            help = "Controlled live-input diagnostic; requires field metadata and never changes text"
        )
    )]
    #[cfg_attr(not(debug_assertions), arg(skip))]
    dry_run: bool,
    #[arg(
        long,
        help = "Report fixed processing-stage labels without input content"
    )]
    diagnostics: bool,
}

#[derive(Debug, Error)]
enum DaemonError {
    #[error("{0}")]
    Settings(#[from] settings::SettingsError),
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
        Commands::CheckLayoutBackend => {
            GnomeShellSwitcher::connect()?;
            println!("layout backend access available; no capture or layout change performed");
            Ok(())
        }
        Commands::CheckInputContext => {
            let context = InputContext::connect().ok_or_else(|| {
                platform_linux::PlatformError::UnsupportedBackend(
                    "AT-SPI metadata unavailable".into(),
                )
            })?;
            context.check().map_err(|code| {
                platform_linux::PlatformError::UnsupportedBackend(format!(
                    "field metadata rejected: {code}"
                ))
            })?;
            println!("eligible field metadata available; no text read or input capture performed");
            Ok(())
        }
        Commands::TestInput(args) => test_input(args),
        Commands::Status => print_status(),
        Commands::Pause | Commands::Resume => {
            let paused = matches!(cli.command, Commands::Pause);
            settings::Store::new(settings::Store::default_path()?).update(|s| s.paused = paused)?;
            println!("persistent pause: {paused}");
            Ok(())
        }
        Commands::Sounds { enabled } => {
            settings::Store::new(settings::Store::default_path()?)
                .update(|s| s.sounds_enabled = enabled)?;
            println!("sounds enabled: {enabled}");
            Ok(())
        }
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

fn print_status() -> Result<(), DaemonError> {
    let preferences = settings::Store::new(settings::Store::default_path()?).load()?;
    println!("persistent pause: {}", preferences.paused);
    println!("sounds enabled: {}", preferences.sounds_enabled);
    println!("automatic correction: unavailable (safe application integration pending)");
    println!("account activation: unavailable (version 1 service integration pending)");
    Ok(())
}

fn run_daemon(args: RunArgs) -> Result<(), DaemonError> {
    // Must precede license I/O, accessibility setup, capture, tray and layout changes.
    if !args.dry_run {
        require_safe_replacement_backend()?;
    }
    let store = settings::Store::new(settings::Store::default_path()?);
    let preferences = store.load()?;
    if preferences.paused {
        println!("paused; no input devices opened");
        return Ok(());
    }
    let current_layout = args.layout;
    let input_context = InputContext::connect().ok_or_else(|| {
        platform_linux::PlatformError::UnsupportedBackend("AT-SPI metadata unavailable".into())
    })?;
    let initial_context = input_context.snapshot().ok_or_else(|| {
        platform_linux::PlatformError::UnsupportedBackend(
            "no eligible field metadata; input capture was not started".into(),
        )
    })?;
    let mut context_guard = ContextGuard::new(Some(initial_context));
    let mut keyboard = Some(MultiEvdevKeyboard::open_all()?);
    eprintln!("controlled input observation started");
    let mut buffer = RingBuffer::<RING_BUFFER_CAPACITY>::new();
    let mut observed_key_count = 0usize;
    let classifier = LanguageClassifier::new();
    let window_filter = ActiveWindowFilter;
    let paused = Arc::new(AtomicBool::new(false));
    tray::spawn_tray(Arc::clone(&paused), store.clone());

    eprintln!("controlled diagnostic mode; corrections disabled");

    let mut input_seen = false;
    let mut held_modifiers = std::collections::HashSet::new();

    let mut next_preferences = std::time::Instant::now();
    loop {
        if std::time::Instant::now() >= next_preferences {
            // Any unreadable preference state ends capture via RAII rather than
            // silently ignoring a possibly requested pause.
            paused.store(store.load()?.paused, Ordering::SeqCst);
            next_preferences = std::time::Instant::now() + std::time::Duration::from_millis(100);
        }
        if paused.load(Ordering::SeqCst) {
            drop(keyboard.take());
            buffer = RingBuffer::new();
            observed_key_count = 0;
            held_modifiers.clear();
            context_guard.clear();
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }
        if keyboard.is_none() {
            // Resume requires fresh eligible context and creates an empty queue.
            let Some(context) = input_context.snapshot() else {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            };
            context_guard = ContextGuard::new(Some(context));
            keyboard = Some(MultiEvdevKeyboard::open_all()?);
        }
        let event = match keyboard
            .as_ref()
            .expect("capture opened")
            .recv_timeout(std::time::Duration::from_millis(50))
        {
            Ok(event) => event,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "diagnostic input stream stopped; device loss or queue overflow",
                )
                .into())
            }
        };

        if !input_seen {
            report_stage(args.diagnostics, DiagnosticStage::InputReceived);
            input_seen = true;
        }

        if event.repeat
            || (event.pressed
                && matches!(event.keycode, 29 | 97 | 56 | 100 | 42 | 54 | 125 | 126 | 58))
        {
            buffer = RingBuffer::new();
            observed_key_count = 0;
            context_guard.clear();
        }
        // Do not interpret command chords or shifted input with the prototype's
        // unmodified letter tables; retain left/right modifier states separately.
        if matches!(event.keycode, 29 | 97 | 56 | 100 | 42 | 54 | 125 | 126) {
            if event.pressed {
                held_modifiers.insert(event.keycode);
            } else if !event.repeat {
                held_modifiers.remove(&event.keycode);
            }
            buffer = RingBuffer::new();
            observed_key_count = 0;
            context_guard.clear();
            continue;
        }

        if !event.pressed || event.repeat {
            continue;
        }
        let Some(character) = keycode_to_character(event.keycode, &current_layout) else {
            buffer = RingBuffer::new();
            observed_key_count = 0;
            context_guard.clear();
            continue;
        };
        if !held_modifiers.is_empty() || !context_guard.observe_insertion(input_context.snapshot())
        {
            buffer = RingBuffer::new();
            observed_key_count = 0;
            if !held_modifiers.is_empty() {
                context_guard.clear();
            }
            report_stage(args.diagnostics, DiagnosticStage::ContextInvalidated);
            continue;
        };

        // TODO: buffer is only ever reset on a word boundary (whitespace/punctuation),
        // never on a pause. A stray keystroke typed seconds earlier, with no boundary
        // character after it, stays in the buffer and attaches to the next word (e.g.
        // an isolated 's' typed 14s before "привет" becomes "спривет"). Consider also
        // resetting on inactivity (e.g. 3-5s since the last keystroke) to avoid this
        // class of stray-leading-character bug. Separate from the synthetic-echo fix
        // in this same commit — not yet implemented.
        if character.is_whitespace() {
            report_stage(args.diagnostics, DiagnosticStage::BoundaryReceived);
            if buffer.len() < 3 {
                report_stage(args.diagnostics, DiagnosticStage::InsufficientInput);
                buffer = RingBuffer::new();
                observed_key_count = 0;
                continue;
            }

            let decision =
                evaluate_layout_candidates(&buffer.as_string(), &current_layout, &classifier, 0.60);
            if !decision.switch {
                report_stage(args.diagnostics, DiagnosticStage::NoCandidate);
                buffer = RingBuffer::new();
                observed_key_count = 0;
                continue;
            }

            let target_layout = decision.target_layout.expect("switch target exists");
            let Some(_replacement) = plan_word_replacement(
                &buffer.as_string(),
                observed_key_count,
                &current_layout,
                &decision.corrected,
                target_layout,
                character,
            ) else {
                report_stage(
                    args.diagnostics,
                    DiagnosticStage::UnrepresentableReplacement,
                );
                buffer = RingBuffer::new();
                observed_key_count = 0;
                continue;
            };
            if window_filter.should_bypass() {
                report_stage(args.diagnostics, DiagnosticStage::ExcludedApplication);
                buffer = RingBuffer::new();
                observed_key_count = 0;
                continue;
            }
            report_stage(args.diagnostics, DiagnosticStage::CorrectionCandidate);

            // Analysis only. Never switch the physical source, suppress genuine
            // input, or invoke the legacy sequential uinput replacement path.
            context_guard.clear();
            buffer = RingBuffer::new();
            observed_key_count = 0;
            continue;
        }

        buffer.push(character);
        observed_key_count = (observed_key_count + 1).min(RING_BUFFER_CAPACITY + 1);
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

// Stages carry no input, keycodes, layout names, device IDs, or event timestamps.
// A returned OS call is deliberately not described as verified text replacement.
#[derive(Clone, Copy)]
enum DiagnosticStage {
    InputReceived,
    BoundaryReceived,
    InsufficientInput,
    NoCandidate,
    ExcludedApplication,
    CorrectionCandidate,
    UnrepresentableReplacement,
    ContextInvalidated,
}

impl DiagnosticStage {
    fn label(self) -> &'static str {
        match self {
            Self::InputReceived => "input_received",
            Self::BoundaryReceived => "boundary_received",
            Self::InsufficientInput => "insufficient_input",
            Self::NoCandidate => "no_candidate",
            Self::ExcludedApplication => "excluded_application",
            Self::CorrectionCandidate => "correction_candidate",
            Self::UnrepresentableReplacement => "unrepresentable_replacement",
            Self::ContextInvalidated => "context_invalidated",
        }
    }
}

fn report_stage(enabled: bool, stage: DiagnosticStage) {
    if enabled {
        eprintln!("typomorph stage={}", stage.label());
    }
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
