use crate::data::{CardAlignment, CardSpeed, GameContent, StoryCardDefinition};
use crate::state::{CollectionCardKind, CollectionSave, DeckPreset};

use super::suggest_story_replacements;

fn card(
    id: &str,
    speed: CardSpeed,
    alignment: CardAlignment,
    card_type: &str,
) -> StoryCardDefinition {
    StoryCardDefinition {
        id: id.to_owned(),
        name: id.to_owned(),
        card_type: card_type.to_owned(),
        speed,
        alignment,
        playable_in_daily_life: true,
        effects: Vec::new(),
    }
}

#[test]
fn replacement_suggestions_prefer_matching_alignment_and_type() {
    let mut content = GameContent::default();
    content.deck_rules.support_deck_size = 3;
    content.deck_rules.max_copies_per_story_card = 2;
    content.story_cards = vec![
        card(
            "missing",
            CardSpeed::Reaction,
            CardAlignment::MagicalGirl,
            "guard",
        ),
        card(
            "best_fit",
            CardSpeed::Reaction,
            CardAlignment::MagicalGirl,
            "guard",
        ),
        card(
            "same_speed_only",
            CardSpeed::Reaction,
            CardAlignment::Neutral,
            "setup",
        ),
        card(
            "wrong_speed",
            CardSpeed::Encounter,
            CardAlignment::MagicalGirl,
            "guard",
        ),
    ];

    let deck = DeckPreset {
        id: "deck_1".to_owned(),
        name: "Test".to_owned(),
        source_template_id: None,
        notes: String::new(),
        archetype_tags: Vec::new(),
        story_cards: vec![
            "missing".to_owned(),
            "missing".to_owned(),
            "best_fit".to_owned(),
        ],
        magical_girl_roster: vec!["mg1".to_owned(); 5],
        baddie_roster: vec!["b1".to_owned(); 5],
        created_at_unix: 1,
        updated_at_unix: 1,
        recent_story_cards: Vec::new(),
    };

    let mut collection = CollectionSave::default();
    collection.add_owned(CollectionCardKind::StoryCard, "missing", 1);
    collection.add_owned(CollectionCardKind::StoryCard, "best_fit", 2);
    collection.add_owned(CollectionCardKind::StoryCard, "same_speed_only", 2);

    let suggestions = suggest_story_replacements(&deck, &content, &collection, 2);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].missing_card_id, "missing");
    assert_eq!(
        suggestions[0].replacement_card_ids,
        vec!["best_fit".to_owned(), "same_speed_only".to_owned()]
    );
}
