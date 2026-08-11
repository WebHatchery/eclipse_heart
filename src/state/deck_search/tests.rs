use crate::data::{CardAlignment, CardEffect, CardSpeed, StoryCardDefinition};

use super::{DeckSearchCardContext, DeckSearchQuery};

fn sample_card() -> StoryCardDefinition {
    StoryCardDefinition {
        id: "quiet_lunch_on_the_rooftop".to_owned(),
        name: "Quiet Lunch on the Rooftop".to_owned(),
        card_type: "bond".to_owned(),
        speed: CardSpeed::DailyLife,
        alignment: CardAlignment::MagicalGirl,
        playable_in_daily_life: true,
        effects: vec![
            CardEffect::GainMainPowerNextEncounter { amount: 1 },
            CardEffect::RevealFirstHiddenOwnSupport,
        ],
    }
}

#[test]
fn free_text_search_matches_name_and_effect_text() {
    let query = DeckSearchQuery::parse("quiet reveal");
    assert!(query.matches(
        &sample_card(),
        DeckSearchCardContext {
            owned_count: 2,
            available_count: 1,
            copies_in_deck: 1,
        }
    ));
}

#[test]
fn tag_search_matches_speed_alignment_and_type() {
    let query = DeckSearchQuery::parse("speed:daily align:mg type:bond");
    assert!(query.matches(&sample_card(), DeckSearchCardContext::default()));
}

#[test]
fn ownership_and_copy_filters_match_softly() {
    let owned_query = DeckSearchQuery::parse("owned copies:2");
    let missing_query = DeckSearchQuery::parse("missing speed:oops");

    assert!(owned_query.matches(
        &sample_card(),
        DeckSearchCardContext {
            owned_count: 3,
            available_count: 1,
            copies_in_deck: 2,
        }
    ));
    assert!(missing_query.matches(
        &sample_card(),
        DeckSearchCardContext {
            owned_count: 1,
            available_count: 0,
            copies_in_deck: 1,
        }
    ));
}
