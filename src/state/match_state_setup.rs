use std::collections::HashMap;

use super::*;

impl MatchState {
    pub fn from_setup(content: &GameContent, setup: &MatchSetup) -> Self {
        Self::from_setup_with_options(
            content,
            setup,
            default_player_a_deck(),
            default_player_b_deck(),
            PlayerId::PlayerA,
        )
    }

    pub fn from_starter_loadouts(
        content: &GameContent,
        player_a: &StarterLoadout,
        player_b: &StarterLoadout,
        starting_player: PlayerId,
    ) -> Self {
        let setup = MatchSetup::from_starter_loadouts(content, player_a, player_b);
        Self::from_setup_with_options(
            content,
            &setup,
            player_a.support_deck.clone(),
            player_b.support_deck.clone(),
            starting_player,
        )
    }

    pub fn default_player_a_support_deck() -> Vec<String> {
        default_player_a_deck()
    }

    pub fn default_player_b_support_deck() -> Vec<String> {
        default_player_b_deck()
    }

    pub fn from_setup_with_options(
        content: &GameContent,
        setup: &MatchSetup,
        mut player_a_deck: Vec<String>,
        mut player_b_deck: Vec<String>,
        starting_player: PlayerId,
    ) -> Self {
        let story_cards = content
            .story_cards
            .iter()
            .cloned()
            .map(|card| (card.id.clone(), card))
            .collect::<HashMap<_, _>>();

        let player_a_hand = draw_opening_hand(&mut player_a_deck, 5);
        let player_b_hand = draw_opening_hand(&mut player_b_deck, 5);

        Self {
            rules: content.rules.clone(),
            story_cards,
            phase: MatchPhase::DailyLife,
            round: 1,
            final_climax_active: false,
            prime_baddie_defeated: false,
            defeated_prime_owner: None,
            winner: None,
            player_a: PlayerState {
                magical_girls: build_side(
                    &content.magical_girls,
                    setup.player_a_mg_main_index,
                    setup.player_a_mg_support_pair_index,
                ),
                baddies: build_side(
                    &content.baddies,
                    setup.player_a_baddie_main_index,
                    setup.player_a_baddie_support_pair_index,
                ),
                deck: player_a_deck,
                hand: player_a_hand,
                discard: Vec::new(),
                encounter_card_played: false,
                supports_revealed_this_round: 0,
            },
            player_b: PlayerState {
                magical_girls: build_side(
                    &content.magical_girls,
                    setup.player_b_mg_main_index,
                    setup.player_b_mg_support_pair_index,
                ),
                baddies: build_side(
                    &content.baddies,
                    setup.player_b_baddie_main_index,
                    setup.player_b_baddie_support_pair_index,
                ),
                deck: player_b_deck,
                hand: player_b_hand,
                discard: Vec::new(),
                encounter_card_played: false,
                supports_revealed_this_round: 0,
            },
            active_player: starting_player,
            priority_player: starting_player,
            phase_passes: 0,
            last_played_card_name: None,
            last_outcome: None,
            reaction_stack: Vec::new(),
            reaction_state: None,
            event_log: Vec::new(),
        }
    }
}

impl MatchSetup {
    pub fn default_for_content(content: &GameContent) -> Self {
        let player_a_mg_main_index = preferred_index(&content.magical_girls, "yuki");
        let player_a_baddie_main_index = preferred_index(&content.baddies, "noctra");
        let player_b_mg_main_index = preferred_index(&content.magical_girls, "hana");
        let player_b_baddie_main_index = preferred_index(&content.baddies, "velvet_hex");

        Self {
            player_a_mg_main_index,
            player_a_mg_support_pair_index: support_pair_index_for_main(
                &content.magical_girls,
                player_a_mg_main_index,
                ["hana", "riri"],
            ),
            player_a_baddie_main_index,
            player_a_baddie_support_pair_index: support_pair_index_for_main(
                &content.baddies,
                player_a_baddie_main_index,
                ["glass_crow", "thorn_waltz"],
            ),
            player_a_support_deck_id: None,
            player_b_mg_main_index,
            player_b_mg_support_pair_index: support_pair_index_for_main(
                &content.magical_girls,
                player_b_mg_main_index,
                ["momo", "kiko"],
            ),
            player_b_baddie_main_index,
            player_b_baddie_support_pair_index: support_pair_index_for_main(
                &content.baddies,
                player_b_baddie_main_index,
                ["noctra", "hollow_marionette"],
            ),
            player_b_support_deck_id: None,
        }
    }

