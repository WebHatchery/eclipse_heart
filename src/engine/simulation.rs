//! Headless AI-vs-AI simulation helpers for balance testing.

use serde::Serialize;

use crate::data::GameContent;
use crate::state::{opposing, CharacterStage, MatchPhase, MatchState, PlayerId};

use super::{AiController, MatchAction, MatchEngine};

#[derive(Clone, Debug, Serialize)]
pub struct EncounterReport {
    pub round: u32,
    pub attacker: PlayerId,
    pub defender: PlayerId,
    pub was_final_climax: bool,
    pub magical_girl_power: i32,
    pub baddie_power: i32,
    pub outcome: String,
    pub attacker_main_stage_before: String,
    pub attacker_main_radiance_before: i32,
    pub attacker_failed_final_climax_bonus_before: i32,
    pub attacker_failed_final_climax_bonus_after: i32,
    pub defender_main_magical_girl_stage_before: String,
    pub defender_prime_baddie_stage_before: String,
    pub defender_prime_baddie_dread_before: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActionReport {
    pub sequence: usize,
    pub round: u32,
    pub phase_before: String,
    pub actor: PlayerId,
    pub action_type: String,
    pub card_name: Option<String>,
    pub is_reaction: bool,
    pub events: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MatchReport {
    pub player_a_loadout: String,
    pub player_b_loadout: String,
    pub starting_player: PlayerId,
    pub winner: Option<PlayerId>,
    pub rounds_completed: u32,
    pub non_final_climax_rounds: u32,
    pub final_climax_declared: bool,
    pub final_climax_started_round: Option<u32>,
    pub failed_final_climax_attempts: u32,
    pub player_a_first_radiant_round: Option<u32>,
    pub player_b_first_radiant_round: Option<u32>,
    pub player_a_prime_first_catastrophe_round: Option<u32>,
    pub player_b_prime_first_catastrophe_round: Option<u32>,
    pub reached_round_cap: bool,
    pub reached_action_cap: bool,
    pub actions: Vec<ActionReport>,
    pub encounters: Vec<EncounterReport>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BatchReport {
    pub matches_run: usize,
    pub player_a_wins: usize,
    pub player_b_wins: usize,
    pub unfinished_matches: usize,
    pub average_rounds_completed: f32,
    pub average_failed_final_climax_attempts: f32,
    pub reports: Vec<MatchReport>,
}

pub struct SimulationRunner;

impl SimulationRunner {
    pub fn run_starter_series(content: &GameContent, round_cap: u32) -> BatchReport {
        let mut reports = Vec::new();

        for player_a in &content.starter_loadouts {
            for player_b in &content.starter_loadouts {
                for starting_player in [PlayerId::PlayerA, PlayerId::PlayerB] {
                    reports.push(Self::run_match(
                        content,
                        player_a,
                        player_b,
                        starting_player,
                        round_cap,
                    ));
                }
            }
        }

        summarize_reports(reports)
    }

    pub fn run_match(
        content: &GameContent,
        player_a: &crate::data::StarterLoadout,
        player_b: &crate::data::StarterLoadout,
        starting_player: PlayerId,
        round_cap: u32,
    ) -> MatchReport {
        let mut state =
            MatchState::from_starter_loadouts(content, player_a, player_b, starting_player);
        let mut failed_final_climax_attempts = 0;
        let mut final_climax_started_round = None;
        let mut actions = Vec::new();
        let mut encounters = Vec::new();
        let mut actions_taken = 0usize;
        let mut event_cursor = 0usize;
        let action_cap = round_cap as usize * 200;

        while state.phase != MatchPhase::Finished
            && state.round <= round_cap
            && actions_taken < action_cap
        {
            if action_will_declare_final_climax(&state) && final_climax_started_round.is_none() {
                final_climax_started_round = Some(state.round);
            }

            let action = next_simulation_action(&state);
            let encounter_snapshot = action
                .as_ref()
                .and_then(|next_action| encounter_snapshot_before_action(&state, next_action));
            let was_failed_final_climax_attempt = action.as_ref().is_some_and(|next_action| {
                action_will_resolve_encounter(&state, next_action)
                    && state.phase == MatchPhase::FinalClimax
            });

            let Some(action) = action else {
                break;
            };
            let action_context = action_context_before(&state, &action, actions_taken + 1);

            MatchEngine::apply_action(&mut state, action);
            actions_taken += 1;
            actions.push(action_context.finish(&state, &mut event_cursor));

            if let Some(snapshot) = encounter_snapshot {
                encounters.push(snapshot.finish(&state));
            }

            if was_failed_final_climax_attempt && state.phase != MatchPhase::Finished {
                failed_final_climax_attempts += 1;
            }
        }

        let milestones = derive_milestones(&encounters, final_climax_started_round, state.round);

        MatchReport {
            player_a_loadout: player_a.name.clone(),
            player_b_loadout: player_b.name.clone(),
            starting_player,
            winner: state.winner,
            rounds_completed: state.round,
            non_final_climax_rounds: milestones.non_final_climax_rounds,
            final_climax_declared: state.final_climax_active,
            final_climax_started_round,
            failed_final_climax_attempts,
            player_a_first_radiant_round: milestones.player_a_first_radiant_round,
            player_b_first_radiant_round: milestones.player_b_first_radiant_round,
            player_a_prime_first_catastrophe_round: milestones
                .player_a_prime_first_catastrophe_round,
            player_b_prime_first_catastrophe_round: milestones
                .player_b_prime_first_catastrophe_round,
            reached_round_cap: state.phase != MatchPhase::Finished && state.round > round_cap,
            reached_action_cap: state.phase != MatchPhase::Finished && actions_taken >= action_cap,
            actions,
            encounters,
        }
    }
}

fn next_simulation_action(state: &MatchState) -> Option<MatchAction> {
    if let Some(priority_player) = state.reaction_priority_player() {
        return AiController::choose_action_for(state, priority_player);
    }

    let proactive_player = state.proactive_priority_player()?;
    AiController::choose_action_for(state, proactive_player)
}

fn action_will_declare_final_climax(state: &MatchState) -> bool {
    matches!(
        next_simulation_action(state),
        Some(MatchAction::DeclareFinalClimax)
    )
}

struct ActionContext {
    sequence: usize,
    round: u32,
    phase_before: MatchPhase,
    actor: PlayerId,
    action_type: &'static str,
    card_name: Option<String>,
    is_reaction: bool,
}

impl ActionContext {
    fn finish(self, state: &MatchState, event_cursor: &mut usize) -> ActionReport {
        ActionReport {
            sequence: self.sequence,
            round: self.round,
            phase_before: format!("{:?}", self.phase_before),
            actor: self.actor,
            action_type: self.action_type.to_string(),
            card_name: self.card_name,
            is_reaction: self.is_reaction,
            events: take_new_events(&state.event_log, event_cursor),
        }
    }
}

fn action_context_before(
    state: &MatchState,
    action: &MatchAction,
    sequence: usize,
) -> ActionContext {
    match action {
        MatchAction::ResolveEncounter => ActionContext {
            sequence,
            round: state.round,
            phase_before: state.phase,
            actor: state.active_player,
            action_type: "ResolveEncounter",
            card_name: None,
            is_reaction: false,
        },
        MatchAction::DeclareFinalClimax => ActionContext {
            sequence,
            round: state.round,
            phase_before: state.phase,
            actor: state.active_player,
            action_type: "DeclareFinalClimax",
            card_name: None,
            is_reaction: false,
        },
        MatchAction::PlayCardFromHand { player, hand_index } => ActionContext {
            sequence,
            round: state.round,
            phase_before: state.phase,
            actor: *player,
            action_type: "PlayCardFromHand",
            card_name: state
                .card_in_hand(*player, *hand_index)
                .map(|card| card.name.clone()),
            is_reaction: state.reaction_priority_player() == Some(*player),
        },
        MatchAction::RevealFirstHiddenSupport {
            player,
            is_magical_girl_side,
        } => ActionContext {
            sequence,
            round: state.round,
            phase_before: state.phase,
            actor: *player,
            action_type: if *is_magical_girl_side {
                "RevealHiddenMagicalGirlSupport"
            } else {
                "RevealHiddenBaddieSupport"
            },
            card_name: None,
            is_reaction: state.reaction_priority_player() == Some(*player),
        },
        MatchAction::PassDailyLife { player } => ActionContext {
            sequence,
            round: state.round,
            phase_before: state.phase,
            actor: *player,
            action_type: "PassDailyLife",
            card_name: None,
            is_reaction: false,
        },
        MatchAction::PassEncounter { player } => ActionContext {
            sequence,
            round: state.round,
            phase_before: state.phase,
            actor: *player,
            action_type: "PassEncounter",
            card_name: None,
            is_reaction: false,
        },
        MatchAction::PassReaction { player } => ActionContext {
            sequence,
            round: state.round,
            phase_before: state.phase,
            actor: *player,
            action_type: "PassReaction",
            card_name: None,
            is_reaction: true,
        },
    }
}

fn take_new_events(current_events: &[String], event_cursor: &mut usize) -> Vec<String> {
    let start = (*event_cursor).min(current_events.len());
    let events = current_events[start..].to_vec();
    *event_cursor = current_events.len();
    events
}

struct EncounterSnapshot {
    round: u32,
    attacker: PlayerId,
    defender: PlayerId,
    was_final_climax: bool,
    magical_girl_power: i32,
    baddie_power: i32,
    attacker_main_stage_before: CharacterStage,
    attacker_main_radiance_before: i32,
    attacker_failed_final_climax_bonus_before: i32,
    defender_main_magical_girl_stage_before: CharacterStage,
    defender_prime_baddie_stage_before: CharacterStage,
    defender_prime_baddie_dread_before: i32,
}

impl EncounterSnapshot {
    fn finish(self, state: &MatchState) -> EncounterReport {
        EncounterReport {
            round: self.round,
            attacker: self.attacker,
            defender: self.defender,
            was_final_climax: self.was_final_climax,
            magical_girl_power: self.magical_girl_power,
            baddie_power: self.baddie_power,
            outcome: format!(
                "{:?}",
                state.last_outcome.unwrap_or(super::EncounterOutcome::Tie)
            ),
            attacker_main_stage_before: format!("{:?}", self.attacker_main_stage_before),
            attacker_main_radiance_before: self.attacker_main_radiance_before,
            attacker_failed_final_climax_bonus_before: self
                .attacker_failed_final_climax_bonus_before,
            attacker_failed_final_climax_bonus_after: state
                .player_for(self.attacker)
                .magical_girls
                .main
                .failed_final_climax_power_bonus,
            defender_main_magical_girl_stage_before: format!(
                "{:?}",
                self.defender_main_magical_girl_stage_before
            ),
            defender_prime_baddie_stage_before: format!(
                "{:?}",
                self.defender_prime_baddie_stage_before
            ),
            defender_prime_baddie_dread_before: self.defender_prime_baddie_dread_before,
        }
    }
}

fn encounter_snapshot_before_action(
    state: &MatchState,
    action: &MatchAction,
) -> Option<EncounterSnapshot> {
    if !action_will_resolve_encounter(state, action) {
        return None;
    }

    if state.phase != MatchPhase::Encounter && state.phase != MatchPhase::FinalClimax {
        return None;
    }

    let attacker = state.active_player;
    let defender = opposing(attacker);
    let attacker_main = &state.player_for(attacker).magical_girls.main;
    let defender_main_magical_girl = &state.player_for(defender).magical_girls.main;
    let defender_prime_baddie = &state.player_for(defender).baddies.main;

    Some(EncounterSnapshot {
        round: state.round,
        attacker,
        defender,
        was_final_climax: state.phase == MatchPhase::FinalClimax,
        magical_girl_power: state.active_magical_girls().total_power(),
        baddie_power: state.defending_baddies().total_power(),
        attacker_main_stage_before: attacker_main.stage,
        attacker_main_radiance_before: attacker_main.radiance,
        attacker_failed_final_climax_bonus_before: attacker_main.failed_final_climax_power_bonus,
        defender_main_magical_girl_stage_before: defender_main_magical_girl.stage,
        defender_prime_baddie_stage_before: defender_prime_baddie.stage,
        defender_prime_baddie_dread_before: defender_prime_baddie.dread,
    })
}

fn action_will_resolve_encounter(state: &MatchState, action: &MatchAction) -> bool {
    if state.phase != MatchPhase::Encounter && state.phase != MatchPhase::FinalClimax {
        return false;
    }

    match action {
        MatchAction::ResolveEncounter => true,
        MatchAction::PassEncounter { player } => {
            state.reaction_state.is_none()
                && state.priority_player == *player
                && state.phase_passes >= 1
        }
        _ => false,
    }
}

struct MatchMilestones {
    non_final_climax_rounds: u32,
    player_a_first_radiant_round: Option<u32>,
    player_b_first_radiant_round: Option<u32>,
    player_a_prime_first_catastrophe_round: Option<u32>,
    player_b_prime_first_catastrophe_round: Option<u32>,
}

fn derive_milestones(
    encounters: &[EncounterReport],
    final_climax_started_round: Option<u32>,
    rounds_completed: u32,
) -> MatchMilestones {
    MatchMilestones {
        non_final_climax_rounds: final_climax_started_round
            .map(|round| round.saturating_sub(1))
            .unwrap_or(rounds_completed),
        player_a_first_radiant_round: first_round_with_magical_girl_stage(
            encounters,
            PlayerId::PlayerA,
            "Radiant",
        ),
        player_b_first_radiant_round: first_round_with_magical_girl_stage(
            encounters,
            PlayerId::PlayerB,
            "Radiant",
        ),
        player_a_prime_first_catastrophe_round: first_round_with_prime_baddie_stage(
            encounters,
            PlayerId::PlayerA,
            "Catastrophe",
        ),
        player_b_prime_first_catastrophe_round: first_round_with_prime_baddie_stage(
            encounters,
            PlayerId::PlayerB,
            "Catastrophe",
        ),
    }
}

fn first_round_with_magical_girl_stage(
    encounters: &[EncounterReport],
    player: PlayerId,
    target_stage: &str,
) -> Option<u32> {
    encounters.iter().find_map(|encounter| {
        if (encounter.attacker == player && encounter.attacker_main_stage_before == target_stage)
            || (encounter.defender == player
                && encounter.defender_main_magical_girl_stage_before == target_stage)
        {
            Some(encounter.round)
        } else {
            None
        }
    })
}

fn first_round_with_prime_baddie_stage(
    encounters: &[EncounterReport],
    player: PlayerId,
    target_stage: &str,
) -> Option<u32> {
    encounters.iter().find_map(|encounter| {
        if encounter.defender == player
            && encounter.defender_prime_baddie_stage_before == target_stage
        {
            Some(encounter.round)
        } else {
            None
        }
    })
}

fn summarize_reports(reports: Vec<MatchReport>) -> BatchReport {
    let matches_run = reports.len();
    let player_a_wins = reports
        .iter()
        .filter(|report| report.winner == Some(PlayerId::PlayerA))
        .count();
    let player_b_wins = reports
        .iter()
        .filter(|report| report.winner == Some(PlayerId::PlayerB))
        .count();
    let unfinished_matches = reports
        .iter()
        .filter(|report| report.reached_round_cap || report.reached_action_cap)
        .count();
    let average_rounds_completed = if matches_run == 0 {
        0.0
    } else {
        reports
            .iter()
            .map(|report| report.rounds_completed as f32)
            .sum::<f32>()
            / matches_run as f32
    };
    let average_failed_final_climax_attempts = if matches_run == 0 {
        0.0
    } else {
        reports
            .iter()
            .map(|report| report.failed_final_climax_attempts as f32)
            .sum::<f32>()
            / matches_run as f32
    };

    BatchReport {
        matches_run,
        player_a_wins,
        player_b_wins,
        unfinished_matches,
        average_rounds_completed,
        average_failed_final_climax_attempts,
        reports,
    }
}

#[cfg(test)]
mod tests;
