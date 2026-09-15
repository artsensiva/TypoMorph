//! Pure keyboard-layout text correction: mapping raw QWERTY-row physical key
//! positions to the character each supported layout produces there, and
//! deciding whether a buffer of text was typed under the wrong layout. No I/O,
//! no OS dependency — usable by the Linux daemon and by any other frontend
//! (e.g. a browser-extension native host) that only has plain text to work
//! with.

use crate::{Language, LanguageClassifier};

#[derive(Debug)]
pub struct LayoutDecision {
    pub language: Language,
    pub confidence: f64,
    pub switch: bool,
    pub target_layout: Option<&'static str>,
    pub corrected: String,
}

pub fn evaluate_layout_candidates(
    text: &str,
    current_layout: &str,
    classifier: &LanguageClassifier,
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
    let mapped_target = target_layout(mapped_language);
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

pub fn target_layout(language: Language) -> Option<&'static str> {
    match language {
        Language::English => Some("us"),
        Language::Russian => Some("ru"),
        Language::Ukrainian => Some("ua"),
        _ => None,
    }
}

pub fn keycode_to_character(keycode: u16, layout: &str) -> Option<char> {
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

pub fn keycode_for_character(character: char, layout: &str) -> Option<u16> {
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

pub fn replacement_keycodes(text: &str, target_layout: &str) -> Vec<u16> {
    text.chars()
        .filter_map(|character| keycode_for_character(character, target_layout))
        .collect()
}

pub fn correct_text_for_layout(text: &str, source_layout: &str, target_layout: &str) -> String {
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

pub fn infer_layout_from_text(text: &str) -> Option<&'static str> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_correction_is_unlimited_for_every_supported_language() {
        assert_eq!(target_layout(Language::English), Some("us"));
        assert_eq!(target_layout(Language::Russian), Some("ru"));
        assert_eq!(target_layout(Language::Ukrainian), Some("ua"));
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
        let decision = evaluate_layout_candidates("ghbdtn", "us", &classifier, 0.75);
        assert!(decision.switch);
        assert_eq!(decision.language, Language::Russian);
        assert_eq!(decision.target_layout, Some("ru"));
        assert_eq!(decision.corrected, "привет");
    }

    #[test]
    fn uppercase_short_input_can_trigger_a_russian_layout_switch() {
        let classifier = LanguageClassifier::new();
        let decision = evaluate_layout_candidates("ALLO", "us", &classifier, 0.65);
        assert!(decision.switch);
        assert_eq!(decision.target_layout, Some("ru"));
        assert_eq!(decision.corrected, "ФДДЩ");
    }

    #[test]
    fn dual_candidate_evaluation_switches_russian_gibberish_to_english() {
        let classifier = LanguageClassifier::new();
        let decision = evaluate_layout_candidates("руддщ", "ru", &classifier, 0.75);
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
            let decision = evaluate_layout_candidates(source, "us", &classifier, 0.65);
            assert!(decision.switch, "expected switch for {source}");
            assert_eq!(decision.corrected, expected);
            assert_eq!(decision.target_layout, Some("ru"));
        }

        let reverse = evaluate_layout_candidates("руддщ цщкдв", "ru", &classifier, 0.65);
        assert!(reverse.switch);
        assert_eq!(reverse.language, Language::English);
        assert_eq!(reverse.target_layout, Some("us"));
        assert_eq!(reverse.corrected, "hello world");
    }

    // Hindi is detected everywhere, but auto-correction across a physical keymap
    // (like en<->ru) is not implemented yet: there is no verified INSCRIPT keycode
    // table, so `target_layout` intentionally returns `None` for `Language::Hindi`.
    // Devanagari text typed directly is recognized and left untouched, regardless
    // of which layout the daemon currently thinks is active.
    #[test]
    fn hindi_text_is_detected_but_not_auto_corrected_regardless_of_current_layout() {
        let classifier = LanguageClassifier::new();
        for current_layout in ["us", "ru"] {
            let decision =
                evaluate_layout_candidates("नमस्ते दुनिया", current_layout, &classifier, 0.60);
            assert!(!decision.switch, "layout={current_layout}");
            assert_eq!(
                decision.language,
                Language::Hindi,
                "layout={current_layout}"
            );
            assert_eq!(decision.target_layout, None, "layout={current_layout}");
            assert_eq!(decision.corrected, "नमस्ते दुनिया", "layout={current_layout}");
        }
    }
}
