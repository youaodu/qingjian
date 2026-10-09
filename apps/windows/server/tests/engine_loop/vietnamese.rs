//! Windows 中英切换键进入越南语 Telex 的闭环测试。

use super::support::*;

#[test]
fn switch_key_target_can_enter_vietnamese_telex() {
    let mut router = router_vietnamese();
    let (_, _, frame) = type_english(&mut router, "em");
    assert!(candidate_texts(&frame).contains(&"em"));

    let (outcome, commit, frame) = press(&mut router, KeyEvent::new(0x20, Some(' '), ENGLISH));
    assert_eq!(outcome, KeyOutcome::Consumed);
    assert_eq!(commit.as_deref(), Some("em"));
    assert!(frame.candidates.items.is_empty());

    let (_, _, frame) = type_english(&mut router, "trai");
    let texts = candidate_texts(&frame);
    assert!(texts.contains(&"trai"), "候选：{texts:?}");
    assert_eq!(texts.first().copied(), Some("trai"));
}

#[test]
fn caps_lock_still_outputs_english_instead_of_vietnamese() {
    let mut router = router_vietnamese();
    let (outcome, commit, frame) = press(&mut router, letter_with('E', CAPS));
    assert_eq!(outcome, KeyOutcome::Consumed);
    assert_eq!(commit.as_deref(), Some("E"));
    assert!(frame.candidates.items.is_empty());
}
