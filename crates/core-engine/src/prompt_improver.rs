/// Deterministic, offline, free-forever prompt cleanup. No language understanding,
/// no generation — just mechanical normalization. The paid cloud path (built on
/// top of this crate, not in it) is what does actual AI-assisted rewriting.
pub fn improve_rule_based(text: &str) -> String {
    let collapsed = collapse_whitespace(text);
    let deduped = collapse_repeated_punctuation(&collapsed);
    let expanded = expand_abbreviations(&deduped);
    ensure_terminal_punctuation(&expanded)
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collapse_repeated_punctuation(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut previous: Option<char> = None;
    for ch in text.chars() {
        if matches!(ch, '?' | '!') && previous == Some(ch) {
            continue;
        }
        out.push(ch);
        previous = Some(ch);
    }
    out
}

fn expand_abbreviations(text: &str) -> String {
    text.split(' ')
        .map(expand_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn expand_word(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    let mut trailing = String::new();
    while let Some(&last) = chars.last() {
        if matches!(last, '.' | ',' | '!' | '?' | ';' | ':') {
            trailing.insert(0, last);
            chars.pop();
        } else {
            break;
        }
    }
    let core: String = chars.into_iter().collect();
    match abbreviation(&core.to_lowercase()) {
        Some(replacement) => format!("{replacement}{trailing}"),
        None => word.to_string(),
    }
}

fn abbreviation(word: &str) -> Option<&'static str> {
    match word {
        "pls" | "plz" => Some("please"),
        "u" => Some("you"),
        "ur" => Some("your"),
        "asap" => Some("as soon as possible"),
        "thx" => Some("thanks"),
        "btw" => Some("by the way"),
        "плз" => Some("пожалуйста"),
        "спс" => Some("спасибо"),
        "щас" => Some("сейчас"),
        _ => None,
    }
}

fn ensure_terminal_punctuation(text: &str) -> String {
    if text.is_empty() || text.ends_with(['.', '!', '?', ':']) {
        text.to_string()
    } else {
        format!("{text}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_abbreviations_and_adds_terminal_punctuation() {
        assert_eq!(
            improve_rule_based("pls   write   me a poem"),
            "please write me a poem."
        );
    }

    #[test]
    fn collapses_repeated_question_marks_without_adding_a_period() {
        assert_eq!(improve_rule_based("what is rust???"), "what is rust?");
    }

    #[test]
    fn expands_russian_abbreviations() {
        assert_eq!(
            improve_rule_based("плз объясни это"),
            "пожалуйста объясни это."
        );
    }

    #[test]
    fn trims_and_collapses_internal_whitespace() {
        assert_eq!(improve_rule_based("  hello   world  "), "hello world.");
    }

    #[test]
    fn leaves_already_well_formed_text_unchanged() {
        assert_eq!(
            improve_rule_based("Explain quantum computing simply."),
            "Explain quantum computing simply."
        );
    }

    #[test]
    fn empty_input_stays_empty() {
        assert_eq!(improve_rule_based("   "), "");
    }
}
