use crate::data::GameContent;

use super::{support_pair_index_for_ids, support_pairs, MatchSetup, MatchState, PlayerId};

#[test]
fn support_pair_lookup_ignores_selection_order() {
    let content = GameContent::load().unwrap_or_default();
    let definitions = &content.magical_girls;
    assert!(
        definitions.len() >= 3,
        "need a main plus two supports to exercise pair lookup"
    );

    let main_id = definitions[0].id.clone();
    let ascending = vec![definitions[1].id.clone(), definitions[2].id.clone()];
    let descending = vec![definitions[2].id.clone(), definitions[1].id.clone()];

    // The campaign hub appends supports in click order, so the same pair can
    // arrive either way round; both must resolve to the same pair index.
    assert_eq!(
        support_pair_index_for_ids(definitions, &main_id, &ascending),
        support_pair_index_for_ids(definitions, &main_id, &descending),
    );
}

#[test]
fn support_pairs_exclude_the_selected_main() {
    let pairs = support_pairs(5, 1);
    assert!(!pairs.iter().any(|pair| pair.contains(&1)));
    assert_eq!(pairs.len(), 6);
}

#[test]
fn setup_builds_hidden_supports_for_both_players() {
    let content = GameContent::load().unwrap_or_default();
    let setup = MatchSetup::default_for_content(&content);
    let state = MatchState::from_setup(&content, &setup);

    assert_eq!(state.player_a.magical_girls.main_character_id, "yuki");
    assert_eq!(state.player_b.baddies.main_character_id, "velvet_hex");
    assert_eq!(state.player_a.magical_girls.supports.len(), 2);
    assert!(state.player_a.magical_girls.supports[0].revealed);
    assert!(!state.player_a.magical_girls.supports[1].revealed);
    assert_eq!(state.active_player, PlayerId::PlayerA);
    assert_eq!(state.player_a.hand.len(), 5);
    assert_eq!(state.player_b.hand.len(), 5);
}
