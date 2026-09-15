use std::collections::HashMap;

pub mod layout;
pub mod prompt_detector;
pub mod prompt_improver;

pub const RING_BUFFER_CAPACITY: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Script {
    Latin,
    Cyrillic,
    Devanagari,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Language {
    English,
    Spanish,
    German,
    French,
    Russian,
    Ukrainian,
    Hindi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    ch: char,
}

impl Token {
    pub fn new(ch: char) -> Self {
        Self { ch }
    }

    pub fn ch(&self) -> char {
        self.ch
    }
}

#[derive(Clone, Debug)]
pub struct RingBuffer<const N: usize> {
    data: [Option<char>; N],
    len: usize,
    head: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub fn new() -> Self {
        Self {
            data: [None; N],
            len: 0,
            head: 0,
        }
    }

    pub fn push(&mut self, token: char) {
        self.data[self.head] = Some(token);
        self.head = (self.head + 1) % N;
        if self.len < N {
            self.len += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_string(&self) -> String {
        let mut out = String::with_capacity(self.len);
        for offset in 0..self.len {
            let idx = (self.head + N - self.len + offset) % N;
            if let Some(ch) = self.data[idx] {
                out.push(ch);
            }
        }
        out
    }
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
struct TrieNode {
    children: HashMap<char, Box<TrieNode>>,
    terminal: bool,
}

#[derive(Clone, Debug)]
pub struct HardFilterTrie {
    root: TrieNode,
}

impl HardFilterTrie {
    pub fn new(illegal_sequences: &[&str]) -> Self {
        let root = TrieNode {
            children: HashMap::new(),
            terminal: false,
        };

        let mut trie = Self { root };
        for pattern in illegal_sequences {
            trie.insert(pattern);
        }
        trie
    }

    fn insert(&mut self, pattern: &str) {
        let mut node = &mut self.root;
        for ch in pattern.chars() {
            node = node.children.entry(ch).or_insert_with(|| {
                Box::new(TrieNode {
                    children: HashMap::new(),
                    terminal: false,
                })
            });
        }
        node.terminal = true;
    }

    pub fn rejects(&self, text: &str) -> bool {
        let lowered = text.to_lowercase();
        for start in 0..lowered.chars().count() {
            let mut node = &self.root;
            for ch in lowered.chars().skip(start) {
                let Some(next) = node.children.get(&ch) else {
                    break;
                };
                node = next;
                if node.terminal {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct LanguageProfile {
    pub language: Language,
    pub script: Script,
    char_weights: HashMap<char, u32>,
    _char_total: u32,
    bigrams: HashMap<String, u32>,
    trigrams: HashMap<String, u32>,
    filter: HardFilterTrie,
}

impl LanguageProfile {
    pub fn score(&self, text: &str) -> f64 {
        let normalized = normalize_for_script(text, self.script);
        if normalized.is_empty() {
            return f64::NEG_INFINITY;
        }

        if self.filter.rejects(&normalized) {
            return f64::NEG_INFINITY;
        }

        let char_score = self.character_score(&normalized);
        let bigram_score = laplace_score(&normalized, &self.bigrams, self.bigrams.len(), 2);
        let trigram_score = laplace_score(&normalized, &self.trigrams, self.trigrams.len(), 3);

        char_score + (bigram_score * 3.0) + (trigram_score * 5.0)
    }

    fn character_score(&self, text: &str) -> f64 {
        let mut score = 0.0;
        let mut count = 0usize;

        for ch in text.chars() {
            let weight = self.char_weights.get(&ch).copied().unwrap_or(1);
            score += weight as f64;
            count += 1;
        }

        if count == 0 {
            0.0
        } else {
            score / count as f64
        }
    }
}

#[derive(Clone, Debug)]
pub struct LanguageClassifier {
    profiles: Vec<LanguageProfile>,
}

impl LanguageClassifier {
    pub fn new() -> Self {
        Self {
            profiles: vec![
                build_english_profile(),
                build_spanish_profile(),
                build_german_profile(),
                build_french_profile(),
                build_russian_profile(),
                build_ukrainian_profile(),
                build_hindi_profile(),
            ],
        }
    }

    pub fn classify(&self, text: &str) -> Language {
        let lower = text.to_lowercase();

        if lower.chars().any(is_devanagari_char) {
            return Language::Hindi;
        }

        let has_cyrillic = lower.chars().any(is_cyrillic_char);

        if has_cyrillic {
            let ukrainian_markers = contains_ukrainian_markers(&lower);
            let russian_markers = contains_russian_markers(&lower);

            if ukrainian_markers && !russian_markers {
                return Language::Ukrainian;
            }

            if ukrainian_markers && russian_markers {
                return Language::Ukrainian;
            }

            return Language::Russian;
        }

        let has_german_markers = contains_german_markers(&lower);
        let has_english_markers = contains_english_markers(&lower);

        if has_german_markers && !has_english_markers {
            return Language::German;
        }

        if has_english_markers || !has_german_markers {
            return Language::English;
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut best_lang = Language::English;

        for profile in &self.profiles {
            if profile.script != Script::Latin {
                continue;
            }
            let score = profile.score(text);
            if score > best_score {
                best_score = score;
                best_lang = profile.language;
            }
        }

        best_lang
    }

    pub fn classify_buffer<const N: usize>(&self, buffer: &RingBuffer<N>) -> Language {
        self.classify(&buffer.as_string())
    }

    pub fn classify_with_confidence(&self, text: &str) -> (Language, f64) {
        let language = self.classify(text);
        let normalized = text
            .chars()
            .filter(|character| !character.is_whitespace())
            .count();
        if normalized == 0 {
            return (language, 0.0);
        }

        let length_factor = (normalized as f64 / 16.0).min(1.0);
        let confidence = match language {
            Language::Russian
                if text
                    .chars()
                    .any(|character| ['ы', 'э', 'ъ'].contains(&character))
                    || contains_russian_words(&text.to_lowercase())
                    || russian_ngram_coverage(&text.to_lowercase()) >= 0.45 =>
            {
                0.98
            }
            Language::Russian => 0.70 + (0.15 * length_factor),
            Language::English if contains_english_markers(&text.to_lowercase()) => 0.94,
            Language::German if contains_german_markers(&text.to_lowercase()) => 0.94,
            Language::Hindi => 0.90 + (0.08 * length_factor),
            _ => 0.55 + (0.25 * length_factor),
        };

        (language, confidence)
    }
}

impl Default for LanguageClassifier {
    fn default() -> Self {
        Self::new()
    }
}

fn is_cyrillic_char(ch: char) -> bool {
    let code = ch as u32;
    (0x430..=0x44f).contains(&code) || code == 0x451 || ch == 'ё'
}

fn is_devanagari_char(ch: char) -> bool {
    (0x0900..=0x097f).contains(&(ch as u32))
}

fn contains_ukrainian_markers(text: &str) -> bool {
    let markers = ["і", "ї", "є", "ґ"];
    markers.iter().any(|marker| text.contains(marker))
}

fn contains_russian_markers(text: &str) -> bool {
    let markers = ["ы", "э", "ъ", "ч", "ш", "щ", "ж", "ц"];
    markers.iter().any(|marker| text.contains(marker))
}

fn contains_russian_words(text: &str) -> bool {
    [
        "привет",
        "этот",
        "быстр",
        "тест",
        "провер",
        "расклад",
        "алло",
        "артем",
        "один",
        "кто",
        "что",
    ]
    .iter()
    .any(|marker| text.contains(marker))
}

fn russian_ngram_coverage(text: &str) -> f64 {
    let valid = [
        "пр", "ро", "ог", "ра", "мм", "ми", "ир", "ов", "ва", "ан", "ни", "ие", "ал", "лл", "ло",
        "ар", "рт", "те", "ем", "од", "ди", "ин", "кт", "то", "чт",
    ];
    let mut total = 0usize;
    let mut matches = 0usize;

    for word in text.split_whitespace() {
        let characters: Vec<char> = word.chars().collect();
        for pair in characters.windows(2) {
            total += 1;
            if valid
                .iter()
                .any(|ngram| pair.iter().copied().eq(ngram.chars()))
            {
                matches += 1;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        matches as f64 / total as f64
    }
}

fn contains_english_markers(text: &str) -> bool {
    let markers = [
        "the", "ing", "ion", "th", "he", "and", "with", "this", "that", "tion", "was",
    ];
    markers.iter().any(|marker| text.contains(marker))
}

fn contains_german_markers(text: &str) -> bool {
    let markers = [
        "sch", "ich", "ein", "und", "der", "den", "mit", "auf", "cht",
    ];
    markers.iter().any(|marker| text.contains(marker))
}

fn normalize_for_script(text: &str, script: Script) -> String {
    let mut out = String::new();
    for ch in text.to_lowercase().chars() {
        match script {
            Script::Latin => {
                if ch.is_ascii_alphabetic() {
                    out.push(ch);
                } else if ch.is_ascii_whitespace() {
                    out.push(' ');
                }
            }
            Script::Cyrillic => {
                if is_cyrillic_char(ch) {
                    out.push(ch);
                } else if ch.is_ascii_whitespace() {
                    out.push(' ');
                }
            }
            Script::Devanagari => {
                if is_devanagari_char(ch) {
                    out.push(ch);
                } else if ch.is_ascii_whitespace() {
                    out.push(' ');
                }
            }
        }
    }
    out
}

fn sliding_ngrams(text: &str, size: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < size {
        return Vec::new();
    }

    let mut grams = Vec::new();
    for window in chars.windows(size) {
        let mut gram = String::new();
        for ch in window {
            gram.push(*ch);
        }
        grams.push(gram);
    }
    grams
}

fn laplace_score(
    text: &str,
    grams: &HashMap<String, u32>,
    vocabulary_size: usize,
    order: usize,
) -> f64 {
    let observed = sliding_ngrams(text, order);
    if observed.is_empty() {
        return 0.0;
    }

    let total_count = grams.values().sum::<u32>() as f64;
    let vocab = vocabulary_size.max(1) as f64;
    let mut score = 0.0;

    for gram in &observed {
        let count = grams.get(gram).copied().unwrap_or(0) as f64;
        let numerator = count + 1.0;
        let denominator = total_count + vocab;
        score += (numerator / denominator).ln() * 4.0;
    }

    score / observed.len() as f64
}

fn map_from_pairs(pairs: &[(&str, u32)]) -> HashMap<String, u32> {
    let mut map = HashMap::new();
    for (key, value) in pairs {
        map.insert((*key).to_string(), *value);
    }
    map
}

fn build_english_profile() -> LanguageProfile {
    let char_weights = HashMap::from([
        ('a', 118),
        ('b', 20),
        ('c', 28),
        ('d', 50),
        ('e', 172),
        ('f', 22),
        ('g', 20),
        ('h', 88),
        ('i', 94),
        ('j', 2),
        ('k', 6),
        ('l', 52),
        ('m', 26),
        ('n', 88),
        ('o', 96),
        ('p', 24),
        ('q', 2),
        ('r', 82),
        ('s', 86),
        ('t', 128),
        ('u', 38),
        ('v', 12),
        ('w', 32),
        ('x', 2),
        ('y', 22),
        ('z', 2),
    ]);
    let _char_total = char_weights.values().sum();
    let bigrams = map_from_pairs(&[
        ("th", 220000),
        ("he", 205000),
        ("in", 190000),
        ("er", 182000),
        ("an", 175000),
        ("ed", 168000),
        ("on", 162000),
        ("at", 158000),
        ("is", 152000),
        ("to", 148000),
        ("it", 144000),
        ("io", 135000),
        ("en", 130000),
        ("ou", 128000),
        ("re", 124000),
        ("ea", 120000),
        ("nd", 118000),
        ("ti", 116000),
        ("es", 112000),
        ("or", 108000),
        ("te", 104000),
        ("of", 101000),
        ("al", 98000),
    ]);
    let trigrams = map_from_pairs(&[
        ("the", 520000),
        ("and", 440000),
        ("ing", 360000),
        ("her", 240000),
        ("ere", 190000),
        ("ent", 170000),
        ("tha", 160000),
        ("was", 150000),
        ("for", 142000),
        ("you", 136000),
        ("not", 128000),
        ("eth", 120000),
        ("ion", 118000),
        ("with", 112000),
        ("this", 108000),
        ("that", 105000),
    ]);

    LanguageProfile {
        language: Language::English,
        script: Script::Latin,
        char_weights,
        _char_total,
        bigrams,
        trigrams,
        filter: HardFilterTrie::new(&["яы", "фф", "йй", "ыы", "жж"]),
    }
}

fn build_spanish_profile() -> LanguageProfile {
    let char_weights = HashMap::from([
        ('a', 115),
        ('b', 14),
        ('c', 25),
        ('d', 46),
        ('e', 140),
        ('f', 9),
        ('g', 17),
        ('h', 8),
        ('i', 59),
        ('j', 1),
        ('k', 0),
        ('l', 48),
        ('m', 31),
        ('n', 65),
        ('o', 85),
        ('p', 26),
        ('q', 4),
        ('r', 66),
        ('s', 72),
        ('t', 45),
        ('u', 40),
        ('v', 9),
        ('w', 0),
        ('x', 2),
        ('y', 1),
        ('z', 4),
    ]);
    let _char_total = char_weights.values().sum();
    let bigrams = map_from_pairs(&[
        ("es", 270),
        ("en", 240),
        ("la", 260),
        ("os", 160),
        ("ar", 150),
        ("de", 220),
        ("el", 210),
        ("er", 130),
        ("ra", 140),
        ("re", 120),
    ]);
    let trigrams = map_from_pairs(&[
        ("que", 430),
        ("est", 250),
        ("los", 200),
        ("por", 220),
        ("con", 190),
        ("del", 170),
    ]);
    LanguageProfile {
        language: Language::Spanish,
        script: Script::Latin,
        _char_total,
        char_weights,
        bigrams,
        trigrams,
        filter: HardFilterTrie::new(&["яы", "ааа", "щщ", "жж"]),
    }
}

fn build_german_profile() -> LanguageProfile {
    let char_weights = HashMap::from([
        ('a', 65),
        ('b', 19),
        ('c', 27),
        ('d', 50),
        ('e', 170),
        ('f', 27),
        ('g', 35),
        ('h', 49),
        ('i', 76),
        ('j', 2),
        ('k', 1),
        ('l', 34),
        ('m', 26),
        ('n', 96),
        ('o', 25),
        ('p', 7),
        ('q', 1),
        ('r', 70),
        ('s', 61),
        ('t', 38),
        ('u', 43),
        ('v', 9),
        ('w', 19),
        ('x', 1),
        ('y', 1),
        ('z', 11),
    ]);
    let bigrams = map_from_pairs(&[
        ("ch", 48),
        ("de", 58),
        ("en", 50),
        ("er", 45),
        ("ei", 43),
        ("in", 41),
        ("te", 39),
    ]);
    let trigrams = map_from_pairs(&[
        ("sch", 17),
        ("nde", 13),
        ("ein", 17),
        ("und", 21),
        ("ung", 12),
        ("gen", 11),
    ]);
    LanguageProfile {
        language: Language::German,
        script: Script::Latin,
        _char_total: char_weights.values().sum(),
        char_weights,
        bigrams,
        trigrams,
        filter: HardFilterTrie::new(&["яя", "щщ", "ыы", "жо"]),
    }
}

fn build_french_profile() -> LanguageProfile {
    let char_weights = HashMap::from([
        ('a', 81),
        ('b', 9),
        ('c', 30),
        ('d', 17),
        ('e', 171),
        ('f', 11),
        ('g', 10),
        ('h', 7),
        ('i', 75),
        ('j', 1),
        ('k', 0),
        ('l', 41),
        ('m', 30),
        ('n', 72),
        ('o', 50),
        ('p', 27),
        ('q', 8),
        ('r', 65),
        ('s', 79),
        ('t', 57),
        ('u', 63),
        ('v', 18),
        ('w', 1),
        ('x', 4),
        ('y', 2),
        ('z', 1),
    ]);
    let bigrams = map_from_pairs(&[
        ("en", 52),
        ("re", 46),
        ("es", 40),
        ("de", 44),
        ("ou", 38),
        ("an", 36),
        ("le", 42),
    ]);
    let trigrams = map_from_pairs(&[
        ("ent", 23),
        ("ion", 21),
        ("que", 18),
        ("des", 17),
        ("par", 13),
        ("les", 15),
    ]);
    LanguageProfile {
        language: Language::French,
        script: Script::Latin,
        _char_total: char_weights.values().sum(),
        char_weights,
        bigrams,
        trigrams,
        filter: HardFilterTrie::new(&["яя", "фф", "ыы", "шш"]),
    }
}

fn build_russian_profile() -> LanguageProfile {
    let char_weights = HashMap::from([
        ('а', 108),
        ('б', 18),
        ('в', 46),
        ('г', 16),
        ('д', 25),
        ('е', 81),
        ('ё', 7),
        ('ж', 16),
        ('з', 17),
        ('и', 54),
        ('й', 10),
        ('к', 35),
        ('л', 43),
        ('м', 32),
        ('н', 67),
        ('о', 97),
        ('п', 28),
        ('р', 42),
        ('с', 45),
        ('т', 53),
        ('у', 28),
        ('ф', 2),
        ('х', 9),
        ('ц', 4),
        ('ч', 16),
        ('ш', 6),
        ('щ', 3),
        ('ъ', 1),
        ('ы', 19),
        ('ь', 17),
        ('э', 3),
        ('ю', 6),
        ('я', 18),
    ]);
    let bigrams = map_from_pairs(&[
        ("ст", 35),
        ("но", 33),
        ("то", 28),
        ("на", 26),
        ("ов", 19),
        ("ен", 22),
        ("ни", 21),
        ("ра", 20),
    ]);
    let trigrams = map_from_pairs(&[
        ("сто", 12),
        ("что", 9),
        ("при", 10),
        ("ого", 8),
        ("ение", 8),
        ("нов", 7),
    ]);
    LanguageProfile {
        language: Language::Russian,
        script: Script::Cyrillic,
        _char_total: char_weights.values().sum(),
        char_weights,
        bigrams,
        trigrams,
        filter: HardFilterTrie::new(&["th", "ing", "tion", "qu", "ght"]),
    }
}

fn build_ukrainian_profile() -> LanguageProfile {
    let char_weights = HashMap::from([
        ('а', 89),
        ('б', 17),
        ('в', 46),
        ('г', 15),
        ('д', 23),
        ('е', 80),
        ('є', 8),
        ('ж', 14),
        ('з', 14),
        ('и', 57),
        ('і', 28),
        ('ї', 4),
        ('й', 10),
        ('к', 27),
        ('л', 42),
        ('м', 31),
        ('н', 62),
        ('о', 100),
        ('п', 31),
        ('р', 40),
        ('с', 45),
        ('т', 56),
        ('у', 34),
        ('ф', 2),
        ('х', 8),
        ('ц', 5),
        ('ч', 15),
        ('ш', 8),
        ('щ', 3),
        ('ь', 13),
        ('ю', 7),
        ('я', 25),
    ]);
    let bigrams = map_from_pairs(&[
        ("ст", 31),
        ("ни", 24),
        ("на", 22),
        ("ро", 20),
        ("ов", 18),
        ("ти", 18),
    ]);
    let trigrams = map_from_pairs(&[("сто", 11), ("при", 9), ("ова", 8), ("ний", 9), ("все", 7)]);
    LanguageProfile {
        language: Language::Ukrainian,
        script: Script::Cyrillic,
        _char_total: char_weights.values().sum(),
        char_weights,
        bigrams,
        trigrams,
        filter: HardFilterTrie::new(&["the", "and", "tion", "ing", "qu"]),
    }
}

fn build_hindi_profile() -> LanguageProfile {
    let char_weights = HashMap::from([
        ('अ', 20),
        ('आ', 25),
        ('इ', 15),
        ('ई', 8),
        ('उ', 12),
        ('ऊ', 4),
        ('ए', 18),
        ('ओ', 14),
        ('क', 55),
        ('ख', 10),
        ('ग', 22),
        ('घ', 4),
        ('च', 18),
        ('छ', 5),
        ('ज', 20),
        ('झ', 4),
        ('ट', 12),
        ('ठ', 4),
        ('ड', 10),
        ('ढ', 3),
        ('ण', 8),
        ('त', 60),
        ('थ', 12),
        ('द', 45),
        ('ध', 10),
        ('न', 65),
        ('प', 40),
        ('फ', 6),
        ('ब', 30),
        ('भ', 10),
        ('म', 48),
        ('य', 30),
        ('र', 55),
        ('ल', 35),
        ('व', 32),
        ('श', 14),
        ('ष', 6),
        ('स', 45),
        ('ह', 42),
        ('ा', 70),
        ('ि', 55),
        ('ी', 45),
        ('ु', 28),
        ('ू', 10),
        ('े', 40),
        ('ै', 15),
        ('ो', 22),
        ('ौ', 6),
        ('ं', 20),
        ('ः', 3),
        ('ँ', 3),
        ('्', 38),
    ]);
    let bigrams = map_from_pairs(&[
        ("है", 40),
        ("का", 38),
        ("की", 30),
        ("के", 32),
        ("में", 34),
        ("से", 22),
        ("को", 26),
        ("ने", 20),
        ("और", 18),
        ("पर", 14),
    ]);
    let trigrams = map_from_pairs(&[
        ("नही", 12),
        ("कार", 10),
        ("रहा", 9),
        ("वाला", 8),
        ("लिए", 11),
    ]);
    LanguageProfile {
        language: Language::Hindi,
        script: Script::Devanagari,
        _char_total: char_weights.values().sum(),
        char_weights,
        bigrams,
        trigrams,
        filter: HardFilterTrie::new(&["the", "and", "tion", "ing", "qu"]),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn english_phrase_is_classified_correctly() {
        let classifier = LanguageClassifier::new();
        let text = "the quick brown fox jumps over the lazy dog";
        assert_eq!(classifier.classify(text), Language::English);
    }

    #[test]
    fn russian_phrase_is_classified_correctly() {
        let classifier = LanguageClassifier::new();
        let text = "этот быстрый тест проверяет переключение раскладки";
        assert_eq!(classifier.classify(text), Language::Russian);
    }

    #[test]
    fn ring_buffer_keeps_32_latest_tokens() {
        let mut buffer = RingBuffer::<32>::new();
        for ch in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
            buffer.push(ch);
        }

        assert_eq!(buffer.len(), 32);
        let text = buffer.as_string();
        assert_eq!(text.len(), 32);
        assert!(text.starts_with('f') || text.starts_with('e'));
    }

    #[test]
    fn classification_runs_in_under_one_millisecond() {
        let classifier = LanguageClassifier::new();
        let text = "the quick brown fox jumps over the lazy dog while reading a technical document";
        let mut buffer = RingBuffer::<32>::new();
        for ch in text.chars() {
            buffer.push(ch);
        }

        for _ in 0..10 {
            let _ = classifier.classify_buffer(&buffer);
        }

        let start = Instant::now();
        for _ in 0..100 {
            let _ = classifier.classify_buffer(&buffer);
        }
        let elapsed = start.elapsed();
        let avg_time = elapsed / 100;

        assert!(
            avg_time < Duration::from_millis(1),
            "avg classification time was {:?}",
            avg_time
        );
    }

    #[test]
    fn ring_buffer_starts_empty() {
        let buffer = RingBuffer::<8>::new();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.as_string(), "");
    }

    #[test]
    fn ring_buffer_preserves_order_before_wraparound() {
        let mut buffer = RingBuffer::<5>::new();
        for ch in "abc".chars() {
            buffer.push(ch);
        }
        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.as_string(), "abc");
    }

    #[test]
    fn ring_buffer_wraps_and_keeps_only_the_latest_n_in_order() {
        let mut buffer = RingBuffer::<4>::new();
        for ch in "abcdef".chars() {
            buffer.push(ch);
        }
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.as_string(), "cdef");
    }

    #[test]
    fn ring_buffer_default_matches_new() {
        let buffer: RingBuffer<8> = RingBuffer::default();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.as_string(), "");
    }

    #[test]
    fn hard_filter_trie_rejects_only_configured_substrings() {
        let trie = HardFilterTrie::new(&["xx", "qzq"]);
        assert!(trie.rejects("aaxxbb"));
        assert!(trie.rejects("qzq"));
        assert!(!trie.rejects("abcdef"));
        assert!(!trie.rejects(""));
    }

    #[test]
    fn hard_filter_trie_with_no_patterns_rejects_nothing() {
        let trie = HardFilterTrie::new(&[]);
        assert!(!trie.rejects("anything at all"));
        assert!(!trie.rejects(""));
    }

    #[test]
    fn profile_score_of_empty_text_is_negative_infinity() {
        let profile = build_english_profile();
        assert_eq!(profile.score(""), f64::NEG_INFINITY);
        // Digits are neither alphabetic nor whitespace, so Latin normalization drops them all.
        assert_eq!(profile.score("12345"), f64::NEG_INFINITY);
    }

    #[test]
    fn empty_input_classifies_as_english_without_panicking() {
        let classifier = LanguageClassifier::new();
        assert_eq!(classifier.classify(""), Language::English);

        let buffer = RingBuffer::<32>::new();
        assert_eq!(classifier.classify_buffer(&buffer), Language::English);
    }

    #[test]
    fn whitespace_only_input_has_zero_confidence() {
        let classifier = LanguageClassifier::new();
        let (_, confidence) = classifier.classify_with_confidence("   ");
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn german_phrase_is_classified_correctly() {
        let classifier = LanguageClassifier::new();
        let text = "ich bin mit dem auf und ein sehr guter mensch";
        assert_eq!(classifier.classify(text), Language::German);
    }

    #[test]
    fn ukrainian_marker_selects_ukrainian_over_russian() {
        let classifier = LanguageClassifier::new();
        let text = "привіт як справи";
        assert_eq!(classifier.classify(text), Language::Ukrainian);
    }

    #[test]
    fn mixed_ukrainian_and_russian_markers_prefer_ukrainian() {
        let classifier = LanguageClassifier::new();
        let text = "їжа і чай ъ";
        assert_eq!(classifier.classify(text), Language::Ukrainian);
    }

    #[test]
    fn hindi_phrase_is_classified_correctly() {
        let classifier = LanguageClassifier::new();
        let text = "नमस्ते दुनिया, यह एक परीक्षण है";
        assert_eq!(classifier.classify(text), Language::Hindi);
    }

    #[test]
    fn hindi_takes_priority_over_latin_and_cyrillic_scripts_when_mixed() {
        let classifier = LanguageClassifier::new();
        let text = "hello नमस्ते привет";
        assert_eq!(classifier.classify(text), Language::Hindi);
    }

    #[test]
    fn hindi_classification_has_high_confidence() {
        let classifier = LanguageClassifier::new();
        let (language, confidence) = classifier.classify_with_confidence("नमस्ते");
        assert_eq!(language, Language::Hindi);
        assert!(confidence >= 0.90, "confidence was {confidence}");
    }

    #[test]
    fn devanagari_normalization_keeps_only_devanagari_and_whitespace() {
        let normalized = normalize_for_script("नमस्ते hello 123", Script::Devanagari);
        assert_eq!(normalized, "नमस्ते  ");
    }

    #[test]
    fn ring_buffer_classifies_hindi_text_end_to_end() {
        let classifier = LanguageClassifier::new();
        let mut buffer = RingBuffer::<32>::new();
        for ch in "नमस्ते दुनिया".chars() {
            buffer.push(ch);
        }
        assert_eq!(classifier.classify_buffer(&buffer), Language::Hindi);
    }
}
