//! 越南语 Telex 模式的 Engine 级测试。

use super::{CountingLearner, HashMap};
use crate::candidate::CandidateKind;
use crate::dictionary::Dictionary;
use crate::{Engine, VietnameseDictionary};

fn vietnamese_engine() -> Engine {
    let vietnamese = VietnameseDictionary::from_sources(
        "tôi\ttooi\t9000\nyêu\tyeeu\t8500\nyếu\tyeeu\t8400\ntiếng\ttieengs\t8200\nViệt\tvieetj\t8000\nviệt\tvieetj\t7900\n",
        "tiếng Việt\ttieengs vieetj\t9500\ntiếng việt\ttieengs vieetj\t9400\ntôi yêu\ttooi yeeu\t8800\ntôi yếu\ttooi yeeu\t8700\n",
    );
    let mut engine = Engine::new(Dictionary::parse("的\tde\t1\n").unwrap())
        .with_vietnamese(vietnamese)
        .with_learner(Box::new(CountingLearner(HashMap::new())));
    engine.set_vietnamese_telex_mode(true);
    engine
}

#[test]
fn compact_telex_returns_phrase_and_raw_fallback() {
    let mut engine = vietnamese_engine();
    engine.set_input("tieengsvieetj");
    let query = engine.query().unwrap();
    assert_eq!(query.candidates.items[0].text, "tiếng Việt");
    assert_eq!(
        query.candidates.items[0].kind,
        CandidateKind::VietnamesePhrase
    );
    assert!(
        query
            .candidates
            .items
            .iter()
            .any(|candidate| candidate.text == "tieengsvieetj"
                && candidate.kind == CandidateKind::VietnameseRaw)
    );
}

#[test]
fn spaced_telex_combines_words_into_sentence_candidate() {
    let mut engine = vietnamese_engine();
    engine.set_input("tooi yeeu tieengs vieetj");
    let query = engine.query().unwrap();
    assert!(
        query
            .candidates
            .items
            .iter()
            .any(|candidate| candidate.text == "tôi yêu tiếng Việt")
    );
}

#[test]
fn chosen_vietnamese_candidate_is_lifted_next_time() {
    let mut engine = vietnamese_engine();
    engine.set_input("tieengs vieetj");
    let second = engine.query().unwrap().candidates.items[1].clone();
    assert_eq!(second.text, "tiếng việt");
    engine.commit(&second);

    engine.set_input("tieengs vieetj");
    let query = engine.query().unwrap();
    assert_eq!(query.candidates.items[0].text, "tiếng việt");
}
