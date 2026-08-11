//! Persistent campaign run data.

use serde::{Deserialize, Serialize};

use crate::data::CampaignDefinition;

use super::{timestamp::current_unix_timestamp, DeckPreset};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CampaignRunStatus {
    InProgress,
    Won,
    Lost,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CampaignBattleRecord {
    pub node_id: String,
    pub encounter_id: String,
    pub won: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignRunSave {
    pub id: String,
    pub name: String,
    pub campaign_id: String,
    pub status: CampaignRunStatus,
    pub current_node_id: String,
    #[serde(default)]
    pub completed_node_ids: Vec<String>,
    pub player_deck: DeckPreset,
    #[serde(default)]
    pub selected_magical_girl_support_ids: Vec<String>,
    #[serde(default)]
    pub battle_history: Vec<CampaignBattleRecord>,
    pub created_at_unix: i64,
    pub updated_at_unix: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignSaveBundle {
    pub version: u32,
    #[serde(default)]
    pub runs: Vec<CampaignRunSave>,
    #[serde(default)]
    pub selected_run_id: Option<String>,
}

impl Default for CampaignSaveBundle {
    fn default() -> Self {
        Self {
            version: 2,
            runs: Vec::new(),
            selected_run_id: None,
        }
    }
}

impl CampaignSaveBundle {
    pub fn create_run(
        &mut self,
        campaign: &CampaignDefinition,
        source_deck: &DeckPreset,
        default_name: &str,
    ) -> Option<String> {
        let first_node = campaign.first_node()?;
        let run_id = format!("campaign_run_{}", self.runs.len() + 1);
        let now = current_unix_timestamp();
        let run_name = if source_deck.name.trim().is_empty() {
            default_name.to_owned()
        } else {
            format!("{} Run", source_deck.name)
        };
        self.runs.push(CampaignRunSave {
            id: run_id.clone(),
            name: run_name,
            campaign_id: campaign.id.clone(),
            status: CampaignRunStatus::InProgress,
            current_node_id: first_node.id.clone(),
            completed_node_ids: Vec::new(),
            player_deck: source_deck.clone(),
            selected_magical_girl_support_ids: Vec::new(),
            battle_history: Vec::new(),
            created_at_unix: now,
            updated_at_unix: now,
        });
        self.selected_run_id = Some(run_id.clone());
        Some(run_id)
    }

    pub fn selected_run(&self) -> Option<&CampaignRunSave> {
        let selected_id = self.selected_run_id.as_deref().or_else(|| {
            self.runs
                .iter()
                .rev()
                .find(|run| run.status == CampaignRunStatus::InProgress)
                .or_else(|| self.runs.last())
                .map(|run| run.id.as_str())
        })?;
        self.runs.iter().find(|run| run.id == selected_id)
    }

    pub fn selected_run_mut(&mut self) -> Option<&mut CampaignRunSave> {
        let selected_id = self.selected_run_id.clone().or_else(|| {
            self.runs
                .iter()
                .rev()
                .find(|run| run.status == CampaignRunStatus::InProgress)
                .or_else(|| self.runs.last())
                .map(|run| run.id.clone())
        })?;
        self.runs.iter_mut().find(|run| run.id == selected_id)
    }

    pub fn select_run(&mut self, run_id: &str) -> bool {
        if self.runs.iter().any(|run| run.id == run_id) {
            self.selected_run_id = Some(run_id.to_owned());
            return true;
        }
        false
    }

    pub fn selected_run_is_in_progress(&self) -> bool {
        self.selected_run()
            .map(|run| run.status == CampaignRunStatus::InProgress)
            .unwrap_or(false)
    }

    pub fn selected_run_has_valid_support_pair(&self) -> bool {
        self.selected_run()
            .map(|run| run.selected_magical_girl_support_ids.len() == 2)
            .unwrap_or(false)
    }

    pub fn update_selected_magical_girl_supports(&mut self, support_ids: &[String]) -> bool {
        let Some(run) = self.selected_run_mut() else {
            return false;
        };
        if support_ids.len() != 2 {
            return false;
        }
        run.selected_magical_girl_support_ids = support_ids.to_vec();
        run.updated_at_unix = current_unix_timestamp();
        true
    }

    pub fn toggle_selected_magical_girl_support(&mut self, character_id: &str) -> bool {
        let Some(run) = self.selected_run_mut() else {
            return false;
        };
        if run
            .player_deck
            .magical_girl_roster
            .first()
            .is_some_and(|main_id| main_id == character_id)
        {
            return false;
        }
        if !run
            .player_deck
            .magical_girl_roster
            .iter()
            .any(|entry| entry == character_id)
        {
            return false;
        }

        if let Some(index) = run
            .selected_magical_girl_support_ids
            .iter()
            .position(|entry| entry == character_id)
        {
            run.selected_magical_girl_support_ids.remove(index);
        } else if run.selected_magical_girl_support_ids.len() < 2 {
            run.selected_magical_girl_support_ids
                .push(character_id.to_owned());
        } else {
            return false;
        }

        run.updated_at_unix = current_unix_timestamp();
        true
    }

    pub fn run_by_id(&self, run_id: &str) -> Option<&CampaignRunSave> {
        self.runs.iter().find(|run| run.id == run_id)
    }

    fn run_by_id_mut(&mut self, run_id: &str) -> Option<&mut CampaignRunSave> {
        self.runs.iter_mut().find(|run| run.id == run_id)
    }

    pub fn abandon_selected_run(&mut self) -> bool {
        let Some(run) = self.selected_run_mut() else {
            return false;
        };
        run.status = CampaignRunStatus::Lost;
        run.updated_at_unix = current_unix_timestamp();
        true
    }

    pub fn record_victory_for_run(
        &mut self,
        run_id: &str,
        campaign: &CampaignDefinition,
        node_id: &str,
        reward_card_id: Option<&str>,
    ) -> Option<bool> {
        let run = self.run_by_id_mut(run_id)?;
        let node = campaign.node(node_id)?;
        let encounter = campaign.encounter(&node.encounter_id)?;
        run.completed_node_ids.push(node.id.clone());
        if let Some(card_id) = reward_card_id {
            run.player_deck.story_cards.push(card_id.to_owned());
            run.player_deck.recent_story_cards.push(card_id.to_owned());
            if run.player_deck.recent_story_cards.len() > 5 {
                run.player_deck.recent_story_cards.remove(0);
            }
        }
        run.battle_history.push(CampaignBattleRecord {
            node_id: node.id.clone(),
            encounter_id: encounter.id.clone(),
            won: true,
        });
        run.updated_at_unix = current_unix_timestamp();
        if let Some(next_node_id) = node.next_node_ids.first() {
            run.current_node_id = next_node_id.clone();
            return Some(false);
        }
        run.status = CampaignRunStatus::Won;
        Some(true)
    }

    pub fn record_defeat_for_run(
        &mut self,
        run_id: &str,
        campaign: &CampaignDefinition,
        node_id: &str,
    ) -> bool {
        let Some(run) = self.run_by_id_mut(run_id) else {
            return false;
        };
        let Some(node) = campaign.node(node_id) else {
            return false;
        };
        let Some(encounter) = campaign.encounter(&node.encounter_id) else {
            return false;
        };
        run.battle_history.push(CampaignBattleRecord {
            node_id: node.id.clone(),
            encounter_id: encounter.id.clone(),
            won: false,
        });
        run.status = CampaignRunStatus::Lost;
        run.updated_at_unix = current_unix_timestamp();
        true
    }
}

#[cfg(test)]
mod tests;
