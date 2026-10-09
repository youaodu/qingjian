//! 越南语候选生成与排序。

use std::collections::HashSet;

use super::dictionary::{VietnameseDictionary, compact};
use super::telex::transcribe;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VietnameseKind {
    Word,

    Phrase,

    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VietnameseCandidate {
    pub text: String,

    pub kind: VietnameseKind,

    pub score: i64,
}

const PHRASE_BONUS: i64 = 1_000_000;
const WORD_COMBO_BONUS: i64 = 500_000;
const WORD_COMBO_PREFIX_BONUS: i64 = 350_000;
const PREFIX_WORD_BONUS: i64 = 300_000;
const PREFIX_PHRASE_BONUS: i64 = 200_000;
const TRANSCRIBED_BONUS: i64 = 100_000;
const USER_BONUS: i64 = 20_000;
const RAW_PENALTY: i64 = 1_000_000;

pub(crate) fn query(
    dictionary: &VietnameseDictionary,
    input: &str,
    choice_weight: impl Fn(&str, &str) -> u32,
) -> Vec<VietnameseCandidate> {
    let key = normalize_input(input);
    if key.is_empty() {
        return Vec::new();
    }
    let compact_key = compact(&key);
    let parts: Vec<&str> = key.split_whitespace().collect();
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for phrase in dictionary.phrases() {
        if phrase.telex == key
            || phrase.compact_telex == compact_key
            || phrase.toneless_telex == key
            || phrase.compact_toneless_telex == compact_key
        {
            push_unique(
                &mut out,
                &mut seen,
                VietnameseCandidate {
                    text: phrase.text.clone(),
                    kind: VietnameseKind::Phrase,
                    score: PHRASE_BONUS
                        + i64::from(phrase.frequency)
                        + common_bonus(&phrase.text)
                        + i64::from(choice_weight(&compact_key, &phrase.text)) * USER_BONUS,
                },
            );
        }
    }

    for candidate in dictionary.segment_words(&parts) {
        let score = candidate.score
            + WORD_COMBO_BONUS
            + common_bonus(&candidate.text)
            + i64::from(choice_weight(&compact_key, &candidate.text)) * USER_BONUS;
        push_unique(
            &mut out,
            &mut seen,
            VietnameseCandidate { score, ..candidate },
        );
    }

    if parts.len() > 1 {
        for candidate in dictionary
            .segment_words_with_last_prefix(&parts)
            .into_iter()
            .take(100)
        {
            let score = candidate.score
                + WORD_COMBO_PREFIX_BONUS
                + common_bonus(&candidate.text)
                + i64::from(choice_weight(&compact_key, &candidate.text)) * USER_BONUS;
            push_unique(
                &mut out,
                &mut seen,
                VietnameseCandidate { score, ..candidate },
            );
        }
    }

    for phrase in dictionary.phrase_prefix(&key, &compact_key).take(100) {
        push_unique(
            &mut out,
            &mut seen,
            VietnameseCandidate {
                text: phrase.text.clone(),
                kind: VietnameseKind::Phrase,
                score: PREFIX_PHRASE_BONUS
                    + i64::from(phrase.frequency)
                    + common_bonus(&phrase.text)
                    + i64::from(choice_weight(&compact_key, &phrase.text)) * USER_BONUS,
            },
        );
    }

    if parts.len() == 1 {
        for word in dictionary.word_prefix(&key).take(100) {
            push_unique(
                &mut out,
                &mut seen,
                VietnameseCandidate {
                    text: word.text.clone(),
                    kind: VietnameseKind::Word,
                    score: PREFIX_WORD_BONUS
                        + i64::from(word.frequency)
                        + common_bonus(&word.text)
                        + i64::from(choice_weight(&compact_key, &word.text)) * USER_BONUS,
                },
            );
        }
    }

    let transcribed = transcribe(&key);
    if transcribed != key {
        push_unique(
            &mut out,
            &mut seen,
            VietnameseCandidate {
                text: transcribed,
                kind: if parts.len() == 1 {
                    VietnameseKind::Word
                } else {
                    VietnameseKind::Phrase
                },
                score: TRANSCRIBED_BONUS,
            },
        );
    }

    push_unique(
        &mut out,
        &mut seen,
        VietnameseCandidate {
            text: input.to_owned(),
            kind: VietnameseKind::Raw,
            score: -RAW_PENALTY,
        },
    );
    out.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.text.cmp(&b.text)));
    out
}

fn push_unique(
    out: &mut Vec<VietnameseCandidate>,
    seen: &mut HashSet<String>,
    candidate: VietnameseCandidate,
) {
    if seen.insert(candidate.text.clone()) {
        out.push(candidate);
    }
}

