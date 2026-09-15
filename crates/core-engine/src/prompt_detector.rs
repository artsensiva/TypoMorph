use crate::{Language, LanguageClassifier};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PromptSignal {
    pub is_prompt: bool,
    pub confidence: f64,
    pub language: Language,
}

pub struct PromptDetector {
    classifier: LanguageClassifier,
}

impl PromptDetector {
    pub fn new() -> Self {
        Self {
            classifier: LanguageClassifier::new(),
        }
    }

    pub fn detect(&self, text: &str) -> PromptSignal {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return PromptSignal {
                is_prompt: false,
                confidence: 0.0,
                language: Language::English,
            };
        }

        let (language, _) = self.classifier.classify_with_confidence(trimmed);
        let lower = trimmed.to_lowercase();
        let word_count = lower.split_whitespace().count();

        let mut score: f64 = 0.0;
        if word_count >= 4 {
            score += 0.15;
        }
        if word_count >= 8 {
            score += 0.10;
        }
        if trimmed.ends_with('?') {
            score += 0.15;
        }
        if contains_instruction_marker(&lower, language) {
            score += 0.45;
        }
        if contains_roleplay_marker(&lower, language) {
            score += 0.25;
        }
        if contains_politeness_marker(&lower, language) {
            score += 0.10;
        }

        let confidence = score.min(1.0);
        PromptSignal {
            is_prompt: confidence >= 0.5,
            confidence,
            language,
        }
    }
}

impl Default for PromptDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Coverage is deliberately limited to EN/RU/HI (the languages this heuristic is
// tested against); other Latin languages fall back to the English marker list,
// and Ukrainian shares the Russian list as a rough proxy.
fn contains_instruction_marker(text: &str, language: Language) -> bool {
    let markers: &[&str] = match language {
        Language::Russian | Language::Ukrainian => &[
            "напиши",
            "объясни",
            "сгенерируй",
            "переведи",
            "составь",
            "придумай",
            "исправь",
            "перепиши",
            "опиши",
            "сравни",
            "расскажи",
            "помоги",
            "как сделать",
            "дай мне",
            "создай",
        ],
        Language::Hindi => &[
            "लिखो",
            "लिखिए",
            "समझाओ",
            "समझाइए",
            "बनाओ",
            "बनाइए",
            "अनुवाद करो",
            "बताओ",
            "बताइए",
            "मदद करो",
            "कैसे करें",
            "सूची बनाओ",
            "सुधारो",
        ],
        _ => &[
            "write",
            "explain",
            "generate",
            "summarize",
            "translate",
            "create",
            "list",
            "analyze",
            "fix",
            "refactor",
            "debug",
            "describe",
            "compare",
            "how do i",
            "how to",
            "give me",
            "help me",
            "act as",
        ],
    };
    markers.iter().any(|marker| text.contains(marker))
}

fn contains_roleplay_marker(text: &str, language: Language) -> bool {
    let markers: &[&str] = match language {
        Language::Russian | Language::Ukrainian => &["ты — ", "представь что ты", "веди себя как"],
        Language::Hindi => &["तुम हो", "एक की तरह व्यवहार करो"],
        _ => &["act as", "you are a", "pretend to be", "behave like"],
    };
    markers.iter().any(|marker| text.contains(marker))
}

fn contains_politeness_marker(text: &str, language: Language) -> bool {
    let markers: &[&str] = match language {
        Language::Russian | Language::Ukrainian => &["пожалуйста"],
        Language::Hindi => &["कृपया"],
        _ => &["please"],
    };
    markers.iter().any(|marker| text.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_english_prompt() {
        let detector = PromptDetector::new();
        let signal = detector.detect("Please write a short story about a dragon and a knight");
        assert!(signal.is_prompt, "confidence was {}", signal.confidence);
        assert_eq!(signal.language, Language::English);
    }

    #[test]
    fn detects_russian_prompt() {
        let detector = PromptDetector::new();
        let signal = detector.detect("Пожалуйста, напиши короткий рассказ про дракона");
        assert!(signal.is_prompt, "confidence was {}", signal.confidence);
        assert_eq!(signal.language, Language::Russian);
    }

    #[test]
    fn detects_hindi_prompt() {
        let detector = PromptDetector::new();
        let signal = detector.detect("कृपया एक छोटी कहानी लिखो");
        assert!(signal.is_prompt, "confidence was {}", signal.confidence);
        assert_eq!(signal.language, Language::Hindi);
    }

    #[test]
    fn detects_question_style_prompt_without_explicit_verb() {
        let detector = PromptDetector::new();
        let signal = detector.detect("how do I fix this bug in my rust program?");
        assert!(signal.is_prompt, "confidence was {}", signal.confidence);
    }

    #[test]
    fn casual_short_chat_is_not_a_prompt() {
        let detector = PromptDetector::new();
        for text in ["hey", "ok thanks", "lol", "привет"] {
            let signal = detector.detect(text);
            assert!(!signal.is_prompt, "{text:?} scored {}", signal.confidence);
        }
    }

    #[test]
    fn empty_input_is_not_a_prompt() {
        let detector = PromptDetector::new();
        let signal = detector.detect("   ");
        assert!(!signal.is_prompt);
        assert_eq!(signal.confidence, 0.0);
    }
}