    pub fn from_starter_loadouts(
        content: &GameContent,
        player_a: &StarterLoadout,
        player_b: &StarterLoadout,
    ) -> Self {
        Self {
            player_a_mg_main_index: required_index(
                &content.magical_girls,
                &player_a.magical_girl_main,
            ),
            player_a_mg_support_pair_index: support_pair_index_for_ids(
                &content.magical_girls,
                &player_a.magical_girl_main,
                &player_a.magical_girl_supports,
            ),
            player_a_baddie_main_index: required_index(&content.baddies, &player_a.prime_baddie),
            player_a_baddie_support_pair_index: support_pair_index_for_ids(
                &content.baddies,
                &player_a.prime_baddie,
                &player_a.baddie_supports,
            ),
            player_a_support_deck_id: None,
            player_b_mg_main_index: required_index(
                &content.magical_girls,
                &player_b.magical_girl_main,
            ),
            player_b_mg_support_pair_index: support_pair_index_for_ids(
                &content.magical_girls,
                &player_b.magical_girl_main,
                &player_b.magical_girl_supports,
            ),
            player_b_baddie_main_index: required_index(&content.baddies, &player_b.prime_baddie),
            player_b_baddie_support_pair_index: support_pair_index_for_ids(
                &content.baddies,
                &player_b.prime_baddie,
                &player_b.baddie_supports,
            ),
            player_b_support_deck_id: None,
        }
    }

    pub fn from_player_deck_and_enemy_loadout(
        content: &GameContent,
        player_deck: &crate::state::DeckPreset,
        selected_magical_girl_supports: &[String],
        enemy: &StarterLoadout,
    ) -> Self {
        let player_mg_main = player_deck
            .magical_girl_roster
            .first()
            .expect("campaign player deck missing magical girl roster entry");
        let player_baddie_main = player_deck
            .baddie_roster
            .first()
            .expect("campaign player deck missing baddie roster entry");
        let player_mg_supports = if selected_magical_girl_supports.len() == 2 {
            selected_magical_girl_supports.to_vec()
        } else {
            player_deck.magical_girl_roster[1..3].to_vec()
        };
        let player_baddie_supports = player_deck.baddie_roster[1..3].to_vec();

        Self {
            player_a_mg_main_index: required_index(&content.magical_girls, player_mg_main),
            player_a_mg_support_pair_index: support_pair_index_for_ids(
                &content.magical_girls,
                player_mg_main,
                &player_mg_supports,
            ),
            player_a_baddie_main_index: required_index(&content.baddies, player_baddie_main),
            player_a_baddie_support_pair_index: support_pair_index_for_ids(
                &content.baddies,
                player_baddie_main,
                &player_baddie_supports,
            ),
            player_a_support_deck_id: None,
            player_b_mg_main_index: required_index(
                &content.magical_girls,
                &enemy.magical_girl_main,
            ),
            player_b_mg_support_pair_index: support_pair_index_for_ids(
                &content.magical_girls,
                &enemy.magical_girl_main,
                &enemy.magical_girl_supports,
            ),
            player_b_baddie_main_index: required_index(&content.baddies, &enemy.prime_baddie),
            player_b_baddie_support_pair_index: support_pair_index_for_ids(
                &content.baddies,
                &enemy.prime_baddie,
                &enemy.baddie_supports,
            ),
            player_b_support_deck_id: None,
        }
    }

    pub fn assign_support_deck(&mut self, player: PlayerId, deck_id: Option<String>) {
        match player {
            PlayerId::PlayerA => self.player_a_support_deck_id = deck_id,
            PlayerId::PlayerB => self.player_b_support_deck_id = deck_id,
        }
    }

    pub fn clear_missing_support_deck_assignments(&mut self, available_deck_ids: &[String]) {
        if self
            .player_a_support_deck_id
            .as_deref()
            .is_some_and(|deck_id| !available_deck_ids.iter().any(|entry| entry == deck_id))
        {
            self.player_a_support_deck_id = None;
        }
        if self
            .player_b_support_deck_id
            .as_deref()
            .is_some_and(|deck_id| !available_deck_ids.iter().any(|entry| entry == deck_id))
        {
            self.player_b_support_deck_id = None;
        }
    }

