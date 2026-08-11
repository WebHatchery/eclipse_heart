use super::{CollectionCardKind, CollectionSave};

#[test]
fn ensure_full_roster_owned_sets_minimum_three_copies_without_overfilling() {
    let mut collection = CollectionSave::default();
    collection.add_owned(CollectionCardKind::MagicalGirl, "yuki", 1);
    collection.add_owned(CollectionCardKind::Baddie, "noctra", 5);

    collection.ensure_full_roster_owned(
        ["yuki".to_owned(), "hana".to_owned()].into_iter(),
        ["noctra".to_owned(), "glass_crow".to_owned()].into_iter(),
    );

    assert_eq!(
        collection.owned_count(CollectionCardKind::MagicalGirl, "yuki"),
        3
    );
    assert_eq!(
        collection.owned_count(CollectionCardKind::MagicalGirl, "hana"),
        3
    );
    assert_eq!(
        collection.owned_count(CollectionCardKind::Baddie, "noctra"),
        5
    );
    assert_eq!(
        collection.owned_count(CollectionCardKind::Baddie, "glass_crow"),
        3
    );
}
