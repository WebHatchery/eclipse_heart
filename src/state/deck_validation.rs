//! Non-persistent deck validation helpers for the deck builder.

use std::collections::BTreeMap;

use crate::data::DeckRules;

use super::{CollectionCardKind, CollectionSave, DeckPreset};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeckValidationCount {
    pub id: String,
    pub count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeckValidation {
    pub support_card_count: usize,
    pub required_support_card_count: usize,
    pub support_card_count_valid: bool,
    pub magical_girl_roster_count: usize,
    pub required_magical_girl_roster_count: usize,
    pub magical_girl_roster_valid: bool,
    pub baddie_roster_count: usize,
    pub required_baddie_roster_count: usize,
    pub baddie_roster_valid: bool,
    pub duplicate_story_cards: Vec<String>,
    pub duplicate_magical_girls: Vec<String>,
    pub duplicate_baddies: Vec<String>,
    pub missing_story_cards: Vec<DeckValidationCount>,
    pub missing_magical_girls: Vec<String>,
    pub missing_baddies: Vec<String>,
    pub missing_card_total: usize,
    pub is_collection_complete: bool,
    pub is_legal: bool,
}

impl DeckValidation {
    pub fn for_deck(
        deck: &DeckPreset,
        deck_rules: &DeckRules,
        collection: &CollectionSave,
    ) -> Self {
        let support_card_count = deck.story_cards.len();
        let required_support_card_count = deck_rules.support_deck_size;
        let support_card_count_valid = support_card_count == required_support_card_count;

        let magical_girl_roster_count = deck.magical_girl_roster.len();
        let required_magical_girl_roster_count = 5;
        let magical_girl_roster_valid =
            magical_girl_roster_count == required_magical_girl_roster_count;

        let baddie_roster_count = deck.baddie_roster.len();
        let required_baddie_roster_count = 5;
        let baddie_roster_valid = baddie_roster_count == required_baddie_roster_count;

        let story_card_counts = count_entries(&deck.story_cards);
        let magical_girl_counts = count_entries(&deck.magical_girl_roster);
        let baddie_counts = count_entries(&deck.baddie_roster);

        let duplicate_story_cards =
            duplicate_ids_for_counts(&story_card_counts, deck_rules.max_copies_per_story_card);
        let duplicate_magical_girls = duplicate_ids_for_counts(&magical_girl_counts, 1);
        let duplicate_baddies = duplicate_ids_for_counts(&baddie_counts, 1);

        let missing_story_cards = missing_owned_counts(
            &story_card_counts,
            collection,
            CollectionCardKind::StoryCard,
        );
        let missing_magical_girls = missing_owned_ids(
            &magical_girl_counts,
            collection,
            CollectionCardKind::MagicalGirl,
        );
        let missing_baddies =
            missing_owned_ids(&baddie_counts, collection, CollectionCardKind::Baddie);

        let missing_card_total = missing_story_cards
            .iter()
            .map(|entry| entry.count)
            .sum::<usize>()
            + missing_magical_girls.len()
            + missing_baddies.len();
        let is_collection_complete = missing_card_total == 0;
        let has_duplicates = !duplicate_story_cards.is_empty()
            || !duplicate_magical_girls.is_empty()
            || !duplicate_baddies.is_empty();
        let has_roster_issues = !magical_girl_roster_valid
            || !baddie_roster_valid
            || !duplicate_magical_girls.is_empty()
            || !duplicate_baddies.is_empty();

        Self {
            support_card_count,
            required_support_card_count,
            support_card_count_valid,
            magical_girl_roster_count,
            required_magical_girl_roster_count,
            magical_girl_roster_valid,
            baddie_roster_count,
            required_baddie_roster_count,
            baddie_roster_valid,
            duplicate_story_cards,
            duplicate_magical_girls,
            duplicate_baddies,
            missing_story_cards,
            missing_magical_girls,
            missing_baddies,
            missing_card_total,
            is_collection_complete,
            is_legal: support_card_count_valid
                && !has_duplicates
                && !has_roster_issues
                && is_collection_complete,
        }
    }
}

fn count_entries(entries: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for entry in entries {
        *counts.entry(entry.clone()).or_insert(0) += 1;
    }
    counts
}

fn duplicate_ids_for_counts(counts: &BTreeMap<String, usize>, limit: usize) -> Vec<String> {
    counts
        .iter()
        .filter(|(_, count)| **count > limit)
        .map(|(id, _)| id.clone())
        .collect()
}

fn missing_owned_counts(
    counts: &BTreeMap<String, usize>,
    collection: &CollectionSave,
    kind: CollectionCardKind,
) -> Vec<DeckValidationCount> {
    counts
        .iter()
        .filter_map(|(id, count)| {
            let owned = collection.owned_count(kind, id) as usize;
            (owned < *count).then(|| DeckValidationCount {
                id: id.clone(),
                count: count - owned,
            })
        })
        .collect()
}

fn missing_owned_ids(
    counts: &BTreeMap<String, usize>,
    collection: &CollectionSave,
    kind: CollectionCardKind,
) -> Vec<String> {
    counts
        .iter()
        .filter(|(id, count)| (collection.owned_count(kind, id) as usize) < **count)
        .map(|(id, _)| id.clone())
        .collect()
}

#[cfg(test)]
mod tests;