    pub fn cycle_player_a_mg_main(&mut self, content: &GameContent) {
        self.player_a_mg_main_index =
            cycle_index(content.magical_girls.len(), self.player_a_mg_main_index);
        self.player_a_mg_support_pair_index = reset_pair_index_if_needed(
            &content.magical_girls,
            self.player_a_mg_main_index,
            self.player_a_mg_support_pair_index,
        );
    }

    pub fn cycle_player_a_mg_supports(&mut self, content: &GameContent) {
        self.player_a_mg_support_pair_index = cycle_pair_index(
            &content.magical_girls,
            self.player_a_mg_main_index,
            self.player_a_mg_support_pair_index,
        );
    }

    pub fn cycle_player_a_baddie_main(&mut self, content: &GameContent) {
        self.player_a_baddie_main_index =
            cycle_index(content.baddies.len(), self.player_a_baddie_main_index);
        self.player_a_baddie_support_pair_index = reset_pair_index_if_needed(
            &content.baddies,
            self.player_a_baddie_main_index,
            self.player_a_baddie_support_pair_index,
        );
    }

    pub fn cycle_player_a_baddie_supports(&mut self, content: &GameContent) {
        self.player_a_baddie_support_pair_index = cycle_pair_index(
            &content.baddies,
            self.player_a_baddie_main_index,
            self.player_a_baddie_support_pair_index,
        );
    }

    pub fn cycle_player_b_mg_main(&mut self, content: &GameContent) {
        self.player_b_mg_main_index =
            cycle_index(content.magical_girls.len(), self.player_b_mg_main_index);
        self.player_b_mg_support_pair_index = reset_pair_index_if_needed(
            &content.magical_girls,
            self.player_b_mg_main_index,
            self.player_b_mg_support_pair_index,
        );
    }

    pub fn cycle_player_b_mg_supports(&mut self, content: &GameContent) {
        self.player_b_mg_support_pair_index = cycle_pair_index(
            &content.magical_girls,
            self.player_b_mg_main_index,
            self.player_b_mg_support_pair_index,
        );
    }

    pub fn cycle_player_b_baddie_main(&mut self, content: &GameContent) {
        self.player_b_baddie_main_index =
            cycle_index(content.baddies.len(), self.player_b_baddie_main_index);
        self.player_b_baddie_support_pair_index = reset_pair_index_if_needed(
            &content.baddies,
            self.player_b_baddie_main_index,
            self.player_b_baddie_support_pair_index,
        );
    }

    pub fn cycle_player_b_baddie_supports(&mut self, content: &GameContent) {
        self.player_b_baddie_support_pair_index = cycle_pair_index(
            &content.baddies,
            self.player_b_baddie_main_index,
            self.player_b_baddie_support_pair_index,
        );
    }

    pub fn select_main(
        &mut self,
        content: &GameContent,
        player: PlayerId,
        is_magical_girl_side: bool,
        main_index: usize,
    ) {
        let definitions = if is_magical_girl_side {
            &content.magical_girls
        } else {
            &content.baddies
        };
        let clamped_index = main_index.min(definitions.len().saturating_sub(1));

        match (player, is_magical_girl_side) {
            (PlayerId::PlayerA, true) => {
                self.player_a_mg_main_index = clamped_index;
                self.player_a_mg_support_pair_index = reset_pair_index_if_needed(
                    definitions,
                    self.player_a_mg_main_index,
                    self.player_a_mg_support_pair_index,
                );
            }
            (PlayerId::PlayerA, false) => {
                self.player_a_baddie_main_index = clamped_index;
                self.player_a_baddie_support_pair_index = reset_pair_index_if_needed(
                    definitions,
                    self.player_a_baddie_main_index,
                    self.player_a_baddie_support_pair_index,
                );
            }
            (PlayerId::PlayerB, true) => {
                self.player_b_mg_main_index = clamped_index;
                self.player_b_mg_support_pair_index = reset_pair_index_if_needed(
                    definitions,
                    self.player_b_mg_main_index,
                    self.player_b_mg_support_pair_index,
                );
            }
            (PlayerId::PlayerB, false) => {
                self.player_b_baddie_main_index = clamped_index;
                self.player_b_baddie_support_pair_index = reset_pair_index_if_needed(
                    definitions,
                    self.player_b_baddie_main_index,
                    self.player_b_baddie_support_pair_index,
                );
            }
        }
    }

