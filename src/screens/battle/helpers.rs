use macroquad::prelude::*;

use crate::state::{
    opposing, AppState, BattleContext, MatchPhase, MatchState, PlayerId, SupportState,
};
use crate::ui::core::{draw_button_frame, TEXT_MUTED};
use crate::ui::layout::UiLayout;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::{draw_ui_text, wrap_text};

pub(super) fn can_reveal_side(
    match_state: &MatchState,
    player: PlayerId,
    is_magical_girl_side: bool,
) -> bool {
    ((match_state.reaction_priority_player() == Some(player))
        || (match_state.reaction_state.is_none()
            && match_state.proactive_priority_player() == Some(player)))
        && match_state.can_reveal_support(player, is_magical_girl_side)
}

pub(super) fn format_supports(state: &AppState, supports: &[SupportState]) -> String {
    let labels = supports
        .iter()
        .enumerate()
        .map(|(index, support)| {
            if support.revealed {
                format!("S{} {}", index + 1, support.runtime.name)
            } else {
                format!(
                    "S{} {}",
                    index + 1,
                    state.ui_text.get("battle_hidden_support_short")
                )
            }
        })
        .collect::<Vec<_>>();

    format!(
        "{}: {}",
        state.ui_text.get("battle_supports_label"),
        labels.join(" | ")
    )
}

pub(super) fn player_status<'a>(
    state: &'a AppState,
    match_state: &'a MatchState,
    player: PlayerId,
) -> &'a str {
    if match_state.phase == MatchPhase::Finished {
        state.ui_text.get("battle_finished_label")
    } else if match_state.active_player == player {
        state.ui_text.get("battle_attacking_label")
    } else if opposing(match_state.active_player) == player {
        state.ui_text.get("battle_defending_label")
    } else {
        state.ui_text.get("battle_idle_label")
    }
}

pub(super) fn winner_label(state: &AppState, winner: Option<PlayerId>) -> &str {
    match winner {
        Some(PlayerId::PlayerA) => state.ui_text.get("battle_result_player_a"),
        Some(PlayerId::PlayerB) => state.ui_text.get("battle_result_player_b"),
        None => state.ui_text.get("battle_result_unknown"),
    }
}

pub(super) fn campaign_winner_label(state: &AppState, winner: Option<PlayerId>) -> &str {
    match winner {
        Some(PlayerId::PlayerA) => state.ui_text.get("campaign_battle_player_wins"),
        Some(PlayerId::PlayerB) => state.ui_text.get("campaign_battle_enemy_wins"),
        None => state.ui_text.get("battle_result_unknown"),
    }
}

pub(super) fn hand_card_status_label(state: &AppState, enabled: bool) -> &str {
    if enabled {
        state.ui_text.get("battle_card_ready")
    } else {
        state.ui_text.get("battle_card_hold")
    }
}

pub(super) fn battle_action_hint<'a>(
    state: &'a AppState,
    match_state: &'a MatchState,
    player: PlayerId,
) -> &'a str {
    if match_state.reaction_priority_player() == Some(player) {
        state.ui_text.get("battle_hint_reaction")
    } else if match_state.proactive_priority_player() == Some(player)
        && match_state.phase == MatchPhase::DailyLife
    {
        state.ui_text.get("battle_hint_daily_life")
    } else if match_state.proactive_priority_player() == Some(player)
        && (match_state.phase == MatchPhase::Encounter
            || match_state.phase == MatchPhase::FinalClimax)
    {
        state.ui_text.get("battle_hint_encounter")
    } else if match_state.phase == MatchPhase::Finished {
        state.ui_text.get("battle_hint_finished")
    } else {
        state.ui_text.get("battle_hint_waiting")
    }
}

pub(super) fn campaign_encounter_name(state: &AppState) -> String {
    let BattleContext::Campaign { node_id, .. } = &state.battle_context else {
        return String::new();
    };
    state
        .content
        .campaign
        .node(node_id)
        .and_then(|node| state.content.campaign.encounter(&node.encounter_id))
        .map(|encounter| encounter.name.clone())
        .unwrap_or_else(|| state.ui_text.get("campaign_missing_encounter").to_owned())
}

pub(super) fn wrap_event_lines(events: &[String], max_width: f32, font_size: f32) -> Vec<String> {
    events
        .iter()
        .flat_map(|event| wrap_text(event, max_width, font_size))
        .collect()
}

pub(super) fn hand_card_rects(card_count: usize) -> Vec<Rect> {
    let ui = UiLayout::current();
    let start_x = ui.x(548.0);
    let y = ui.y(986.0);
    let card_width = ui.w(250.0);
    let card_height = ui.h(352.0);
    let preferred_gap = ui.w(18.0);
    let available_width = ui.w(1920.0);

    if card_count == 0 {
        return Vec::new();
    }

    let step = if card_count == 1 {
        0.0
    } else {
        let preferred_total =
            card_width * card_count as f32 + preferred_gap * (card_count.saturating_sub(1)) as f32;
        if preferred_total <= available_width {
            card_width + preferred_gap
        } else {
            (available_width - card_width) / (card_count.saturating_sub(1)) as f32
        }
    };

    (0..card_count)
        .map(|index| Rect::new(start_x + index as f32 * step, y, card_width, card_height))
        .collect()
}

pub(super) fn draw_stat_chip(rect: Rect, label: &str, value: &str, accent: Color) {
    let ui = UiLayout::current();
    draw_button_frame(
        rect,
        Color::new(0.016, 0.024, 0.066, 0.92),
        with_alpha(accent, 0.72),
        accent,
    );
    draw_ui_text(
        label,
        rect.x + ui.w(14.0),
        rect.y + ui.h(20.0),
        ui.font(16.0),
        TEXT_MUTED,
    );
    draw_ui_text(
        value,
        rect.x + ui.w(14.0),
        rect.y + ui.h(43.0),
        ui.font(23.0),
        WHITE,
    );
}
