//! Campaign hub screen.

use macroquad::prelude::*;

use crate::data::{CampaignEncounterDefinition, CharacterDefinition, StoryCardDefinition};
use crate::screens::ScreenAction;
use crate::state::AppState;
use crate::ui::card_widgets::{action_button, point_in_rect};
use crate::ui::core::{
    draw_background_texture, draw_focus_panel, draw_screen_scrim, draw_soft_panel, BADDIE_PINK,
    MG_BLUE, PRIORITY_GOLD, TEXT_MUTED,
};
use crate::ui::layout::UiLayout;
use macroquad_toolkit::ui::{draw_ui_text, wrap_text};

pub struct CampaignHubScreen;

impl Default for CampaignHubScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl CampaignHubScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, state: &AppState) -> ScreenAction {
        let ui = UiLayout::current();
        let mouse = mouse_position();

        for target in support_targets(state) {
            if point_in_rect(target.rect, mouse) && is_mouse_button_pressed(MouseButton::Left) {
                return ScreenAction::CampaignToggleSupportSelection {
                    character_id: target.character_id.clone(),
                };
            }
        }

        if state.saves.campaigns.selected_run_is_in_progress()
            && state.saves.campaigns.selected_run_has_valid_support_pair()
            && action_button(
                ui.rect(1860.0, 1328.0, 620.0, 70.0),
                state.ui_text.get("campaign_start_encounter"),
            )
        {
            return ScreenAction::CampaignStartEncounter;
        }
        if action_button(
            ui.rect(80.0, 1328.0, 400.0, 70.0),
            state.ui_text.get("battle_back_to_menu"),
        ) {
            return ScreenAction::OpenCampaignMenu;
        }
        ScreenAction::None
    }

    pub fn draw(&self, state: &AppState) {
        let ui = UiLayout::current();
        if let Some(background) = state.assets.ui_background("campaign") {
            draw_background_texture(background, Color::new(1.0, 1.0, 1.0, 0.22));
        }
        draw_screen_scrim(0.58);
        let Some(run) = state.saves.campaigns.selected_run() else {
            draw_ui_text(
                state.ui_text.get("campaign_no_active_run"),
                ui.x(80.0),
                ui.y(110.0),
                ui.font(48.0),
                WHITE,
            );
            return;
        };
        let Some(node) = state.content.campaign.node(&run.current_node_id) else {
            draw_ui_text(
                state.ui_text.get("campaign_missing_node"),
                ui.x(80.0),
                ui.y(110.0),
                ui.font(48.0),
                RED,
            );
            return;
        };
        let Some(encounter) = state.content.campaign.encounter(&node.encounter_id) else {
            draw_ui_text(
                state.ui_text.get("campaign_missing_encounter"),
                ui.x(80.0),
                ui.y(110.0),
                ui.font(48.0),
                RED,
            );
            return;
        };

        draw_focus_panel(ui.rect(56.0, 58.0, 1210.0, 430.0), MG_BLUE);
        draw_focus_panel(ui.rect(56.0, 500.0, 1470.0, 580.0), PRIORITY_GOLD);
        draw_focus_panel(ui.rect(1580.0, 220.0, 850.0, 360.0), BADDIE_PINK);
        self.draw_encounter_card(state, encounter, ui.rect(1580.0, 220.0, 850.0, 360.0));

        draw_ui_text(
            state.ui_text.get("campaign_hub_title"),
            ui.x(80.0),
            ui.y(110.0),
            ui.font(72.0),
            WHITE,
        );
        draw_ui_text(
            &format!(
                "{}: {}",
                state.ui_text.get("campaign_selected_slot_label"),
                run.name
            ),
            ui.x(80.0),
            ui.y(164.0),
            ui.font(28.0),
            GOLD,
        );
        draw_ui_text(
            &format!(
                "{}: {}",
                state.ui_text.get("campaign_current_encounter_label"),
                encounter.name
            ),
            ui.x(80.0),
            ui.y(220.0),
            ui.font(32.0),
            WHITE,
        );
        draw_ui_text(
            &encounter.intro_text,
            ui.x(80.0),
            ui.y(266.0),
            ui.font(24.0),
            TEXT_MUTED,
        );
        draw_ui_text(
            &format!(
                "{}: {}/{}",
                state.ui_text.get("campaign_progress_label"),
                run.completed_node_ids.len(),
                state.content.campaign.nodes.len()
            ),
            ui.x(80.0),
            ui.y(328.0),
            ui.font(24.0),
            SKYBLUE,
        );
        draw_ui_text(
            &format!(
                "{}: {}",
                state.ui_text.get("campaign_deck_label"),
                run.player_deck.name
            ),
            ui.x(80.0),
            ui.y(384.0),
            ui.font(28.0),
            WHITE,
        );
        draw_ui_text(
            &format!(
                "{} {}",
                state.ui_text.get("campaign_deck_cards_label"),
                run.player_deck.story_cards.len()
            ),
            ui.x(80.0),
            ui.y(420.0),
            ui.font(24.0),
            TEXT_MUTED,
        );
        if !run.player_deck.recent_story_cards.is_empty() {
            draw_ui_text(
                &format!(
                    "{}: {}",
                    state.ui_text.get("campaign_recent_rewards_label"),
                    format_story_cards(
                        &run.player_deck.recent_story_cards,
                        &state.content.story_cards
                    )
                ),
                ui.x(80.0),
                ui.y(456.0),
                ui.font(20.0),
                TEXT_MUTED,
            );
        }

        draw_ui_text(
            state.ui_text.get("campaign_support_pair_label"),
            ui.x(80.0),
            ui.y(520.0),
            ui.font(24.0),
            WHITE,
        );
        draw_ui_text(
            state.ui_text.get("campaign_support_pair_help"),
            ui.x(80.0),
            ui.y(554.0),
            ui.font(18.0),
            TEXT_MUTED,
        );

        if let Some(main_character) = run
            .player_deck
            .magical_girl_roster
            .first()
            .and_then(|character_id| lookup_character(&state.content.magical_girls, character_id))
        {
            self.draw_character_tile(
                state,
                ui.rect(80.0, 590.0, 320.0, 118.0),
                main_character,
                true,
                false,
                false,
            );
        }

        let selected_supports = current_support_characters(state);
        for index in 0..2 {
            let rect = ui.rect(430.0 + index as f32 * 350.0, 590.0, 320.0, 118.0);
            if let Some(character) = selected_supports.get(index) {
                self.draw_character_tile(state, rect, character, true, false, true);
            } else {
                self.draw_empty_support_slot(state, rect);
            }
        }

        for target in support_targets(state) {
            self.draw_character_tile(
                state,
                target.rect,
                target.character,
                target.selected,
                target.hovered,
                true,
            );
        }

        draw_ui_text(
            &format!(
                "{}: {}",
                state.ui_text.get("campaign_run_roster_label"),
                format_character_roster(
                    &run.player_deck.magical_girl_roster,
                    &state.content.magical_girls
                )
            ),
            ui.x(80.0),
            ui.y(1036.0),
            ui.font(20.0),
            TEXT_MUTED,
        );
        if !state.saves.campaigns.selected_run_has_valid_support_pair() {
            draw_ui_text(
                state.ui_text.get("campaign_support_pair_required"),
                ui.x(80.0),
                ui.y(1076.0),
                ui.font(22.0),
                ORANGE,
            );
        }
        if let Some(message) = &state.campaign_notice {
            draw_ui_text(message, ui.x(80.0), ui.y(1112.0), ui.font(24.0), GOLD);
        }
    }

    fn draw_character_tile(
        &self,
        state: &AppState,
        rect: Rect,
        character: &CharacterDefinition,
        selected: bool,
        hovered: bool,
        selectable: bool,
    ) {
        let ui = UiLayout::current();
        let outline = if selected {
            PRIORITY_GOLD
        } else if hovered {
            WHITE
        } else {
            MG_BLUE
        };
        draw_soft_panel(rect.x, rect.y, rect.w, rect.h, DARKGRAY);
        if let Some(texture) = state.assets.portrait(&character.id) {
            draw_texture_ex(
                texture,
                rect.x + ui.w(6.0),
                rect.y + ui.h(6.0),
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(ui.w(92.0), rect.h - ui.h(12.0))),
                    ..Default::default()
                },
            );
        }
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, outline);
        draw_ui_text(
            &character.name,
            rect.x + ui.w(110.0),
            rect.y + ui.h(32.0),
            ui.font(22.0),
            WHITE,
        );
        draw_ui_text(
            &format!(
                "{} / {} / {}",
                character.base_power, character.transformed_power, character.final_power
            ),
            rect.x + ui.w(110.0),
            rect.y + ui.h(66.0),
            ui.font(18.0),
            TEXT_MUTED,
        );
        draw_ui_text(
            &format!(
                "{} {} / {}",
                state.ui_text.get("battle_growth_label"),
                character.first_threshold,
                character.second_threshold
            ),
            rect.x + ui.w(110.0),
            rect.y + ui.h(96.0),
            ui.font(18.0),
            if selectable { TEXT_MUTED } else { GOLD },
        );
    }

    fn draw_empty_support_slot(&self, state: &AppState, rect: Rect) {
        let ui = UiLayout::current();
        draw_soft_panel(rect.x, rect.y, rect.w, rect.h, DARKGRAY);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, GRAY);
        draw_ui_text(
            state.ui_text.get("campaign_support_slot_empty"),
            rect.x + ui.w(14.0),
            rect.y + ui.h(58.0),
            ui.font(20.0),
            TEXT_MUTED,
        );
    }

    fn draw_encounter_card(
        &self,
        state: &AppState,
        encounter: &CampaignEncounterDefinition,
        rect: Rect,
    ) {
        let ui = UiLayout::current();
        draw_ui_text(
            state.ui_text.get("campaign_current_encounter_label"),
            rect.x + ui.w(42.0),
            rect.y + ui.h(62.0),
            ui.font(26.0),
            TEXT_MUTED,
        );
        draw_ui_text(
            &encounter.name,
            rect.x + ui.w(42.0),
            rect.y + ui.h(118.0),
            ui.font(56.0),
            WHITE,
        );
        for (index, line) in wrap_text(&encounter.intro_text, rect.w - ui.w(84.0), ui.font(24.0))
            .into_iter()
            .take(2)
            .enumerate()
        {
            draw_ui_text(
                &line,
                rect.x + ui.w(42.0),
                rect.y + ui.h(172.0 + index as f32 * 34.0),
                ui.font(24.0),
                TEXT_MUTED,
            );
        }
        let reward_names =
            format_story_cards(&encounter.reward_story_card_ids, &state.content.story_cards);
        draw_ui_text(
            state.ui_text.get("campaign_reward_label"),
            rect.x + ui.w(42.0),
            rect.y + ui.h(268.0),
            ui.font(22.0),
            PRIORITY_GOLD,
        );
        draw_ui_text(
            if reward_names.is_empty() {
                state.ui_text.get("campaign_no_reward_label")
            } else {
                &reward_names
            },
            rect.x + ui.w(42.0),
            rect.y + ui.h(304.0),
            ui.font(22.0),
            WHITE,
        );
    }
}

