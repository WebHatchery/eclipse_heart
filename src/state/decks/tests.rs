use crate::data::{DeckRules, StarterLoadout};

use super::{CollectionSave, DecksSave};
use crate::state::CollectionCardKind;

fn sample_starter() -> StarterLoadout {
    StarterLoadout {
        id: "starter_alpha".to_owned(),
        name: "Starter Alpha".to_owned(),
        description: "Sample starter".to_owned(),
        playstyle: "Balanced".to_owned(),
        magical_girl_main: "yuki".to_owned(),
        magical_girl_supports: vec!["hana".to_owned(), "riri".to_owned()],
        prime_baddie: "noctra".to_owned(),
        baddie_supports: vec!["glass_crow".to_owned(), "thorn_waltz".to_owned()],
        support_deck: vec!["quiet_lunch_on_the_rooftop".to_owned()],
    }
}

#[test]
fn create_select_duplicate_and_delete_decks_work_independently() {
    let starter = sample_starter();
    let mut decks = DecksSave::default();
    let magical_girls = [
        "yuki".to_owned(),
        "hana".to_owned(),
        "riri".to_owned(),
        "momo".to_owned(),
        "kiko".to_owned(),
    ];
    let baddies = [
        "noctra".to_owned(),
        "glass_crow".to_owned(),
        "thorn_waltz".to_owned(),
        "velvet_hex".to_owned(),
        "hollow_marionette".to_owned(),
    ];

    let starter_deck_id =
        decks.create_deck_from_template(&starter, &magical_girls, &baddies, "Copy");
    assert_eq!(
        decks
            .selected_support_deck()
            .and_then(|deck| deck.source_template_id.as_deref()),
        Some("starter_alpha")
    );

    let custom_deck_id = decks.create_empty_deck("New Deck", &magical_girls, &baddies);
    assert_eq!(
        decks.selected_support_deck().map(|deck| deck.name.as_str()),
        Some("New Deck")
    );

    assert!(decks.select_support_deck(&starter_deck_id));
    let duplicated_id = decks
        .duplicate_selected_deck("Copy")
        .expect("duplicate deck");
    assert_ne!(duplicated_id, starter_deck_id);
    assert_eq!(
        decks
            .selected_support_deck()
            .map(|deck| deck.story_cards.len()),
        Some(1)
    );

    assert!(decks.select_support_deck(&custom_deck_id));
    assert!(decks.rename_selected_deck("Control Build"));
    assert_eq!(
        decks.selected_support_deck().map(|deck| deck.name.as_str()),
        Some("Control Build")
    );

    assert!(decks.delete_selected_deck());
    assert!(decks.selected_support_deck().is_some());
    assert_eq!(decks.support_decks.len(), 2);
}

#[test]
fn deck_save_respects_copy_limit_for_selected_deck() {
    let starter = sample_starter();
    let mut decks = DecksSave::default();
    let rules = DeckRules {
        support_deck_size: 5,
        max_copies_per_story_card: 2,
        universal_copy_limit: true,
    };

    decks.create_deck_from_template(
        &starter,
        &["yuki".to_owned(), "hana".to_owned(), "riri".to_owned()],
        &[
            "noctra".to_owned(),
            "glass_crow".to_owned(),
            "thorn_waltz".to_owned(),
        ],
        "Copy",
    );
    assert_eq!(
        decks.selected_support_deck().map(|deck| deck.name.as_str()),
        Some("Starter Alpha")
    );

    let mut collection = CollectionSave::default();
    collection.add_owned(CollectionCardKind::StoryCard, "not_on_my_watch", 2);

    assert!(decks.add_card("not_on_my_watch", &rules, &collection));
    assert!(decks.add_card("not_on_my_watch", &rules, &collection));
    assert!(!decks.add_card("not_on_my_watch", &rules, &collection));

    assert!(decks.remove_card("not_on_my_watch"));
    assert_eq!(decks.card_count("not_on_my_watch"), 1);
}

#[test]
fn set_roster_slot_swaps_existing_character_into_selected_slot() {
    let starter = sample_starter();
    let mut decks = DecksSave::default();
    let magical_girls = [
        "yuki".to_owned(),
        "hana".to_owned(),
        "riri".to_owned(),
        "momo".to_owned(),
        "kiko".to_owned(),
    ];
    let baddies = [
        "noctra".to_owned(),
        "glass_crow".to_owned(),
        "thorn_waltz".to_owned(),
        "velvet_hex".to_owned(),
        "hollow_marionette".to_owned(),
    ];
    decks.create_deck_from_template(&starter, &magical_girls, &baddies, "Copy");

    assert!(decks.set_roster_slot(true, 0, "hana"));
    assert!(decks.set_roster_slot(true, 0, "riri"));
    assert_eq!(
        decks
            .selected_support_deck()
            .expect("selected deck")
            .magical_girl_roster[0],
        "riri"
    );
    assert_eq!(
        decks
            .selected_support_deck()
            .expect("selected deck")
            .magical_girl_roster[2],
        "hana"
    );
}
