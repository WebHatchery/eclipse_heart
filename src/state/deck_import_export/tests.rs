use crate::data::GameContent;

use super::{export_deck_code, import_deck_code, DeckCodeError, ImportedDeck};
use crate::state::DeckPreset;

fn sample_deck() -> DeckPreset {
    DeckPreset {
        id: "deck_1".to_owned(),
        name: "Moonlit Recipe".to_owned(),
        source_template_id: None,
        notes: "notes".to_owned(),
        archetype_tags: vec!["Tempo".to_owned()],
        story_cards: vec![
            "quiet_lunch_on_the_rooftop".to_owned(),
            "quiet_lunch_on_the_rooftop".to_owned(),
            "crescent_counterpose".to_owned(),
        ],
        magical_girl_roster: vec![
            "yuki".to_owned(),
            "hana".to_owned(),
            "riri".to_owned(),
            "momo".to_owned(),
            "kiko".to_owned(),
        ],
        baddie_roster: vec![
            "noctra".to_owned(),
            "glass_crow".to_owned(),
            "thorn_waltz".to_owned(),
            "velvet_hex".to_owned(),
            "hollow_marionette".to_owned(),
        ],
        created_at_unix: 1,
        updated_at_unix: 1,
        recent_story_cards: vec!["crescent_counterpose".to_owned()],
    }
}

#[test]
fn deck_code_round_trip_preserves_contents() {
    let content = GameContent::load().expect("content loads");
    let deck = sample_deck();

    let code = export_deck_code(&deck);
    let imported = import_deck_code(&code, &content).expect("deck imports");

    assert_eq!(
        imported,
        ImportedDeck {
            name: deck.name,
            story_cards: deck.story_cards,
            magical_girl_roster: deck.magical_girl_roster,
            baddie_roster: deck.baddie_roster,
        }
    );
}

#[test]
fn import_rejects_unknown_card_ids() {
    let content = GameContent::load().expect("content loads");

    let code = r#"EH1:{"version":1,"name":"Broken","story_cards":["unknown_card"],"magical_girl_roster":["yuki","hana","riri","momo","kiko"],"baddie_roster":["noctra","glass_crow","thorn_waltz","velvet_hex","hollow_marionette"]}"#;

    assert_eq!(
        import_deck_code(code, &content),
        Err(DeckCodeError::UnknownStoryCard("unknown_card".to_owned()))
    );
}

#[test]
fn import_rejects_wrong_roster_size() {
    let content = GameContent::load().expect("content loads");

    let code = r#"EH1:{"version":1,"name":"Broken","story_cards":["quiet_lunch_on_the_rooftop"],"magical_girl_roster":["yuki"],"baddie_roster":["noctra","glass_crow","thorn_waltz","velvet_hex","hollow_marionette"]}"#;

    assert_eq!(
        import_deck_code(code, &content),
        Err(DeckCodeError::InvalidMagicalGirlRosterCount(1))
    );
}
