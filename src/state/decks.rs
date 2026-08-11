//! Versioned deck and roster preset save data.

use serde::{Deserialize, Serialize};

use crate::data::{DeckRules, StarterLoadout};

use super::{timestamp::current_unix_timestamp, CollectionSave, ImportedDeck};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DeckPreset {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub source_template_id: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub archetype_tags: Vec<String>,
    pub story_cards: Vec<String>,
    #[serde(default)]
    pub magical_girl_roster: Vec<String>,
    #[serde(default)]
    pub baddie_roster: Vec<String>,
    #[serde(default = "current_unix_timestamp")]
    pub created_at_unix: i64,
    #[serde(default = "current_unix_timestamp")]
    pub updated_at_unix: i64,
    #[serde(default)]
    pub recent_story_cards: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecksSave {
    pub version: u32,
    pub support_decks: Vec<DeckPreset>,
    #[serde(default)]
    pub selected_support_deck_id: Option<String>,
    #[serde(default)]
    pub roster_presets: Vec<String>,
    #[serde(default, skip_serializing)]
    legacy_active_support_deck_index: usize,
    #[serde(default, skip_serializing)]
    undo_snapshot: Option<DeckPreset>,
}

impl Default for DecksSave {
    fn default() -> Self {
        Self {
            version: 3,
            support_decks: Vec::new(),
            selected_support_deck_id: None,
            roster_presets: Vec::new(),
            legacy_active_support_deck_index: 0,
            undo_snapshot: None,
        }
    }
}

impl DecksSave {
    pub fn ensure_valid_support_decks(
        &mut self,
        magical_girl_ids: &[String],
        baddie_ids: &[String],
    ) {
        for deck in &mut self.support_decks {
            fill_missing_roster(&mut deck.magical_girl_roster, magical_girl_ids);
            fill_missing_roster(&mut deck.baddie_roster, baddie_ids);
            ensure_deck_metadata(deck);
        }

        self.selected_support_deck_id = self.resolve_selected_deck_id();
        self.version = self.version.max(3);
    }

    pub fn selected_support_deck(&self) -> Option<&DeckPreset> {
        let selected_id = self.selected_support_deck_id.as_deref()?;
        self.support_decks
            .iter()
            .find(|deck| deck.id == selected_id)
    }

    pub fn selected_support_deck_mut(&mut self) -> Option<&mut DeckPreset> {
        let selected_id = self.selected_support_deck_id.clone()?;
        self.support_decks
            .iter_mut()
            .find(|deck| deck.id == selected_id)
    }

    pub fn select_support_deck(&mut self, deck_id: &str) -> bool {
        if self.support_decks.iter().any(|deck| deck.id == deck_id) {
            self.selected_support_deck_id = Some(deck_id.to_owned());
            return true;
        }

        false
    }

    pub fn deck_for_template(&self, starter_id: &str) -> Option<&DeckPreset> {
        self.support_decks
            .iter()
            .find(|deck| deck.source_template_id.as_deref() == Some(starter_id))
    }

    pub fn card_count(&self, card_id: &str) -> usize {
        self.selected_support_deck()
            .map(|deck| {
                deck.story_cards
                    .iter()
                    .filter(|entry| entry.as_str() == card_id)
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn can_add_card(
        &self,
        card_id: &str,
        deck_rules: &DeckRules,
        collection: &CollectionSave,
    ) -> bool {
        let Some(deck) = self.selected_support_deck() else {
            return false;
        };

        deck.story_cards.len() < deck_rules.support_deck_size
            && self.card_count(card_id) < deck_rules.max_copies_per_story_card
            && collection.story_cards_available_for_deck(card_id, self.card_count(card_id)) > 0
    }

    pub fn add_card(
        &mut self,
        card_id: &str,
        deck_rules: &DeckRules,
        collection: &CollectionSave,
    ) -> bool {
        if !self.can_add_card(card_id, deck_rules, collection) {
            return false;
        }

        self.snapshot_selected_deck();
        if let Some(deck) = self.selected_support_deck_mut() {
            deck.story_cards.push(card_id.to_owned());
            touch_deck(deck);
            push_recent_story_card(&mut deck.recent_story_cards, card_id);
            return true;
        }

        false
    }

    pub fn remove_card(&mut self, card_id: &str) -> bool {
        if self.selected_support_deck().is_none() {
            return false;
        }
        self.snapshot_selected_deck();

        let Some(deck) = self.selected_support_deck_mut() else {
            return false;
        };

        let Some(index) = deck.story_cards.iter().position(|entry| entry == card_id) else {
            return false;
        };

        deck.story_cards.remove(index);
        touch_deck(deck);
        push_recent_story_card(&mut deck.recent_story_cards, card_id);
        true
    }

    pub fn create_empty_deck(
        &mut self,
        base_name: &str,
        magical_girl_ids: &[String],
        baddie_ids: &[String],
    ) -> String {
        let deck_id = self.next_deck_id();
        let deck_name = self.unique_deck_name(base_name);
        let now = current_unix_timestamp();
        self.support_decks.push(DeckPreset {
            id: deck_id.clone(),
            name: deck_name,
            source_template_id: None,
            notes: String::new(),
            archetype_tags: Vec::new(),
            story_cards: Vec::new(),
            magical_girl_roster: magical_girl_ids.to_vec(),
            baddie_roster: baddie_ids.to_vec(),
            created_at_unix: now,
            updated_at_unix: now,
            recent_story_cards: Vec::new(),
        });
        self.selected_support_deck_id = Some(deck_id.clone());
        self.undo_snapshot = None;
        deck_id
    }

    pub fn create_deck_from_template(
        &mut self,
        starter: &StarterLoadout,
        magical_girl_ids: &[String],
        baddie_ids: &[String],
        copy_suffix: &str,
    ) -> String {
        let deck_id = self.next_deck_id();
        let preferred_name = if self.deck_name_count(&starter.name) == 0 {
            starter.name.clone()
        } else {
            format!("{} {copy_suffix}", starter.name)
        };
        let deck_name = self.unique_deck_name(&preferred_name);
        let now = current_unix_timestamp();
        let magical_girl_roster = build_template_roster(
            &starter.magical_girl_main,
            &starter.magical_girl_supports,
            magical_girl_ids,
        );
        let baddie_roster =
            build_template_roster(&starter.prime_baddie, &starter.baddie_supports, baddie_ids);
        let deck = DeckPreset {
            id: deck_id.clone(),
            name: deck_name,
            source_template_id: Some(starter.id.clone()),
            notes: starter.description.clone(),
            archetype_tags: normalized_tags(std::slice::from_ref(&starter.playstyle)),
            story_cards: starter.support_deck.clone(),
            magical_girl_roster,
            baddie_roster,
            created_at_unix: now,
            updated_at_unix: now,
            recent_story_cards: starter.support_deck.iter().take(5).cloned().collect(),
        };
        self.support_decks.push(deck);
        self.selected_support_deck_id = Some(deck_id.clone());
        self.undo_snapshot = None;

        deck_id
    }

    pub fn rename_selected_deck(&mut self, new_name: &str) -> bool {
        let trimmed_name = new_name.trim();
        if trimmed_name.is_empty() {
            return false;
        }

        let selected_id = self.selected_support_deck_id.clone();
        let Some(selected_id) = selected_id else {
            return false;
        };
        if !self.support_decks.iter().any(|deck| deck.id == selected_id) {
            return false;
        }

        let unique_name = self.unique_deck_name_for(trimmed_name, Some(selected_id.as_str()));
        self.snapshot_selected_deck();
        if let Some(deck) = self.selected_support_deck_mut() {
            deck.name = unique_name;
            touch_deck(deck);
            return true;
        }

        false
    }

    pub fn duplicate_selected_deck(&mut self, copy_suffix: &str) -> Option<String> {
        let source_deck = self.selected_support_deck()?.clone();
        let deck_id = self.next_deck_id();
        let deck_name = self.unique_deck_name(&format!("{} {copy_suffix}", source_deck.name));
        let now = current_unix_timestamp();
        self.support_decks.push(DeckPreset {
            id: deck_id.clone(),
            name: deck_name,
            source_template_id: source_deck.source_template_id,
            notes: source_deck.notes,
            archetype_tags: source_deck.archetype_tags,
            story_cards: source_deck.story_cards,
            magical_girl_roster: source_deck.magical_girl_roster,
            baddie_roster: source_deck.baddie_roster,
            created_at_unix: now,
            updated_at_unix: now,
            recent_story_cards: source_deck.recent_story_cards,
        });
        self.selected_support_deck_id = Some(deck_id.clone());
        self.undo_snapshot = None;
        Some(deck_id)
    }

    pub fn delete_selected_deck(&mut self) -> bool {
        let Some(selected_id) = self.selected_support_deck_id.clone() else {
            return false;
        };
        let Some(index) = self
            .support_decks
            .iter()
            .position(|deck| deck.id == selected_id)
        else {
            return false;
        };

        self.support_decks.remove(index);
        self.selected_support_deck_id = self.resolve_selected_deck_id();
        self.undo_snapshot = None;
        true
    }

    pub fn import_deck(&mut self, imported: ImportedDeck, default_name: &str) -> String {
        let ImportedDeck {
            name,
            story_cards,
            magical_girl_roster,
            baddie_roster,
        } = imported;
        let deck_id = self.next_deck_id();
        let preferred_name = if name.trim().is_empty() {
            default_name
        } else {
            name.trim()
        };
        let deck_name = self.unique_deck_name(preferred_name);
        let now = current_unix_timestamp();
        let recent_story_cards = story_cards.iter().take(5).cloned().collect();
        self.support_decks.push(DeckPreset {
            id: deck_id.clone(),
            name: deck_name,
            source_template_id: None,
            notes: String::new(),
            archetype_tags: Vec::new(),
            story_cards,
            magical_girl_roster,
            baddie_roster,
            created_at_unix: now,
            updated_at_unix: now,
            recent_story_cards,
        });
        self.selected_support_deck_id = Some(deck_id.clone());
        self.undo_snapshot = None;
        deck_id
    }

    pub fn set_roster_slot(
        &mut self,
        is_magical_girl_side: bool,
        slot_index: usize,
        character_id: &str,
    ) -> bool {
        if self.selected_support_deck().is_none() {
            return false;
        }
        self.snapshot_selected_deck();

        let Some(deck) = self.selected_support_deck_mut() else {
            return false;
        };

        let roster = if is_magical_girl_side {
            &mut deck.magical_girl_roster
        } else {
            &mut deck.baddie_roster
        };

        if slot_index >= roster.len() {
            return false;
        }

        if let Some(existing_index) = roster.iter().position(|entry| entry == character_id) {
            roster.swap(slot_index, existing_index);
            touch_deck(deck);
            return true;
        }

        roster[slot_index] = character_id.to_owned();
        touch_deck(deck);
        true
    }

    pub fn update_selected_deck_metadata(&mut self, notes: &str, tags: &[String]) -> bool {
        let Some(_) = self.selected_support_deck() else {
            return false;
        };
        self.snapshot_selected_deck();
        if let Some(deck) = self.selected_support_deck_mut() {
            deck.notes = notes.trim().to_owned();
            deck.archetype_tags = normalized_tags(tags);
            touch_deck(deck);
            return true;
        }
        false
    }

    pub fn undo_selected_deck_change(&mut self) -> bool {
        let Some(snapshot) = self.undo_snapshot.clone() else {
            return false;
        };
        let Some(index) = self
            .support_decks
            .iter()
            .position(|deck| deck.id == snapshot.id)
        else {
            self.undo_snapshot = None;
            return false;
        };

        self.support_decks[index] = snapshot.clone();
        self.selected_support_deck_id = Some(snapshot.id);
        self.undo_snapshot = None;
        true
    }

    pub fn can_undo_selected_deck_change(&self) -> bool {
        self.undo_snapshot.is_some()
    }

    pub fn reset_selected_deck_to_template(&mut self, starter: &StarterLoadout) -> bool {
        let Some(deck) = self.selected_support_deck() else {
            return false;
        };
        if deck.source_template_id.as_deref() != Some(starter.id.as_str()) {
            return false;
        }

        self.snapshot_selected_deck();
        if let Some(deck) = self.selected_support_deck_mut() {
            deck.story_cards = starter.support_deck.clone();
            deck.magical_girl_roster = build_template_roster(
                &starter.magical_girl_main,
                &starter.magical_girl_supports,
                &deck.magical_girl_roster.clone(),
            );
            deck.baddie_roster = build_template_roster(
                &starter.prime_baddie,
                &starter.baddie_supports,
                &deck.baddie_roster.clone(),
            );
            deck.recent_story_cards = starter.support_deck.iter().take(5).cloned().collect();
            touch_deck(deck);
            return true;
        }

        false
    }

    pub fn recently_modified_deck_ids(&self, limit: usize) -> Vec<String> {
        let mut decks = self
            .support_decks
            .iter()
            .map(|deck| (deck.id.clone(), deck.updated_at_unix))
            .collect::<Vec<_>>();
        decks.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        decks.into_iter().take(limit).map(|(id, _)| id).collect()
    }

    fn snapshot_selected_deck(&mut self) {
        self.undo_snapshot = self.selected_support_deck().cloned();
    }

    fn resolve_selected_deck_id(&self) -> Option<String> {
        if let Some(selected_id) = self.selected_support_deck_id.as_deref() {
            if self.support_decks.iter().any(|deck| deck.id == selected_id) {
                return Some(selected_id.to_owned());
            }
        }

        self.support_decks
            .get(self.legacy_active_support_deck_index)
            .or_else(|| self.support_decks.first())
            .map(|deck| deck.id.clone())
    }

    fn next_deck_id(&self) -> String {
        let mut next_index = self.support_decks.len() + 1;
        loop {
            let candidate = format!("deck_{next_index}");
            if !self.support_decks.iter().any(|deck| deck.id == candidate) {
                return candidate;
            }
            next_index += 1;
        }
    }

    fn deck_name_count(&self, name: &str) -> usize {
        self.support_decks
            .iter()
            .filter(|deck| deck.name == name)
            .count()
    }

    fn unique_deck_name(&self, desired_name: &str) -> String {
        self.unique_deck_name_for(desired_name, None)
    }

    fn unique_deck_name_for(&self, desired_name: &str, ignored_deck_id: Option<&str>) -> String {
        let trimmed_name = desired_name.trim();
        if trimmed_name.is_empty() {
            return "Deck".to_owned();
        }

        if !self.support_decks.iter().any(|deck| {
            Some(deck.id.as_str()) != ignored_deck_id
                && deck.name.eq_ignore_ascii_case(trimmed_name)
        }) {
            return trimmed_name.to_owned();
        }

        let mut suffix = 2;
        loop {
            let candidate = format!("{trimmed_name} {suffix}");
            let already_used = self.support_decks.iter().any(|deck| {
                Some(deck.id.as_str()) != ignored_deck_id
                    && deck.name.eq_ignore_ascii_case(&candidate)
            });
            if !already_used {
                return candidate;
            }
            suffix += 1;
        }
    }
}

fn fill_missing_roster(roster: &mut Vec<String>, fallback: &[String]) {
    if roster.len() == fallback.len() && roster.iter().all(|entry| fallback.contains(entry)) {
        return;
    }

    *roster = fallback.to_vec();
}

fn touch_deck(deck: &mut DeckPreset) {
    deck.updated_at_unix = current_unix_timestamp();
}

fn ensure_deck_metadata(deck: &mut DeckPreset) {
    if deck.created_at_unix <= 0 {
        deck.created_at_unix = current_unix_timestamp();
    }
    if deck.updated_at_unix <= 0 {
        deck.updated_at_unix = deck.created_at_unix;
    }
    deck.archetype_tags = normalized_tags(&deck.archetype_tags);
    if deck.recent_story_cards.is_empty() {
        deck.recent_story_cards = deck.story_cards.iter().rev().take(5).cloned().collect();
        deck.recent_story_cards.reverse();
    } else {
        deck.recent_story_cards.truncate(5);
    }
}

fn normalized_tags(tags: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if normalized
            .iter()
            .any(|entry: &String| entry.eq_ignore_ascii_case(trimmed))
        {
            continue;
        }
        normalized.push(trimmed.to_owned());
    }
    normalized
}

fn push_recent_story_card(recent_story_cards: &mut Vec<String>, card_id: &str) {
    recent_story_cards.retain(|entry| entry != card_id);
    recent_story_cards.push(card_id.to_owned());
    if recent_story_cards.len() > 5 {
        recent_story_cards.remove(0);
    }
}

fn build_template_roster(
    main_id: &str,
    support_ids: &[String],
    fallback: &[String],
) -> Vec<String> {
    let mut roster = Vec::new();
    roster.push(main_id.to_owned());

    for support_id in support_ids {
        push_unique_card_id(&mut roster, support_id);
    }

    for fallback_id in fallback {
        push_unique_card_id(&mut roster, fallback_id);
    }

    roster.truncate(fallback.len());
    if roster.len() == fallback.len() {
        roster
    } else {
        fallback.to_vec()
    }
}

fn push_unique_card_id(entries: &mut Vec<String>, candidate: &str) {
    if !entries.iter().any(|entry| entry == candidate) {
        entries.push(candidate.to_owned());
    }
}

#[cfg(test)]
mod tests;