struct SupportTarget<'a> {
    character_id: String,
    rect: Rect,
    character: &'a CharacterDefinition,
    selected: bool,
    hovered: bool,
}

fn support_targets<'a>(state: &'a AppState) -> Vec<SupportTarget<'a>> {
    let ui = UiLayout::current();
    let mouse = mouse_position();
    let Some(run) = state.saves.campaigns.selected_run() else {
        return Vec::new();
    };
    let selected_ids = &run.selected_magical_girl_support_ids;

    run.player_deck
        .magical_girl_roster
        .iter()
        .skip(1)
        .enumerate()
        .filter_map(|(index, character_id)| {
            let character = lookup_character(&state.content.magical_girls, character_id)?;
            let rect = ui.rect(80.0 + index as f32 * 350.0, 748.0, 320.0, 118.0);
            Some(SupportTarget {
                character_id: character_id.clone(),
                rect,
                character,
                selected: selected_ids.iter().any(|entry| entry == character_id),
                hovered: point_in_rect(rect, mouse),
            })
        })
        .collect()
}

fn current_support_characters(state: &AppState) -> Vec<&CharacterDefinition> {
    let Some(run) = state.saves.campaigns.selected_run() else {
        return Vec::new();
    };
    run.selected_magical_girl_support_ids
        .iter()
        .filter_map(|character_id| lookup_character(&state.content.magical_girls, character_id))
        .collect()
}

fn lookup_character<'a>(
    definitions: &'a [CharacterDefinition],
    character_id: &str,
) -> Option<&'a CharacterDefinition> {
    definitions.iter().find(|entry| entry.id == character_id)
}

fn format_character_roster(roster: &[String], definitions: &[CharacterDefinition]) -> String {
    roster
        .iter()
        .map(|character_id| {
            lookup_character(definitions, character_id)
                .map(|entry| entry.name.clone())
                .unwrap_or_else(|| character_id.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_story_cards(card_ids: &[String], definitions: &[StoryCardDefinition]) -> String {
    card_ids
        .iter()
        .map(|card_id| {
            definitions
                .iter()
                .find(|entry| &entry.id == card_id)
                .map(|entry| entry.name.clone())
                .unwrap_or_else(|| card_id.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}