    pub fn select_support_pair(
        &mut self,
        content: &GameContent,
        player: PlayerId,
        is_magical_girl_side: bool,
        pair_index: usize,
    ) {
        let pair_count = match (player, is_magical_girl_side) {
            (PlayerId::PlayerA, true) => {
                support_pairs(content.magical_girls.len(), self.player_a_mg_main_index).len()
            }
            (PlayerId::PlayerA, false) => {
                support_pairs(content.baddies.len(), self.player_a_baddie_main_index).len()
            }
            (PlayerId::PlayerB, true) => {
                support_pairs(content.magical_girls.len(), self.player_b_mg_main_index).len()
            }
            (PlayerId::PlayerB, false) => {
                support_pairs(content.baddies.len(), self.player_b_baddie_main_index).len()
            }
        };
        let clamped_index = pair_index.min(pair_count.saturating_sub(1));

        match (player, is_magical_girl_side) {
            (PlayerId::PlayerA, true) => self.player_a_mg_support_pair_index = clamped_index,
            (PlayerId::PlayerA, false) => self.player_a_baddie_support_pair_index = clamped_index,
            (PlayerId::PlayerB, true) => self.player_b_mg_support_pair_index = clamped_index,
            (PlayerId::PlayerB, false) => self.player_b_baddie_support_pair_index = clamped_index,
        }
    }

    pub fn player_a_mg_main_name<'a>(&self, content: &'a GameContent) -> &'a str {
        lookup_name(&content.magical_girls, self.player_a_mg_main_index)
    }

    pub fn player_a_mg_support_names(&self, content: &GameContent) -> Vec<String> {
        support_names(
            &content.magical_girls,
            self.player_a_mg_main_index,
            self.player_a_mg_support_pair_index,
        )
    }

    pub fn player_a_baddie_main_name<'a>(&self, content: &'a GameContent) -> &'a str {
        lookup_name(&content.baddies, self.player_a_baddie_main_index)
    }

    pub fn player_a_baddie_support_names(&self, content: &GameContent) -> Vec<String> {
        support_names(
            &content.baddies,
            self.player_a_baddie_main_index,
            self.player_a_baddie_support_pair_index,
        )
    }

    pub fn player_b_mg_main_name<'a>(&self, content: &'a GameContent) -> &'a str {
        lookup_name(&content.magical_girls, self.player_b_mg_main_index)
    }

    pub fn player_b_mg_support_names(&self, content: &GameContent) -> Vec<String> {
        support_names(
            &content.magical_girls,
            self.player_b_mg_main_index,
            self.player_b_mg_support_pair_index,
        )
    }

    pub fn player_b_baddie_main_name<'a>(&self, content: &'a GameContent) -> &'a str {
        lookup_name(&content.baddies, self.player_b_baddie_main_index)
    }

    pub fn player_b_baddie_support_names(&self, content: &GameContent) -> Vec<String> {
        support_names(
            &content.baddies,
            self.player_b_baddie_main_index,
            self.player_b_baddie_support_pair_index,
        )
    }
}

fn build_side(
    definitions: &[CharacterDefinition],
    main_index: usize,
    support_pair_index: usize,
) -> SideState {
    let main_definition = &definitions[main_index];
    let support_indices = support_pairs(definitions.len(), main_index)
        .get(support_pair_index)
        .cloned()
        .unwrap_or_default();

    SideState {
        main_character_id: main_definition.id.clone(),
        main: CharacterRuntimeState::from_definition(main_definition),
        supports: support_indices
            .into_iter()
            .enumerate()
            .map(|(index, support_index)| {
                let definition = &definitions[support_index];
                SupportState {
                    runtime: CharacterRuntimeState::from_definition(definition),
                    revealed: index == 0,
                }
            })
            .collect(),
    }
}

fn preferred_index(definitions: &[CharacterDefinition], preferred_id: &str) -> usize {
    definitions
        .iter()
        .position(|character| character.id == preferred_id)
        .unwrap_or(0)
}

fn required_index(definitions: &[CharacterDefinition], character_id: &str) -> usize {
    definitions
        .iter()
        .position(|character| character.id == character_id)
        .expect("starter loadout references missing character")
}

