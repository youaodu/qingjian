//! 越南语 Telex 查询入口。

use super::*;
use crate::vietnamese::{VietnameseKind, transcribe};

impl Engine {
    pub(super) fn query_vietnamese(&self, keys: &str, start: Instant) -> Query {
        let candidates = match &self.vietnamese {
            Some(dictionary) => {
                dictionary.query(keys, |input, text| self.learner.choice_weight(input, text))
            }
            None => Vec::new(),
        };
        let mut items: Vec<Candidate> = candidates
            .into_iter()
            .map(|candidate| Candidate {
                text: candidate.text,
                kind: match candidate.kind {
                    VietnameseKind::Word => CandidateKind::VietnameseWord,
                    VietnameseKind::Phrase => CandidateKind::VietnamesePhrase,
                    VietnameseKind::Raw => CandidateKind::VietnameseRaw,
                },
                syllables: Vec::new(),
                reading: None,
                translation: None,
                aux_code: None,
            })
            .collect();
        if items.is_empty() && !keys.is_empty() {
            let converted = transcribe(keys);
            if converted != keys {
                items.push(Candidate {
                    text: converted,
                    kind: CandidateKind::VietnamesePhrase,
                    syllables: Vec::new(),
                    reading: None,
                    translation: None,
                    aux_code: None,
                });
            }
            items.push(Candidate {
                text: keys.to_owned(),
                kind: CandidateKind::VietnameseRaw,
                syllables: Vec::new(),
                reading: None,
                translation: None,
                aux_code: None,
            });
        }
        Query {
            candidates: CandidateList { items },
            tail: String::new(),
            text: self.composition.typed_text(),
            cursor: self.composition.cursor(),
            rest: String::new(),
            decoded_keys: true,
            typed_display: Some(transcribe(keys)),
            timings: Timings {
                parse: Duration::ZERO,
                lookup: start.elapsed(),
                rank: Duration::ZERO,
            },
            ..Query::default()
        }
    }
}
