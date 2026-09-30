//! Replacement planning for the prototype's plain, unmodified US/RU words.
//! The caller must separately establish field/cursor safety and input ordering.
use crate::layout::{keycode_for_character, keycode_to_character};
use crate::RING_BUFFER_CAPACITY;

pub struct WordReplacement {
    backspaces: usize,
    keycodes: Vec<u16>,
}

impl WordReplacement {
    pub fn backspaces(&self) -> usize {
        self.backspaces
    }
    pub fn keycodes(&self) -> &[u16] {
        &self.keycodes
    }
}

/// Plan a complete word followed by a space already delivered to the application.
/// Reject lossy mappings, truncated buffers, unsupported layouts and composition.
pub fn plan_word_replacement(
    source: &str,
    observed_key_count: usize,
    source_layout: &str,
    corrected: &str,
    target_layout: &str,
    boundary: char,
) -> Option<WordReplacement> {
    if boundary != ' '
        || source_layout == target_layout
        || !matches!(source_layout, "us" | "ru")
        || !matches!(target_layout, "us" | "ru")
    {
        return None;
    }
    let length = source.chars().count();
    if length == 0
        || length > RING_BUFFER_CAPACITY
        || length != observed_key_count
        || corrected.chars().count() != length
    {
        return None;
    }
    // Round trips must be exact: the legacy encoder otherwise lowercases or
    // silently drops characters it cannot emit (including unsupported marks).
    for character in source.chars() {
        let key = keycode_for_character(character, source_layout)?;
        if keycode_to_character(key, source_layout)? != character {
            return None;
        }
    }
    let mut keycodes = Vec::with_capacity(length + 1);
    for character in corrected.chars() {
        let key = keycode_for_character(character, target_layout)?;
        if keycode_to_character(key, target_layout)? != character {
            return None;
        }
        keycodes.push(key);
    }
    keycodes.push(57); // Preserve the consumed space; it has no inverse letter mapping.
    Some(WordReplacement {
        backspaces: length + 1,
        keycodes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_to_field(before: &str, after: &str, plan: &WordReplacement, layout: &str) -> String {
        let mut field: Vec<char> = before.chars().collect();
        for _ in 0..plan.backspaces() {
            assert!(field.pop().is_some(), "replacement crossed the field start");
        }
        for &key in plan.keycodes() {
            field.push(keycode_to_character(key, layout).expect("fully mapped plan"));
        }
        field.extend(after.chars());
        field.into_iter().collect()
    }

    #[test]
    fn wrong_layout_word_and_delimiter_are_replaced_without_leftover_letter() {
        let plan = plan_word_replacement("ghbdtn", 6, "us", "привет", "ru", ' ').unwrap();
        assert_eq!(apply_to_field("ghbdtn ", "", &plan, "ru"), "привет ");
        assert_eq!(
            apply_to_field("prefix: ghbdtn ", "suffix", &plan, "ru"),
            "prefix: привет suffix"
        );
    }

    #[test]
    fn reverse_direction_keeps_delimiter_and_neighboring_text() {
        let plan = plan_word_replacement("руддщ", 5, "ru", "hello", "us", ' ').unwrap();
        assert_eq!(
            apply_to_field("до руддщ ", "после", &plan, "us"),
            "до hello после"
        );
    }

    #[test]
    fn all_supported_word_lengths_preserve_surrounding_text() {
        for length in 1..=RING_BUFFER_CAPACITY {
            let source = "g".repeat(length);
            let corrected = "п".repeat(length);
            let plan = plan_word_replacement(&source, length, "us", &corrected, "ru", ' ').unwrap();
            assert_eq!(
                apply_to_field(&format!("prefix {source} "), "tail", &plan, "ru"),
                format!("prefix {corrected} tail")
            );
        }
    }

    #[test]
    fn a_truncated_or_mismatched_word_is_never_partially_replaced() {
        assert!(plan_word_replacement("ghbdtn", 7, "us", "привет", "ru", ' ').is_none());
        assert!(plan_word_replacement("ghbdtn", 5, "us", "привет", "ru", ' ').is_none());
        assert!(
            plan_word_replacement(&"g".repeat(32), 33, "us", &"п".repeat(32), "ru", ' ').is_none()
        );
        assert!(
            plan_word_replacement(&"g".repeat(33), 33, "us", &"п".repeat(33), "ru", ' ').is_none()
        );
        assert!(plan_word_replacement("", 0, "us", "", "ru", ' ').is_none());
    }

    #[test]
    fn unmapped_characters_and_case_are_rejected_instead_of_lost() {
        for (source, corrected) in [
            ("ghbdtn", "привёт"),
            ("GHBDTN", "ПРИВЕТ"),
            ("ghbdtn", "Привет"),
            ("ghbdtn", "привет!"),
            ("ghb!tn", "привет"),
        ] {
            assert!(plan_word_replacement(source, 6, "us", corrected, "ru", ' ').is_none());
        }
    }

    #[test]
    fn unsupported_layouts_and_delimiters_are_rejected() {
        for boundary in ['\n', '\t', '.', '\u{a0}'] {
            assert!(plan_word_replacement("ghbdtn", 6, "us", "привет", "ru", boundary).is_none());
        }
        for (source, target) in [("us", "ua"), ("de", "ru"), ("us", "us")] {
            assert!(plan_word_replacement("ghbdtn", 6, source, "привет", target, ' ').is_none());
        }
    }
}
