use crate::data::DeckRules;
use crate::state::{CollectionCardKind, CollectionSave, DeckPreset};

use super::{DeckValidation, DeckValidationCount};

#[test]
fn validation_marks_complete_legal_deck() {
    let deck = DeckPreset {
        id: "deck_1".to_owned(),
        name: "Complete Deck".to_owned(),
        source_template_id: None,
        notes: String::new(),
        archetype_tags: Vec::new(),
        story_cards: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        magical_girl_roster: vec![
            "mg1".to_owned(),
            "mg2".to_owned(),
            "mg3".to_owned(),
            "mg4".to_owned(),
            "mg5".to_owned(),
        ],
        baddie_roster: vec![
            "b1".to_owned(),
            "b2".to_owned(),
            "b3".to_owned(),
            "b4".to_owned(),
            "b5".to_owned(),
        ],
        created_at_unix: 1,
        updated_at_unix: 1,
        recent_story_cards: Vec::new(),
    };
    let rules = DeckRules {
        support_deck_size: 3,
        max_copies_per_story_card: 2,
        universal_copy_limit: true,
    };
    let mut collection = CollectionSave::default();
    for id in ["a", "b", "c"] {
        collection.add_owned(CollectionCardKind::StoryCard, id, 1);
    }
    for id in ["mg1", "mg2", "mg3", "mg4", "mg5"] {
        collection.add_owned(CollectionCardKind::MagicalGirl, id, 1);
    }
    for id in ["b1", "b2", "b3", "b4", "b5"] {
        collection.add_owned(CollectionCardKind::Baddie, id, 1);
    }

    let validation = DeckValidation::for_deck(&deck, &rules, &collection);

    assert!(validation.support_card_count_valid);
    assert!(validation.magical_girl_roster_valid);
    assert!(validation.baddie_roster_valid);
    assert!(validation.is_collection_complete);
    assert!(validation.is_legal);
}

#[test]
fn validation_reports_missing_cards_and_duplicates() {
    let deck = DeckPreset {
        id: "deck_2".to_owned(),
        name: "Broken Deck".to_owned(),
        source_template_id: None,
        notes: String::new(),
        archetype_tags: Vec::new(),
        story_cards: vec![
            "a".to_owned(),
            "a".to_owned(),
            "a".to_owned(),
            "missing_story".to_owned(),
        ],
        magical_girl_roster: vec![
            "mg1".to_owned(),
            "mg1".to_owned(),
            "mg3".to_owned(),
            "mg4".to_owned(),
        ],
        baddie_roster: vec![
            "b1".to_owned(),
            "b2".to_owned(),
            "b3".to_owned(),
            "b4".to_owned(),
            "missing_baddie".to_owned(),
        ],
        created_at_unix: 1,
        updated_at_unix: 1,
        recent_story_cards: Vec::new(),
    };
    let rules = DeckRules {
        support_deck_size: 5,
        max_copies_per_story_card: 2,
        universal_copy_limit: true,
    };
    let mut collection = CollectionSave::default();
    collection.add_owned(CollectionCardKind::StoryCard, "a", 1);
    collection.add_owned(CollectionCardKind::MagicalGirl, "mg1", 1);
    collection.add_owned(CollectionCardKind::MagicalGirl, "mg3", 1);
    collection.add_owned(CollectionCardKind::MagicalGirl, "mg4", 1);
    for id in ["b1", "b2", "b3", "b4"] {
        collection.add_owned(CollectionCardKind::Baddie, id, 1);
    }

    let validation = DeckValidation::for_deck(&deck, &rules, &collection);

    assert!(!validation.support_card_count_valid);
    assert!(!validation.magical_girl_roster_valid);
    assert!(validation.baddie_roster_valid);
    assert_eq!(validation.duplicate_story_cards, vec!["a".to_owned()]);
    assert_eq!(validation.duplicate_magical_girls, vec!["mg1".to_owned()]);
    assert_eq!(
        validation.missing_story_cards,
        vec![
            DeckValidationCount {
                id: "a".to_owned(),
                count: 2,
            },
            DeckValidationCount {
                id: "missing_story".to_owned(),
                count: 1,
            },
        ]
    );
    assert_eq!(
        validation.missing_baddies,
        vec!["missing_baddie".to_owned()]
    );
    assert_eq!(validation.missing_magical_girls, vec!["mg1".to_owned()]);
    assert_eq!(validation.missing_card_total, 5);
    assert!(!validation.is_collection_complete);
    assert!(!validation.is_legal);
}
