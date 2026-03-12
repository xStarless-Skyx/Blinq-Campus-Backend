use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;

const DEFAULT_WORDS_PATH: &str = "crates/delta/resources/words.json";
const DEFAULT_REPORT_PHRASES_PATH: &str = "crates/delta/resources/report_phrases.json";

#[derive(Deserialize)]
#[serde(transparent)]
struct WordList(Vec<String>);

struct FilterSets {
    single_terms: HashSet<String>,
    phrases: HashSet<String>,
    report_phrases: HashSet<String>,
}

fn normalize_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_space = false;

    for ch in text.chars() {
        if ch.is_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }

    out.trim().to_string()
}

fn load_words() -> Vec<String> {
    let path = std::env::var("BLINQ_BAD_WORDS_PATH").unwrap_or_else(|_| {
        DEFAULT_WORDS_PATH.to_string()
    });

    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(list) = serde_json::from_str::<WordList>(&raw) {
            return list
                .0
                .into_iter()
                .map(|word| word.trim().to_string())
                .filter(|word| !word.is_empty())
                .collect();
        }
    }

    Vec::new()
}

fn load_report_phrases() -> Vec<String> {
    let path = std::env::var("BLINQ_REPORT_PHRASES_PATH").unwrap_or_else(|_| {
        DEFAULT_REPORT_PHRASES_PATH.to_string()
    });

    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(list) = serde_json::from_str::<WordList>(&raw) {
            return list
                .0
                .into_iter()
                .map(|word| word.trim().to_string())
                .filter(|word| !word.is_empty())
                .collect();
        }
    }

    Vec::new()
}

static FILTER_SETS: Lazy<FilterSets> = Lazy::new(|| {
    let mut single_terms = HashSet::new();
    let mut phrases = HashSet::new();
    let mut report_phrases = HashSet::new();

    for raw in load_words() {
        let normalized = normalize_text(&raw);
        if normalized.is_empty() {
            continue;
        }

        if normalized.contains(' ') {
            phrases.insert(normalized);
        } else {
            single_terms.insert(normalized);
        }
    }

    for raw in load_report_phrases() {
        let normalized = normalize_text(&raw);
        if normalized.is_empty() {
            continue;
        }

        report_phrases.insert(normalized);
    }

    FilterSets {
        single_terms,
        phrases,
        report_phrases,
    }
});

pub enum LanguageFilterResult {
    Allow,
    Block,
    BlockAndReport { matches: Vec<String> },
}

pub fn classify_language(text: &str) -> LanguageFilterResult {
    let normalized = normalize_text(text);
    if normalized.is_empty() {
        return LanguageFilterResult::Allow;
    }

    let mut blocked = false;

    for token in normalized.split_whitespace() {
        if FILTER_SETS.single_terms.contains(token) {
            blocked = true;
            break;
        }
    }

    let padded = if !FILTER_SETS.phrases.is_empty() || !FILTER_SETS.report_phrases.is_empty() {
        Some(format!(" {} ", normalized))
    } else {
        None
    };

    if !FILTER_SETS.phrases.is_empty() {
        if let Some(padded) = padded.as_ref() {
            for phrase in FILTER_SETS.phrases.iter() {
                let needle = format!(" {} ", phrase);
                if padded.contains(&needle) {
                    blocked = true;
                    break;
                }
            }
        }
    }

    let mut report_matches = Vec::new();
    if !FILTER_SETS.report_phrases.is_empty() {
        if let Some(padded) = padded.as_ref() {
            for phrase in FILTER_SETS.report_phrases.iter() {
                let needle = format!(" {} ", phrase);
                if padded.contains(&needle) {
                    report_matches.push(phrase.clone());
                }
            }
        }
    }

    if !blocked && report_matches.is_empty() {
        return LanguageFilterResult::Allow;
    }

    if !report_matches.is_empty() {
        return LanguageFilterResult::BlockAndReport {
            matches: report_matches,
        };
    }

    LanguageFilterResult::Block
}

pub fn contains_blocked_language(text: &str) -> bool {
    !matches!(classify_language(text), LanguageFilterResult::Allow)
}
