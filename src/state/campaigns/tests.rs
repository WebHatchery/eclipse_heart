use crate::data::{CampaignDefinition, CampaignEncounterDefinition, CampaignNodeDefinition};
use crate::state::{CampaignRunStatus, DeckPreset};

use super::CampaignSaveBundle;

fn sample_campaign() -> CampaignDefinition {
    CampaignDefinition {
        id: "campaign_alpha".to_owned(),
        name: "Campaign Alpha".to_owned(),
        description: String::new(),
        nodes: vec![
            CampaignNodeDefinition {
                id: "node_1".to_owned(),
                encounter_id: "encounter_1".to_owned(),
                next_node_ids: vec!["node_2".to_owned()],
                boss: false,
            },
            CampaignNodeDefinition {
                id: "node_2".to_owned(),
                encounter_id: "encounter_2".to_owned(),
                next_node_ids: Vec::new(),
                boss: true,
            },
        ],
        encounters: vec![
            CampaignEncounterDefinition {
                id: "encounter_1".to_owned(),
                name: "First".to_owned(),
                enemy_loadout_id: "starter_a".to_owned(),
                intro_text: String::new(),
                reward_story_card_ids: vec!["reward_a".to_owned()],
                boss: false,
            },
            CampaignEncounterDefinition {
                id: "encounter_2".to_owned(),
                name: "Second".to_owned(),
                enemy_loadout_id: "starter_b".to_owned(),
                intro_text: String::new(),
                reward_story_card_ids: vec!["reward_b".to_owned()],
                boss: true,
            },
        ],
    }
}

fn sample_deck() -> DeckPreset {
    DeckPreset {
        id: "deck_1".to_owned(),
        name: "Moonlight".to_owned(),
        story_cards: vec!["quiet_lunch_on_the_rooftop".to_owned()],
        magical_girl_roster: vec!["yuki".to_owned(), "hana".to_owned(), "riri".to_owned()],
        baddie_roster: vec![
            "noctra".to_owned(),
            "glass_crow".to_owned(),
            "thorn_waltz".to_owned(),
        ],
        ..DeckPreset::default()
    }
}

#[test]
fn create_run_selects_new_slot_without_closing_existing_runs() {
    let campaign = sample_campaign();
    let mut saves = CampaignSaveBundle::default();

    saves.create_run(&campaign, &sample_deck(), "New Run");
    saves.create_run(&campaign, &sample_deck(), "New Run");

    assert_eq!(saves.runs.len(), 2);
    assert_eq!(saves.runs[0].status, CampaignRunStatus::InProgress);
    assert_eq!(
        saves.selected_run().map(|run| run.id.as_str()),
        Some("campaign_run_2")
    );
    assert_eq!(
        saves
            .selected_run()
            .map(|run| run.selected_magical_girl_support_ids.clone()),
        Some(Vec::new())
    );
}

#[test]
fn record_victory_advances_and_then_completes_selected_run() {
    let campaign = sample_campaign();
    let mut saves = CampaignSaveBundle::default();
    let run_id = saves
        .create_run(&campaign, &sample_deck(), "New Run")
        .expect("run");

    assert_eq!(
        saves.record_victory_for_run(&run_id, &campaign, "node_1", Some("reward_a")),
        Some(false)
    );
    assert_eq!(
        saves.selected_run().map(|run| run.current_node_id.as_str()),
        Some("node_2")
    );

    assert_eq!(
        saves.record_victory_for_run(&run_id, &campaign, "node_2", Some("reward_b")),
        Some(true)
    );
    assert_eq!(saves.runs[0].status, CampaignRunStatus::Won);
    assert!(saves.runs[0]
        .player_deck
        .story_cards
        .iter()
        .any(|card| card == "reward_b"));
}

#[test]
fn select_run_switches_current_slot() {
    let campaign = sample_campaign();
    let mut saves = CampaignSaveBundle::default();
    let first_run_id = saves
        .create_run(&campaign, &sample_deck(), "New Run")
        .expect("first run");
    let second_run_id = saves
        .create_run(&campaign, &sample_deck(), "New Run")
        .expect("second run");

    assert!(saves.select_run(&first_run_id));
    assert_eq!(
        saves.selected_run().map(|run| run.id.as_str()),
        Some(first_run_id.as_str())
    );
    assert_ne!(first_run_id, second_run_id);
}

#[test]
fn selected_support_pair_can_be_updated_for_selected_run() {
    let campaign = sample_campaign();
    let mut saves = CampaignSaveBundle::default();
    saves.create_run(&campaign, &sample_deck(), "New Run");

    assert!(saves.update_selected_magical_girl_supports(&["riri".to_owned(), "momo".to_owned()]));
    assert_eq!(
        saves
            .selected_run()
            .map(|run| run.selected_magical_girl_support_ids.clone()),
        Some(vec!["riri".to_owned(), "momo".to_owned()])
    );
}

#[test]
fn toggled_supports_can_be_cleared_and_do_not_autofill() {
    let campaign = sample_campaign();
    let mut saves = CampaignSaveBundle::default();
    saves.create_run(&campaign, &sample_deck(), "New Run");

    assert!(saves.toggle_selected_magical_girl_support("hana"));
    assert_eq!(
        saves
            .selected_run()
            .map(|run| run.selected_magical_girl_support_ids.clone()),
        Some(vec!["hana".to_owned()])
    );
    assert!(saves.toggle_selected_magical_girl_support("hana"));
    assert_eq!(
        saves
            .selected_run()
            .map(|run| run.selected_magical_girl_support_ids.clone()),
        Some(Vec::new())
    );
}
