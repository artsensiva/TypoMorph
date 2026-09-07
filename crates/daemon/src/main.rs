use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::Command;

use clap::{Args, Parser, Subcommand};
use core_engine::{Language, LanguageClassifier, RingBuffer, RING_BUFFER_CAPACITY};
use licensing::{FeatureAccess, LemonSqueezyClient, LicenseError, LicenseStore};
use platform_linux::{
    open_first_keyboard, EvdevKeyboard, GnomeShellSwitcher, LayoutSwitcher, UinputKeyboard,
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
    TestInput(TestInputArgs),
    License {
        #[command(subcommand)]
        command: LicenseCommand,
    },
    Status,
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
    #[arg(
        long,
        help = "Use a specific evdev device instead of automatic keyboard discovery"
    )]
    input: Option<PathBuf>,
    #[arg(long, default_value = "us")]
    layout: String,
    #[arg(long, help = "Classify and report without opening uinput or D-Bus")]
    dry_run: bool,
}

#[derive(Debug)]
struct LayoutDecision {
    language: Language,
    confidence: f64,
    switch: bool,
    target_layout: Option<&'static str>,
    corrected: String,
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
        let decision =
            evaluate_layout_candidates(&text, &layout, &classifier, true, args.threshold);

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
    println!(
        "multi-language profiles: {}",
        access.multi_language_profiles
    );
    println!("developer mode: {}", access.developer_mode);
    Ok(())
}

fn run_daemon(args: RunArgs) -> Result<(), DaemonError> {
    let store = LicenseStore::new(LicenseStore::default_path()?);
    let status = store.load()?;
    let access = FeatureAccess::from_status(status.as_ref());
    let keyboard_result = match args.input {
        Some(path) => EvdevKeyboard::open(path),
        None => open_first_keyboard(),
    }?;
    eprintln!(
        "Selected input device: {} ({})",
        keyboard_result.name().unwrap_or("unnamed keyboard"),
        keyboard_result.path().display()
    );
    let mut keyboard = keyboard_result;
    let mut buffer = RingBuffer::<RING_BUFFER_CAPACITY>::new();
    let classifier = LanguageClassifier::new();
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
    let window_filter = ActiveWindowFilter::default();

    eprintln!(
        "running in {} tier; developer mode {}",
        if status.is_some() { "pro" } else { "free" },
        if access.developer_mode {
            "enabled"
        } else {
            "disabled"
        }
    );

    loop {
        for event in keyboard.next_events()? {
            if !event.pressed || event.repeat {
                continue;
            }
            let Some(character) = keycode_to_character(event.keycode, &current_layout) else {
                continue;
            };
            buffer.push(character);

            let decision = evaluate_layout_candidates(
                &buffer.as_string(),
                &current_layout,
                &classifier,
                access.multi_language_profiles,
                0.60,
            );
            if !decision.switch {
                continue;
            }
            let target_layout = decision.target_layout.expect("switch target exists");
            if access.developer_mode && window_filter.should_bypass() {
                continue;
            }

            if let Some(switcher) = switcher.as_ref() {
                switcher.switch_to(target_layout)?;
            }
            if let Some(emitter) = emitter.as_mut() {
                let replacement = replacement_keycodes(&decision.corrected, target_layout);
                emitter.replace_text(buffer.len(), &replacement)?;
            }
            current_layout = target_layout.to_string();
            buffer = RingBuffer::new();
        }
    }
}

fn evaluate_layout_candidates(
    text: &str,
    current_layout: &str,
    classifier: &LanguageClassifier,
    multi_language_profiles: bool,
    threshold: f64,
) -> LayoutDecision {
    let (original_language, original_confidence) = classifier.classify_with_confidence(text);
    let alternate_layout = if current_layout == "ru" { "us" } else { "ru" };
    let alternate_text = correct_text_for_layout(text, current_layout, alternate_layout);
    let (mapped_language, whole_mapped_confidence) =
        classifier.classify_with_confidence(&alternate_text);
    let (recent_mapped_language, recent_mapped_confidence) = alternate_text
        .split_whitespace()
        .last()
        .map(|word| classifier.classify_with_confidence(word))
        .unwrap_or((mapped_language, whole_mapped_confidence));
    let (mapped_language, mapped_confidence) = if recent_mapped_confidence > whole_mapped_confidence
    {
        (recent_mapped_language, recent_mapped_confidence)
    } else {
        (mapped_language, whole_mapped_confidence)
    };
    let mapped_target = target_layout(mapped_language, multi_language_profiles);
    let coherent_alternate = mapped_confidence >= 0.80;
    let beats_original = coherent_alternate || mapped_confidence > original_confidence + 0.10;
    let switch = alternate_text != text
        && mapped_target.is_some_and(|target| target != current_layout)
        && mapped_confidence >= threshold
        && beats_original;

    if switch {
        LayoutDecision {
            language: mapped_language,
            confidence: mapped_confidence,
            switch: true,
            target_layout: mapped_target,
            corrected: alternate_text,
        }
    } else {
        LayoutDecision {
            language: original_language,
            confidence: original_confidence,
            switch: false,
            target_layout: None,
            corrected: text.to_string(),
        }
    }
}

