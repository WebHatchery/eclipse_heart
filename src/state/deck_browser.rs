//! Deck-builder browser sort, group, and view modes.

use std::cmp::Ordering;

use crate::data::{CardAlignment, CardSpeed, StoryCardDefinition};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeckSortMode {
    Alphabetical,
    Newest,
    OwnedCount,
    CopiesInDeck,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeckGroupMode {
    None,
    Alignment,
    Speed,
    CardType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeckViewMode {
    Grid,
    CompactList,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeckBrowserCardStats {
    pub original_index: usize,
    pub owned_count: u32,
    pub copies_in_deck: usize,
}

pub fn compare_story_cards(
    left: (&StoryCardDefinition, DeckBrowserCardStats),
    right: (&StoryCardDefinition, DeckBrowserCardStats),
    mode: DeckSortMode,
) -> Ordering {
    match mode {
        DeckSortMode::Alphabetical => left
            .0
            .name
            .cmp(&right.0.name)
            .then(left.0.id.cmp(&right.0.id)),
        DeckSortMode::Newest => right
            .1
            .original_index
            .cmp(&left.1.original_index)
            .then(left.0.name.cmp(&right.0.name)),
        DeckSortMode::OwnedCount => right
            .1
            .owned_count
            .cmp(&left.1.owned_count)
            .then(left.0.name.cmp(&right.0.name)),
        DeckSortMode::CopiesInDeck => right
            .1
            .copies_in_deck
            .cmp(&left.1.copies_in_deck)
            .then(left.0.name.cmp(&right.0.name)),
    }
}

pub fn card_group_label(card: &StoryCardDefinition, mode: DeckGroupMode) -> Option<String> {
    match mode {
        DeckGroupMode::None => None,
        DeckGroupMode::Alignment => Some(match card.alignment {
            CardAlignment::MagicalGirl => "Magical Girl".to_owned(),
            CardAlignment::Baddie => "Baddie".to_owned(),
            CardAlignment::Neutral => "Neutral".to_owned(),
        }),
        DeckGroupMode::Speed => Some(match card.speed {
            CardSpeed::DailyLife => "Daily".to_owned(),
            CardSpeed::Reaction => "Reaction".to_owned(),
            CardSpeed::Encounter => "Encounter".to_owned(),
        }),
        DeckGroupMode::CardType => Some(card.card_type.clone()),
    }
}

#[cfg(test)]
mod tests;
