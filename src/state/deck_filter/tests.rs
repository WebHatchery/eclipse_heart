use crate::data::{CardAlignment, CardEffect, CardSpeed, StoryCardDefinition};

use super::DeckFilterState;
use crate::state::DeckSearchCardContext;

fn sample_card() -> StoryCardDefinition {
    StoryCardDefinition {
        id: "sample".to_owned(),
        name: "Sample".to_owned(),
        card_type: "bond".to_owned(),
        speed: CardSpeed::DailyLife,
        alignment: CardAlignment::MagicalGirl,
        playable_in_daily_life: true,
        effects: vec![CardEffect::RevealFirstHiddenOwnSupport],
    }
}

#[test]
fn facet_filters_stack_together() {
    let mut filters = DeckFilterState::default();
    filters.toggle_speed(CardSpeed::DailyLife);
    filters.toggle_alignment(CardAlignment::MagicalGirl);
    filters.toggle_card_type("bond");
    filters.owned_only = true;
    filters.in_deck_only = true;

    assert!(filters.matches(
        &sample_card(),
        DeckSearchCardContext {
            owned_count: 1,
            available_count: 0,
            copies_in_deck: 2,
        }
    ));

    assert!(!filters.matches(
        &sample_card(),
        DeckSearchCardContext {
            owned_count: 0,
            available_count: 0,
            copies_in_deck: 2,
        }
    ));
}

#[test]
fn clear_resets_all_filters() {
    let mut filters = DeckFilterState::default();
    filters.toggle_speed(CardSpeed::Reaction);
    filters.missing_only = true;
    filters.clear();

    assert!(!filters.has_active_filters());
    assert!(filters.matches(&sample_card(), DeckSearchCardContext::default()));
}
