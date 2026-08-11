use crate::data::UiText;

use super::GameContent;

#[test]
fn loads_all_core_content_files() {
    let content = GameContent::load().expect("content loads");

    assert_eq!(content.magical_girls.len(), 5);
    assert_eq!(content.baddies.len(), 5);
    assert_eq!(content.story_cards.len(), 32);
    assert_eq!(content.deck_rules.support_deck_size, 40);
    assert!(content.progression_rules.overflow_is_lost);
    assert!(!content.starter_loadouts.is_empty());
    assert!(content
        .starter_loadouts
        .iter()
        .all(|starter| !starter.description.trim().is_empty()));
    assert!(content
        .starter_loadouts
        .iter()
        .all(|starter| !starter.playstyle.trim().is_empty()));
    assert_eq!(content.campaign.id, "magical_girl_rising");
    assert_eq!(content.campaign.nodes.len(), 3);
    assert_eq!(content.card_visuals.canvas.width, 750);
    assert_eq!(content.card_visuals.template_families.len(), 3);
    assert_eq!(content.card_visuals.speed_badges.len(), 3);
    assert_eq!(content.art_catalog.character_portraits.len(), 10);
    assert_eq!(content.art_catalog.story_card_art.len(), 32);
    assert_eq!(content.art_catalog.ui_backgrounds.len(), 3);
}

#[test]
fn loads_ui_text_catalog() {
    let ui_text = UiText::load().expect("ui text loads");

    assert_eq!(ui_text.get("menu_title"), "Eclipse Heart");
    assert_eq!(ui_text.get("deck_builder_title"), "Deck Builder");
    assert_eq!(ui_text.get("battle_stage_label"), "Stage");
}
