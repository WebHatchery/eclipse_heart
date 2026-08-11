//! Portable deck-code import and export helpers.

use serde::{Deserialize, Serialize};

use crate::data::GameContent;

use super::DeckPreset;

const DECK_CODE_PREFIX: &str = "EH1:";
const REQUIRED_ROSTER_SIZE: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedDeck {
    pub name: String,
    pub story_cards: Vec<String>,
    pub magical_girl_roster: Vec<String>,
    pub baddie_roster: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeckCodeError {
    Empty,
    InvalidFormat,
    UnsupportedVersion(u32),
    InvalidMagicalGirlRosterCount(usize),
    InvalidBaddieRosterCount(usize),
    UnknownStoryCard(String),
    UnknownMagicalGirl(String),
    UnknownBaddie(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeckCodeV1 {
    version: u32,
    name: String,
    story_cards: Vec<String>,
    magical_girl_roster: Vec<String>,
    baddie_roster: Vec<String>,
}

pub fn export_deck_code(deck: &DeckPreset) -> String {
    let payload = DeckCodeV1 {
        version: 1,
        name: deck.name.clone(),
        story_cards: deck.story_cards.clone(),
        magical_girl_roster: deck.magical_girl_roster.clone(),
        baddie_roster: deck.baddie_roster.clone(),
    };

    format!(
        "{DECK_CODE_PREFIX}{}",
        serde_json::to_string(&payload).expect("deck code serialization should succeed")
    )
}

pub fn import_deck_code(code: &str, content: &GameContent) -> Result<ImportedDeck, DeckCodeError> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Err(DeckCodeError::Empty);
    }

    let payload_text = trimmed.strip_prefix(DECK_CODE_PREFIX).unwrap_or(trimmed);
    let payload = serde_json::from_str::<DeckCodeV1>(payload_text)
        .map_err(|_| DeckCodeError::InvalidFormat)?;

    if payload.version != 1 {
        return Err(DeckCodeError::UnsupportedVersion(payload.version));
    }
    if payload.magical_girl_roster.len() != REQUIRED_ROSTER_SIZE {
        return Err(DeckCodeError::InvalidMagicalGirlRosterCount(
            payload.magical_girl_roster.len(),
        ));
    }
    if payload.baddie_roster.len() != REQUIRED_ROSTER_SIZE {
        return Err(DeckCodeError::InvalidBaddieRosterCount(
            payload.baddie_roster.len(),
        ));
    }

    for card_id in &payload.story_cards {
        if !content.story_cards.iter().any(|entry| entry.id == *card_id) {
            return Err(DeckCodeError::UnknownStoryCard(card_id.clone()));
        }
    }
    for character_id in &payload.magical_girl_roster {
        if !content
            .magical_girls
            .iter()
            .any(|entry| entry.id == *character_id)
        {
            return Err(DeckCodeError::UnknownMagicalGirl(character_id.clone()));
        }
    }
    for character_id in &payload.baddie_roster {
        if !content
            .baddies
            .iter()
            .any(|entry| entry.id == *character_id)
        {
            return Err(DeckCodeError::UnknownBaddie(character_id.clone()));
        }
    }

    Ok(ImportedDeck {
        name: payload.name.trim().to_owned(),
        story_cards: payload.story_cards,
        magical_girl_roster: payload.magical_girl_roster,
        baddie_roster: payload.baddie_roster,
    })
}

#[cfg(test)]
mod tests;
