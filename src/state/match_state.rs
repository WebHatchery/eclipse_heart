//! Deterministic runtime state for a match.

#[path = "match_state_access.rs"]
mod access;
#[path = "match_state_round_flow.rs"]
mod round_flow;
#[path = "match_state_setup.rs"]
mod setup;

use std::collections::HashMap;

use serde::Serialize;

use crate::data::{
    CardSpeed, CharacterDefinition, GameContent, MatchRules, StarterLoadout, StoryCardDefinition,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchPhase {
    DailyLife,
    Encounter,
    FinalClimax,
    Finished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterStage {
    Base,
    Transformed,
    Radiant,
    Awakened,
    Catastrophe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    Radiance,
    Dread,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PlayerId {
    PlayerA,
    PlayerB,
}

#[derive(Clone, Debug)]
pub enum StackItemKind {
    PlayCard {
        card_id: String,
        card_name: String,
    },
    RevealSupport {
        is_magical_girl_side: bool,
        support_index: usize,
    },
}

#[derive(Clone, Debug)]
pub struct StackItem {
    pub player: PlayerId,
    pub is_reaction: bool,
    pub resolves_in_phase: MatchPhase,
    pub kind: StackItemKind,
}

#[derive(Clone, Debug)]
pub struct ReactionState {
    pub priority_player: PlayerId,
    pub passes_in_row: u8,
}

#[derive(Clone, Debug)]
pub struct CharacterRuntimeState {
    pub name: String,
    pub stage: CharacterStage,
    pub base_power: i32,
    pub transformed_power: i32,
    pub final_power: i32,
    pub temporary_power_bonus: i32,
    pub next_encounter_power_bonus: i32,
    pub failed_final_climax_power_bonus: i32,
    pub radiance: i32,
    pub dread: i32,
    pub first_threshold: i32,
    pub second_threshold: i32,
    pub exhausted_until_next_encounter: bool,
    pub abilities_blocked_until_next_encounter: bool,
}

impl CharacterRuntimeState {
    pub fn from_definition(definition: &CharacterDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            stage: CharacterStage::Base,
            base_power: definition.base_power,
            transformed_power: definition.transformed_power,
            final_power: definition.final_power,
            temporary_power_bonus: 0,
            next_encounter_power_bonus: 0,
            failed_final_climax_power_bonus: 0,
            radiance: 0,
            dread: 0,
            first_threshold: definition.first_threshold,
            second_threshold: definition.second_threshold,
            exhausted_until_next_encounter: false,
            abilities_blocked_until_next_encounter: false,
        }
    }

    pub fn current_power(&self) -> i32 {
        if self.exhausted_until_next_encounter {
            return 0;
        }

        let base = match self.stage {
            CharacterStage::Base => self.base_power,
            CharacterStage::Transformed | CharacterStage::Awakened => self.transformed_power,
            CharacterStage::Radiant | CharacterStage::Catastrophe => self.final_power,
        };

        base + self.temporary_power_bonus + self.failed_final_climax_power_bonus
    }

    pub fn gain(&mut self, resource: ResourceKind, amount: i32) {
        match resource {
            ResourceKind::Radiance => {
                self.radiance = (self.radiance + amount).max(0);
                self.check_upgrade();
            }
            ResourceKind::Dread => {
                self.dread = (self.dread + amount).max(0);
                self.check_upgrade();
            }
        }
    }

    fn check_upgrade(&mut self) {
        match self.stage {
            CharacterStage::Base => {
                if self.radiance >= self.first_threshold {
                    self.stage = CharacterStage::Transformed;
                    self.radiance = 0;
                }
                if self.dread >= self.first_threshold {
                    self.stage = CharacterStage::Awakened;
                    self.dread = 0;
                }
            }
            CharacterStage::Transformed => {
                if self.radiance >= self.second_threshold {
                    self.stage = CharacterStage::Radiant;
                    self.radiance = 0;
                }
            }
            CharacterStage::Awakened => {
                if self.dread >= self.second_threshold {
                    self.stage = CharacterStage::Catastrophe;
                    self.dread = 0;
                }
            }
            CharacterStage::Radiant | CharacterStage::Catastrophe => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct SupportState {
    pub runtime: CharacterRuntimeState,
    pub revealed: bool,
}

impl SupportState {
    pub fn gain(&mut self, resource: ResourceKind, amount: i32) {
        self.runtime.gain(resource, amount);
    }
}

#[derive(Clone, Debug)]
pub struct SideState {
    pub main_character_id: String,
    pub main: CharacterRuntimeState,
    pub supports: Vec<SupportState>,
}

impl SideState {
    pub fn total_power(&self) -> i32 {
        let mut power = self.main.current_power();

        for support in &self.supports {
            if support.revealed && !support.runtime.exhausted_until_next_encounter {
                power += support.runtime.current_power();
            }
        }

        power
    }
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub magical_girls: SideState,
    pub baddies: SideState,
    pub deck: Vec<String>,
    pub hand: Vec<String>,
    pub discard: Vec<String>,
    pub encounter_card_played: bool,
    pub supports_revealed_this_round: u8,
}

#[derive(Clone, Debug)]
pub struct MatchState {
    pub rules: MatchRules,
    pub story_cards: HashMap<String, StoryCardDefinition>,
    pub phase: MatchPhase,
    pub round: u32,
    pub final_climax_active: bool,
    pub prime_baddie_defeated: bool,
    pub defeated_prime_owner: Option<PlayerId>,
    pub winner: Option<PlayerId>,
    pub player_a: PlayerState,
    pub player_b: PlayerState,
    pub active_player: PlayerId,
    pub priority_player: PlayerId,
    pub phase_passes: u8,
    pub last_played_card_name: Option<String>,
    pub last_outcome: Option<crate::engine::EncounterOutcome>,
    pub reaction_stack: Vec<StackItem>,
    pub reaction_state: Option<ReactionState>,
    pub event_log: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct MatchSetup {
    pub player_a_mg_main_index: usize,
    pub player_a_mg_support_pair_index: usize,
    pub player_a_baddie_main_index: usize,
    pub player_a_baddie_support_pair_index: usize,
    pub player_a_support_deck_id: Option<String>,
    pub player_b_mg_main_index: usize,
    pub player_b_mg_support_pair_index: usize,
    pub player_b_baddie_main_index: usize,
    pub player_b_baddie_support_pair_index: usize,
    pub player_b_support_deck_id: Option<String>,
}

pub fn opposing(player: PlayerId) -> PlayerId {
    match player {
        PlayerId::PlayerA => PlayerId::PlayerB,
        PlayerId::PlayerB => PlayerId::PlayerA,
    }
}

#[cfg(test)]
mod tests;
