//! 越南语明文 TSV 词表与短语表。

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::query::{VietnameseCandidate, VietnameseKind, query};

#[derive(Debug, Clone)]
pub struct VietnameseDictionary {
    words: Vec<Entry>,

    phrases: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub text: String,

    pub telex: String,

    pub compact_telex: String,

    pub frequency: u32,
}

impl VietnameseDictionary {
    pub fn from_paths(words: impl AsRef<Path>, phrases: impl AsRef<Path>) -> std::io::Result<Self> {
        let words = parse_entries(&fs::read_to_string(words)?, false);
        let phrases = parse_entries(&fs::read_to_string(phrases)?, true);
        Ok(Self { words, phrases })
    }

    pub fn from_sources(words: &str, phrases: &str) -> Self {
        Self {
            words: parse_entries(words, false),
            phrases: parse_entries(phrases, true),
        }
    }

    pub fn query(
        &self,
        input: &str,
        choice_weight: impl Fn(&str, &str) -> u32,
    ) -> Vec<VietnameseCandidate> {
        query(self, input, choice_weight)
    }

    pub(crate) fn word_exact(&self, key: &str) -> impl Iterator<Item = &Entry> {
        self.words.iter().filter(move |entry| entry.telex == key)
    }

    pub(crate) fn word_prefix(&self, key: &str) -> impl Iterator<Item = &Entry> {
        self.words
            .iter()
            .filter(move |entry| entry.telex.starts_with(key) && entry.telex != key)
    }

    pub(crate) fn phrases(&self) -> &[Entry] {
        &self.phrases
    }

    pub(crate) fn phrase_prefix(
        &self,
        key: &str,
        compact_key: &str,
    ) -> impl Iterator<Item = &Entry> {
        self.phrases.iter().filter(move |entry| {
            (entry.telex.starts_with(key) || entry.compact_telex.starts_with(compact_key))
                && entry.telex != key
                && entry.compact_telex != compact_key
        })
    }

    pub(crate) fn segment_words(&self, parts: &[&str]) -> Vec<VietnameseCandidate> {
        self.segment_words_inner(parts, false)
    }

    pub(crate) fn segment_words_with_last_prefix(
        &self,
        parts: &[&str],
    ) -> Vec<VietnameseCandidate> {
        self.segment_words_inner(parts, true)
    }

    fn segment_words_inner(&self, parts: &[&str], last_prefix: bool) -> Vec<VietnameseCandidate> {
        let mut variants: Vec<(String, u32)> = vec![(String::new(), 0)];
        for (index, part) in parts.iter().enumerate() {
            let hits: Vec<&Entry> = if last_prefix && index + 1 == parts.len() {
                self.word_prefix(part).collect()
            } else {
                self.word_exact(part).collect()
            };
            if hits.is_empty() {
                return Vec::new();
            }
            let mut next = Vec::new();
            for (prefix, score) in &variants {
                for hit in &hits {
                    let text = if prefix.is_empty() {
                        hit.text.clone()
                    } else {
                        format!("{prefix} {}", hit.text)
                    };
                    next.push((text, score.saturating_add(hit.frequency)));
                }
            }
            variants = next;
        }
        variants
            .into_iter()
            .map(|(text, frequency)| VietnameseCandidate {
                text,
                kind: if parts.len() == 1 {
                    VietnameseKind::Word
                } else {
                    VietnameseKind::Phrase
                },
                score: i64::from(frequency) - i64::from(parts.len().saturating_sub(1) as u32) * 20,
            })
            .collect()
    }
}

fn parse_entries(source: &str, phrase: bool) -> Vec<Entry> {
    let mut best: BTreeMap<(String, String), Entry> = BTreeMap::new();
    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let (Some(text), Some(telex), Some(frequency)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let frequency = frequency.trim().parse::<u32>().unwrap_or(1);
        let telex = normalize_telex(telex, phrase);
        if telex.is_empty() || text.trim().is_empty() {
            continue;
        }
        let entry = Entry {
            text: text.trim().to_owned(),
            compact_telex: compact(&telex),
            telex,
            frequency,
        };
        let key = (entry.text.clone(), entry.telex.clone());
        best.entry(key)
            .and_modify(|old| {
                if entry.frequency > old.frequency {
                    *old = entry.clone();
                }
            })
            .or_insert(entry);
    }
    let mut entries: Vec<Entry> = best.into_values().collect();
    entries.sort_by(|a, b| {
        b.frequency
            .cmp(&a.frequency)
            .then_with(|| a.text.cmp(&b.text))
    });
    entries
}

pub(crate) fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn normalize_telex(text: &str, phrase: bool) -> String {
    let lower = text.trim().to_ascii_lowercase();
    if phrase {
        lower.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        lower
    }
}
