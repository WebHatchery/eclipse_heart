use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::CollectionCardKind;

use super::{PersistenceBundle, PersistenceManager};

fn temp_save_dir() -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    std::env::temp_dir().join(format!("eclipse_heart_save_test_{unique}"))
}

#[test]
fn missing_files_load_defaults() {
    let root = temp_save_dir();
    let manager = PersistenceManager::new(&root);

    let bundle = manager.load_all().expect("loads defaults");

    assert_eq!(bundle.profile.version, 1);
    assert!(bundle.collection.owned_magical_girls.is_empty());
    assert!(bundle.decks.support_decks.is_empty());
    assert!(bundle.campaigns.runs.is_empty());
    assert_eq!(bundle.settings.window_width, 2560);
    assert!(bundle.settings.display.fullscreen);

    std::fs::remove_dir_all(root).expect("cleanup temp dir");
}

#[test]
fn save_round_trip_preserves_values() {
    let root = temp_save_dir();
    let manager = PersistenceManager::new(&root);
    let mut bundle = PersistenceBundle::default();
    bundle.profile.player_name = "Yuki".to_owned();
    bundle.profile.total_matches_played = 7;
    bundle.profile.total_wins = 5;
    bundle
        .collection
        .add_owned(CollectionCardKind::MagicalGirl, "yuki", 1);
    bundle.collection.add_owned(
        CollectionCardKind::StoryCard,
        "quiet_lunch_on_the_rooftop",
        2,
    );
    bundle.decks.roster_presets = vec!["starter_a".to_owned()];
    bundle.campaigns.runs = Vec::new();
    bundle.settings.display.fullscreen = true;

    manager.save_all(&bundle).expect("save bundle");
    let loaded = manager.load_all().expect("reload bundle");

    assert_eq!(loaded.profile.player_name, "Yuki");
    assert_eq!(loaded.profile.total_wins, 5);
    assert_eq!(
        loaded
            .collection
            .owned_count(CollectionCardKind::MagicalGirl, "yuki"),
        1
    );
    assert_eq!(
        loaded
            .collection
            .owned_count(CollectionCardKind::StoryCard, "quiet_lunch_on_the_rooftop"),
        2
    );
    assert_eq!(loaded.decks.roster_presets, vec!["starter_a"]);
    assert!(loaded.campaigns.runs.is_empty());
    assert!(loaded.settings.display.fullscreen);

    std::fs::remove_dir_all(root).expect("cleanup temp dir");
}
