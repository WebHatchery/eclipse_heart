//! Versioned collection save data.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectionCardKind {
    MagicalGirl,
    Baddie,
    StoryCard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectionSave {
    pub version: u32,
    #[serde(deserialize_with = "deserialize_inventory")]
    pub owned_magical_girls: HashMap<String, u32>,
    #[serde(deserialize_with = "deserialize_inventory")]
    pub owned_baddies: HashMap<String, u32>,
    #[serde(deserialize_with = "deserialize_inventory")]
    pub owned_story_cards: HashMap<String, u32>,
}

impl CollectionSave {
    pub fn add_owned(&mut self, kind: CollectionCardKind, id: &str, amount: u32) {
        if amount == 0 {
            return;
        }

        let inventory = self.inventory_mut(kind);
        *inventory.entry(id.to_owned()).or_insert(0) += amount;
    }

    pub fn owned_count(&self, kind: CollectionCardKind, id: &str) -> u32 {
        self.inventory(kind).get(id).copied().unwrap_or(0)
    }

    pub fn story_cards_available_for_deck(&self, card_id: &str, cards_in_deck: usize) -> u32 {
        self.owned_count(CollectionCardKind::StoryCard, card_id)
            .saturating_sub(cards_in_deck as u32)
    }

    pub fn ensure_full_roster_owned(
        &mut self,
        magical_girl_ids: impl Iterator<Item = String>,
        baddie_ids: impl Iterator<Item = String>,
    ) {
        for id in magical_girl_ids {
            self.ensure_min_owned(CollectionCardKind::MagicalGirl, &id, 3);
        }
        for id in baddie_ids {
            self.ensure_min_owned(CollectionCardKind::Baddie, &id, 3);
        }
    }

    pub fn ensure_min_owned(&mut self, kind: CollectionCardKind, id: &str, minimum: u32) {
        let inventory = self.inventory_mut(kind);
        let owned = inventory.get(id).copied().unwrap_or(0);
        if owned < minimum {
            inventory.insert(id.to_owned(), minimum);
        }
    }

    fn inventory(&self, kind: CollectionCardKind) -> &HashMap<String, u32> {
        match kind {
            CollectionCardKind::MagicalGirl => &self.owned_magical_girls,
            CollectionCardKind::Baddie => &self.owned_baddies,
            CollectionCardKind::StoryCard => &self.owned_story_cards,
        }
    }

    fn inventory_mut(&mut self, kind: CollectionCardKind) -> &mut HashMap<String, u32> {
        match kind {
            CollectionCardKind::MagicalGirl => &mut self.owned_magical_girls,
            CollectionCardKind::Baddie => &mut self.owned_baddies,
            CollectionCardKind::StoryCard => &mut self.owned_story_cards,
        }
    }
}

impl Default for CollectionSave {
    fn default() -> Self {
        Self {
            version: 2,
            owned_magical_girls: HashMap::new(),
            owned_baddies: HashMap::new(),
            owned_story_cards: HashMap::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum InventoryRepr {
    Counts(HashMap<String, u32>),
    Legacy(Vec<String>),
}

fn deserialize_inventory<'de, D>(deserializer: D) -> Result<HashMap<String, u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let repr = Option::<InventoryRepr>::deserialize(deserializer)?;
    Ok(match repr {
        Some(InventoryRepr::Counts(entries)) => entries,
        Some(InventoryRepr::Legacy(entries)) => {
            let mut counts = HashMap::new();
            for id in entries {
                *counts.entry(id).or_insert(0) += 1;
            }
            counts
        }
        None => HashMap::new(),
    })
}

#[cfg(test)]
mod tests;