fn common_bonus(text: &str) -> i64 {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut bonus = match text {
        "tôi là" | "tôi muốn" | "tôi có" | "tôi không" | "tôi đi" | "tôi biết" | "tôi nghĩ"
        | "tôi thích" | "tôi cần" | "tôi ở" => 120_000,
        "bạn là" | "bạn muốn" | "bạn có" | "bạn không" | "bạn đi" | "bạn biết" | "bạn nghĩ"
        | "bạn thích" | "bạn cần" | "bạn ở" => 100_000,
        "anh là" | "anh muốn" | "anh có" | "anh không" | "em là" | "em muốn" | "em có"
        | "em không" => 90_000,
        _ => 0,
    };
    if let Some(last) = words.last().copied().or(Some(text)) {
        bonus += match last {
            "là" => 300_000,
            "có" | "không" | "muốn" | "cần" | "được" | "phải" => 220_000,
            "đi" | "đến" | "ở" | "làm" | "nói" | "biết" | "nghĩ" | "thích" | "xem" | "ăn"
            | "uống" | "nghe" | "học" | "mua" | "gặp" => 150_000,
            "và" | "với" | "cho" | "của" | "trong" | "này" | "đó" | "rất" | "đã" | "đang"
            | "sẽ" => 120_000,
            _ => 0,
        };
    }
    bonus
}

fn normalize_input(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict() -> VietnameseDictionary {
        VietnameseDictionary::from_sources(
            "tôi\ttooi\t9000\ntối\ttoois\t8900\ntốt\ttoots\t8800\nyêu\tyeeu\t8500\nmuốn\tmuoons\t8400\nlà\tlaf\t9300\nla\tla\t9000\ngì\tgif\t8700\ntiếng\ttieengs\t8200\nViệt\tvieetj\t8000\n",
            "tiếng Việt\ttieengs vieetj\t9500\ntôi yêu\ttooi yeeu\t8800\n",
        )
    }

    #[test]
    fn phrase_matches_spaced_and_compact_telex() {
        let spaced = dict().query("tieengs vieetj", |_, _| 0);
        assert_eq!(spaced.first().map(|c| c.text.as_str()), Some("tiếng Việt"));
        let compact = dict().query("tieengsvieetj", |_, _| 0);
        assert_eq!(compact.first().map(|c| c.text.as_str()), Some("tiếng Việt"));
    }

    #[test]
    fn combines_words_and_keeps_raw_fallback() {
        let candidates = dict().query("tooi yeeu tieengs vieetj", |_, _| 0);
        assert!(candidates.iter().any(|c| c.text == "tôi yêu tiếng Việt"));
        assert_eq!(
            candidates.last().map(|c| c.text.as_str()),
            Some("tooi yeeu tieengs vieetj")
        );
    }

    #[test]
    fn prefix_input_suggests_phrases_and_words() {
        let candidates = dict().query("tooi", |_, _| 0);
        assert!(candidates.iter().any(|c| c.text == "tôi yêu"));
        assert_eq!(candidates.first().map(|c| c.text.as_str()), Some("tôi"));
    }

    #[test]
    fn last_word_prefix_combines_with_previous_words() {
        let candidates = dict().query("tooi m", |_, _| 0);
        assert_eq!(
            candidates.first().map(|c| c.text.as_str()),
            Some("tôi muốn")
        );
    }

    #[test]
    fn common_function_words_beat_literal_unaccented_matches() {
        let candidates = dict().query("tooi la", |_, _| 0);
        assert_eq!(candidates.first().map(|c| c.text.as_str()), Some("tôi là"));
    }

    #[test]
    fn omitted_tone_keys_still_match_accented_words() {
        let candidates = dict().query("la gi", |_, _| 0);
        assert_eq!(candidates.first().map(|c| c.text.as_str()), Some("là gì"));
    }

    #[test]
    fn short_prefix_lists_word_candidates() {
        let candidates = dict().query("to", |_, _| 0);
        let texts: Vec<&str> = candidates.iter().take(4).map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"tôi"));
        assert!(texts.contains(&"tối"));
        assert!(texts.contains(&"tốt"));
    }

    #[test]
    fn user_choice_lifts_candidate() {
        let candidates = dict().query("tieengs vieetj", |_, text| {
            u32::from(text == "tiếng Việt") * 10
        });
        assert_eq!(
            candidates.first().map(|c| c.text.as_str()),
            Some("tiếng Việt")
        );
    }
}
