use crate::data::GameContent;

use super::round_flow::discard_player_down_to_hand_limit;
use super::{MatchSetup, MatchState, PlayerId};

#[test]
fn end_of_round_discards_down_to_seven_cards() {
    let content = GameContent::load().unwrap_or_default();
    let setup = MatchSetup::default_for_content(&content);
    let mut state = MatchState::from_setup(&content, &setup);

    state.player_a.hand.extend([
        "identity_reveal".to_string(),
        "break_the_formation".to_string(),
        "secret_rehearsal".to_string(),
        "crescent_counterpose".to_string(),
    ]);
    state.player_b.hand.extend([
        "ominous_graffiti".to_string(),
        "break_the_formation".to_string(),
        "panic_spiral".to_string(),
        "panic_in_the_parade_route".to_string(),
    ]);
    state.ready_end_of_round();

    assert_eq!(state.player_a.hand.len(), 9);
    assert_eq!(state.player_b.hand.len(), 9);
    assert!(!state.player_a.discard.is_empty());
    assert!(!state.player_b.discard.is_empty());
}

#[test]
fn discard_helper_moves_only_overflow_cards() {
    let mut hand = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
    ];
    let mut discard = Vec::new();

    let discarded = discard_player_down_to_hand_limit(&mut hand, &mut discard, 2);

    assert_eq!(discarded, 2);
    assert_eq!(hand, vec!["c".to_string(), "d".to_string()]);
    assert_eq!(discard, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn discard_helper_is_no_op_when_hand_is_within_limit() {
    let mut hand = vec!["a".to_string(), "b".to_string()];
    let mut discard = Vec::new();

    let discarded = discard_player_down_to_hand_limit(&mut hand, &mut discard, 2);

    assert_eq!(discarded, 0);
    assert_eq!(hand, vec!["a".to_string(), "b".to_string()]);
    assert!(discard.is_empty());
}

#[test]
fn setup_can_store_assigned_support_deck_ids() {
    let content = GameContent::load().unwrap_or_default();
    let mut setup = MatchSetup::default_for_content(&content);

    setup.assign_support_deck(PlayerId::PlayerA, Some("deck_7".to_owned()));
    setup.assign_support_deck(PlayerId::PlayerB, Some("deck_9".to_owned()));
    setup.clear_missing_support_deck_assignments(&["deck_9".to_owned()]);

    assert_eq!(setup.player_a_support_deck_id, None);
    assert_eq!(setup.player_b_support_deck_id.as_deref(), Some("deck_9"));
}
