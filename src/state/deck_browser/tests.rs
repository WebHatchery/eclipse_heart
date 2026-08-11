use std::cmp::Ordering;

use crate::data::{CardAlignment, CardEffect, CardSpeed, StoryCardDefinition};

use super::{
    card_group_label, compare_story_cards, DeckBrowserCardStats, DeckGroupMode, DeckSortMode,
};

fn card(
    id: &str,
    name: &str,
    speed: CardSpeed,
    alignment: CardAlignment,
    card_type: &str,
) -> StoryCardDefinition {
    StoryCardDefinition {
        id: id.to_owned(),
        name: name.to_owned(),
        card_type: card_type.to_owned(),
        speed,
        alignment,
        playable_in_daily_life: speed == CardSpeed::DailyLife,
        effects: vec![CardEffect::RevealFirstHiddenOwnSupport],
    }
}

#[test]
fn sorting_uses_requested_mode() {
    let alpha = card(
        "a",
        "Alpha",
        CardSpeed::DailyLife,
        CardAlignment::MagicalGirl,
        "bond",
    );
    let beta = card(
        "b",
        "Beta",
        CardSpeed::Reaction,
        CardAlignment::Baddie,
        "scheme",
    );

    assert_eq!(
        compare_story_cards(
            (
                &alpha,
                DeckBrowserCardStats {
                    original_index: 0,
                    owned_count: 1,
                    copies_in_deck: 0
                }
            ),
            (
                &beta,
                DeckBrowserCardStats {
                    original_index: 1,
                    owned_count: 3,
                    copies_in_deck: 2
                }
            ),
            DeckSortMode::Alphabetical
        ),
        Ordering::Less
    );
    assert_eq!(
        compare_story_cards(
            (
                &alpha,
                DeckBrowserCardStats {
                    original_index: 0,
                    owned_count: 1,
                    copies_in_deck: 0
                }
            ),
            (
                &beta,
                DeckBrowserCardStats {
                    original_index: 1,
                    owned_count: 3,
                    copies_in_deck: 2
                }
            ),
            DeckSortMode::Newest
        ),
        Ordering::Greater
    );
}

#[test]
fn grouping_returns_expected_labels() {
    let sample = card(
        "a",
        "Alpha",
        CardSpeed::DailyLife,
        CardAlignment::MagicalGirl,
        "bond",
    );

    assert_eq!(
        card_group_label(&sample, DeckGroupMode::Alignment),
        Some("Magical Girl".to_owned())
    );
    assert_eq!(
        card_group_label(&sample, DeckGroupMode::Speed),
        Some("Daily".to_owned())
    );
    assert_eq!(
        card_group_label(&sample, DeckGroupMode::CardType),
        Some("bond".to_owned())
    );
}