fn target_layout(language: Language, multi_language_profiles: bool) -> Option<&'static str> {
    match language {
        Language::English => Some("us"),
        Language::Russian => Some("ru"),
        Language::Ukrainian if multi_language_profiles => Some("ua"),
        _ => None,
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

fn is_developer_window(value: &str) -> bool {
    let normalized = value.to_lowercase();
    ["gnome-terminal", "alacritty", "kitty", "code", "clion"]
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn keycode_to_character(keycode: u16, layout: &str) -> Option<char> {
    let index = match keycode {
        16..=25 => usize::from(keycode - 16),
        30..=38 => usize::from(keycode - 30 + 10),
        44..=50 => usize::from(keycode - 44 + 19),
        57 => return Some(' '),
        _ => return None,
    };
    let english = "qwertyuiopasdfghjklzxcvbnm";
    let russian = "йцукенгшщзфывапролдячсмить";
    let characters = if layout == "ru" { russian } else { english };
    characters.chars().nth(index)
}

fn infer_layout_from_text(text: &str) -> Option<&'static str> {
    if text.chars().any(is_cyrillic_character) {
        Some("ru")
    } else if text
        .chars()
        .any(|character| character.is_ascii_alphabetic())
    {
        Some("us")
    } else {
        None
    }
}

fn is_cyrillic_character(character: char) -> bool {
    matches!(character as u32, 0x400..=0x4ff)
}

fn replacement_keycodes(text: &str, target_layout: &str) -> Vec<u16> {
    text.chars()
        .filter_map(|character| keycode_for_character(character, target_layout))
        .collect()
}

fn keycode_for_character(character: char, layout: &str) -> Option<u16> {
    let english = "qwertyuiopasdfghjklzxcvbnm";
    let russian = "йцукенгшщзфывапролдячсмить";
    let characters = if layout == "ru" { russian } else { english };
    let normalized = character.to_lowercase().next()?;
    let index = characters
        .chars()
        .position(|candidate| candidate == normalized)?;
    let keycode = match index {
        0..=9 => 16 + index,
        10..=18 => 30 + index - 10,
        19..=25 => 44 + index - 19,
        _ => return None,
    };
    u16::try_from(keycode).ok()
}

fn correct_text_for_layout(text: &str, source_layout: &str, target_layout: &str) -> String {
    text.chars()
        .map(|character| {
            keycode_for_character(character, source_layout)
                .and_then(|keycode| keycode_to_character(keycode, target_layout))
                .map(|mapped| {
                    if character.is_uppercase() {
                        mapped.to_uppercase().next().unwrap_or(mapped)
                    } else {
                        mapped
                    }
                })
                .unwrap_or(character)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_tier_only_switches_between_english_and_russian() {
        assert_eq!(target_layout(Language::English, false), Some("us"));
        assert_eq!(target_layout(Language::Russian, false), Some("ru"));
        assert_eq!(target_layout(Language::Ukrainian, false), None);
    }

    #[test]
    fn developer_window_markers_bypass_layout_switching() {
        assert!(is_developer_window("Alacritty"));
        assert!(is_developer_window("Visual Studio Code"));
        assert!(!is_developer_window("Firefox"));
    }

    #[test]
    fn keymaps_round_trip_common_letters() {
        for (keycode, character) in [(16, 'q'), (30, 'a'), (44, 'z'), (57, ' ')] {
            assert_eq!(keycode_to_character(keycode, "us"), Some(character));
        }
        assert_eq!(keycode_for_character('й', "ru"), Some(16));
    }

    #[test]
    fn corrected_text_translates_between_layouts() {
        assert_eq!(correct_text_for_layout("руддщ", "ru", "us"), "hello");
        assert_eq!(correct_text_for_layout("FHNTV", "us", "ru"), "АРТЕМ");
        assert_eq!(correct_text_for_layout("FKKJ", "us", "ru"), "АЛЛО");
    }

    #[test]
    fn simulation_infers_source_layout_from_script() {
        assert_eq!(infer_layout_from_text("ghbdtn"), Some("us"));
        assert_eq!(infer_layout_from_text("руддщ"), Some("ru"));
        assert_eq!(infer_layout_from_text("123 !"), None);
    }

    #[test]
    fn dual_candidate_evaluation_switches_us_gibberish_to_russian() {
        let classifier = LanguageClassifier::new();
        let decision = evaluate_layout_candidates("ghbdtn", "us", &classifier, false, 0.75);
        assert!(decision.switch);
        assert_eq!(decision.language, Language::Russian);
        assert_eq!(decision.target_layout, Some("ru"));
        assert_eq!(decision.corrected, "привет");
    }

    #[test]
    fn uppercase_short_input_can_trigger_a_russian_layout_switch() {
        let classifier = LanguageClassifier::new();
        let decision = evaluate_layout_candidates("ALLO", "us", &classifier, false, 0.65);
        assert!(decision.switch);
        assert_eq!(decision.target_layout, Some("ru"));
        assert_eq!(decision.corrected, "ФДДЩ");
    }

    #[test]
    fn dual_candidate_evaluation_switches_russian_gibberish_to_english() {
        let classifier = LanguageClassifier::new();
        let decision = evaluate_layout_candidates("руддщ", "ru", &classifier, false, 0.75);
        assert!(decision.switch);
        assert_eq!(decision.language, Language::English);
        assert_eq!(decision.target_layout, Some("us"));
        assert_eq!(decision.corrected, "hello");
    }

    #[test]
    fn dual_candidate_evaluation_handles_phrases_and_long_words() {
        let classifier = LanguageClassifier::new();
        for (source, expected) in [
            ("ghbdtn vbh", "привет мир"),
            ("ghjuhfvvbhjdfybt", "программирование"),
        ] {
            let decision = evaluate_layout_candidates(source, "us", &classifier, false, 0.65);
            assert!(decision.switch, "expected switch for {source}");
            assert_eq!(decision.corrected, expected);
            assert_eq!(decision.target_layout, Some("ru"));
        }

        let reverse = evaluate_layout_candidates("руддщ цщкдв", "ru", &classifier, false, 0.65);
        assert!(reverse.switch);
        assert_eq!(reverse.language, Language::English);
        assert_eq!(reverse.target_layout, Some("us"));
        assert_eq!(reverse.corrected, "hello world");
    }
}
