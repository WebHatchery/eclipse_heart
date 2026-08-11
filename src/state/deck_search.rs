//! Deck-builder support-card search parsing and matching.

use crate::data::{CardAlignment, CardEffect, CardSpeed, StoryCardDefinition};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeckSearchQuery {
    text_terms: Vec<String>,
    speed: Option<CardSpeed>,
    alignment: Option<CardAlignment>,
    card_type: Option<String>,
    require_owned: bool,
    require_missing: bool,
    copies_in_deck: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeckSearchCardContext {
    pub owned_count: u32,
    pub available_count: u32,
    pub copies_in_deck: usize,
}

impl DeckSearchQuery {
    pub fn parse(input: &str) -> Self {
        let mut query = Self::default();

        for raw_term in input.split_whitespace() {
            let term = raw_term.trim().to_ascii_lowercase();
            if term.is_empty() {
                continue;
            }

            if term == "owned" {
                query.require_owned = true;
                continue;
            }
            if term == "missing" {
                query.require_missing = true;
                continue;
            }

            if let Some((key, value)) = term.split_once(':') {
                match key {
                    "speed" => {
                        if let Some(speed) = parse_speed(value) {
                            query.speed = Some(speed);
                        }
                        continue;
                    }
                    "align" => {
                        if let Some(alignment) = parse_alignment(value) {
                            query.alignment = Some(alignment);
                        }
                        continue;
                    }
                    "type" => {
                        if !value.is_empty() {
                            query.card_type = Some(value.to_owned());
                        }
                        continue;
                    }
                    "copies" => {
                        if let Ok(copies) = value.parse::<usize>() {
                            query.copies_in_deck = Some(copies);
                        }
                        continue;
                    }
                    _ => {}
                }
            }

            query.text_terms.push(term);
        }

        query
    }

    pub fn matches(&self, card: &StoryCardDefinition, context: DeckSearchCardContext) -> bool {
        if self.require_owned && context.owned_count == 0 {
            return false;
        }
        if self.require_missing && context.available_count > 0 {
            return false;
        }
        if let Some(expected_speed) = self.speed {
            if card.speed != expected_speed {
                return false;
            }
        }
        if let Some(expected_alignment) = self.alignment {
            if card.alignment != expected_alignment {
                return false;
            }
        }
        if let Some(expected_type) = self.card_type.as_deref() {
            if !card.card_type.eq_ignore_ascii_case(expected_type) {
                return false;
            }
        }
        if let Some(expected_copies) = self.copies_in_deck {
            if context.copies_in_deck != expected_copies {
                return false;
            }
        }

        let searchable_text = build_searchable_text(card, context);
        self.text_terms
            .iter()
            .all(|term| searchable_text.contains(term))
    }
}

fn build_searchable_text(card: &StoryCardDefinition, context: DeckSearchCardContext) -> String {
    let mut parts = vec![
        card.name.to_ascii_lowercase(),
        card.id.to_ascii_lowercase(),
        card.card_type.to_ascii_lowercase(),
        speed_search_text(card.speed),
        alignment_search_text(card.alignment),
    ];

    if context.owned_count > 0 {
        parts.push("owned".to_owned());
    }
    if context.available_count == 0 {
        parts.push("missing".to_owned());
    }

    for effect in &card.effects {
        parts.push(effect_search_text(effect));
    }

    parts.join(" ")
}

fn effect_search_text(effect: &CardEffect) -> String {
    match effect {
        CardEffect::GainMainRadiance { .. } => "gain main radiance".to_owned(),
        CardEffect::GainRevealedSupportRadiance { .. } => {
            "gain revealed support radiance".to_owned()
        }
        CardEffect::ReduceOpponentMainRadiance { .. } => "reduce opponent main radiance".to_owned(),
        CardEffect::GainPrimeDread { .. } => "gain prime dread".to_owned(),
        CardEffect::GainRevealedSupportDread { .. } => "gain revealed support dread".to_owned(),
        CardEffect::ReduceOpponentPrimeDread { .. } => "reduce opponent prime dread".to_owned(),
        CardEffect::GainMainPowerThisEncounter { .. } => "gain main power encounter".to_owned(),
        CardEffect::GainMainPowerNextEncounter { .. } => {
            "gain main power next encounter".to_owned()
        }
        CardEffect::ReduceOpponentMainPowerThisEncounter { .. } => {
            "reduce opponent main power encounter".to_owned()
        }
        CardEffect::GainPrimePowerThisEncounter { .. } => "gain prime power encounter".to_owned(),
        CardEffect::GainRevealedSupportPowerThisEncounter { .. } => {
            "gain revealed support power encounter".to_owned()
        }
        CardEffect::GainFirstRevealedSupportRadiance { .. } => {
            "gain first revealed support radiance".to_owned()
        }
        CardEffect::ExhaustFirstRevealedOpponentSupport => {
            "exhaust first revealed opponent support".to_owned()
        }
        CardEffect::RevealFirstHiddenOwnSupport => "reveal first hidden own support".to_owned(),
    }
}

fn speed_search_text(speed: CardSpeed) -> String {
    match speed {
        CardSpeed::DailyLife => "daily daily_life".to_owned(),
        CardSpeed::Reaction => "reaction".to_owned(),
        CardSpeed::Encounter => "encounter".to_owned(),
    }
}

fn alignment_search_text(alignment: CardAlignment) -> String {
    match alignment {
        CardAlignment::MagicalGirl => "magical_girl magical girl mg".to_owned(),
        CardAlignment::Baddie => "baddie".to_owned(),
        CardAlignment::Neutral => "neutral".to_owned(),
    }
}

fn parse_speed(value: &str) -> Option<CardSpeed> {
    match value {
        "daily" | "daily_life" | "dailylife" => Some(CardSpeed::DailyLife),
        "reaction" => Some(CardSpeed::Reaction),
        "encounter" => Some(CardSpeed::Encounter),
        _ => None,
    }
}

fn parse_alignment(value: &str) -> Option<CardAlignment> {
    match value {
        "mg" | "magical_girl" | "magicalgirl" => Some(CardAlignment::MagicalGirl),
        "baddie" => Some(CardAlignment::Baddie),
        "neutral" => Some(CardAlignment::Neutral),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