fn support_pair_index_for_main(
    definitions: &[CharacterDefinition],
    main_index: usize,
    preferred_ids: [&str; 2],
) -> usize {
    let pairs = support_pairs(definitions.len(), main_index);
    let mut preferred = preferred_ids
        .iter()
        .filter_map(|preferred_id| {
            definitions
                .iter()
                .position(|character| character.id == *preferred_id)
        })
        .collect::<Vec<_>>();
    // `support_pairs` emits each pair in ascending index order, so match on the
    // set rather than the order the ids were written in.
    preferred.sort_unstable();

    pairs
        .iter()
        .position(|pair| pair == &preferred)
        .unwrap_or(0)
}

fn support_pair_index_for_ids(
    definitions: &[CharacterDefinition],
    main_id: &str,
    support_ids: &[String],
) -> usize {
    let main_index = required_index(definitions, main_id);
    let mut required_pair = support_ids
        .iter()
        .map(|support_id| required_index(definitions, support_id))
        .collect::<Vec<_>>();
    // Supports arrive in selection order — the campaign hub appends them as the
    // player clicks — but `support_pairs` only ever emits ascending pairs. Sort
    // before matching so a pair chosen "backwards" resolves to the same index
    // instead of failing to match.
    required_pair.sort_unstable();
    let pairs = support_pairs(definitions.len(), main_index);

    pairs
        .iter()
        .position(|pair| pair == &required_pair)
        .expect("starter loadout references invalid support pair")
}

fn cycle_index(len: usize, current: usize) -> usize {
    if len == 0 {
        0
    } else {
        (current + 1) % len
    }
}

fn cycle_pair_index(
    definitions: &[CharacterDefinition],
    main_index: usize,
    current: usize,
) -> usize {
    let pair_count = support_pairs(definitions.len(), main_index).len();
    if pair_count == 0 {
        0
    } else {
        (current + 1) % pair_count
    }
}

fn reset_pair_index_if_needed(
    definitions: &[CharacterDefinition],
    main_index: usize,
    current: usize,
) -> usize {
    let pair_count = support_pairs(definitions.len(), main_index).len();
    if current < pair_count {
        current
    } else {
        0
    }
}

fn support_names(
    definitions: &[CharacterDefinition],
    main_index: usize,
    support_pair_index: usize,
) -> Vec<String> {
    let pair = support_pairs(definitions.len(), main_index)
        .get(support_pair_index)
        .cloned()
        .unwrap_or_default();

    pair.into_iter()
        .filter_map(|index| definitions.get(index))
        .map(|definition| definition.name.clone())
        .collect()
}

fn lookup_name(definitions: &[CharacterDefinition], index: usize) -> &str {
    definitions
        .get(index)
        .map(|character| character.name.as_str())
        .unwrap_or("Unknown")
}

fn support_pairs(len: usize, main_index: usize) -> Vec<Vec<usize>> {
    let candidates = (0..len)
        .filter(|index| *index != main_index)
        .collect::<Vec<_>>();
    let mut pairs = Vec::new();

    for first in 0..candidates.len() {
        for second in (first + 1)..candidates.len() {
            pairs.push(vec![candidates[first], candidates[second]]);
        }
    }

    pairs
}

fn draw_opening_hand(deck: &mut Vec<String>, count: usize) -> Vec<String> {
    let draw_count = count.min(deck.len());
    deck.drain(0..draw_count).collect()
}

fn default_player_a_deck() -> Vec<String> {
    vec![
        "quiet_lunch_on_the_rooftop",
        "false_sense_of_calm",
        "secret_rehearsal",
        "not_on_my_watch",
        "shared_umbrella",
        "emergency_costume_repair",
        "crescent_counterpose",
        "moonlit_rescue_chain",
        "break_the_formation",
        "identity_reveal",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn default_player_b_deck() -> Vec<String> {
    vec![
        "ominous_graffiti",
        "whisper_campaign",
        "twisted_headline",
        "expose_the_accomplice",
        "curtain_call_denied",
        "last_minute_cover_story",
        "panic_spiral",
        "panic_in_the_parade_route",
        "forbidden_bloom_of_calamity",
        "break_the_formation",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[cfg(test)]
#[path = "match_state_setup/tests.rs"]
mod tests;
