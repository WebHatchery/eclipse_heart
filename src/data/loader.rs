//! JSON loading helpers.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::data::{
    ArtCatalog, CampaignDefinition, CardVisualSpec, CharacterDefinition, DeckRules, MatchRules,
    ProgressionRules, StarterLoadout, StoryCardDefinition, UiText,
};

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to load asset: {0}")]
    Asset(String),
}

#[derive(Clone, Debug)]
pub struct GameContent {
    pub magical_girls: Vec<CharacterDefinition>,
    pub baddies: Vec<CharacterDefinition>,
    pub story_cards: Vec<StoryCardDefinition>,
    pub rules: MatchRules,
    pub deck_rules: DeckRules,
    pub progression_rules: ProgressionRules,
    pub starter_loadouts: Vec<StarterLoadout>,
    pub campaign: CampaignDefinition,
    pub card_visuals: CardVisualSpec,
    pub art_catalog: ArtCatalog,
}

impl Default for GameContent {
    fn default() -> Self {
        Self {
            magical_girls: Vec::new(),
            baddies: Vec::new(),
            story_cards: Vec::new(),
            rules: MatchRules::default(),
            deck_rules: DeckRules::default(),
            progression_rules: ProgressionRules::default(),
            starter_loadouts: Vec::new(),
            campaign: CampaignDefinition::default(),
            card_visuals: CardVisualSpec {
                canvas: crate::data::CardCanvas {
                    width: 0,
                    height: 0,
                    safe_margin: 0,
                },
                zones: Vec::new(),
                template_families: Vec::new(),
                speed_badges: Vec::new(),
            },
            art_catalog: ArtCatalog::default(),
        }
    }
}

impl GameContent {
    pub fn load() -> Result<Self, LoadError> {
        let magical_girls = load_json::<Vec<CharacterDefinition>>(asset_path(
            "assets/data/magical_girls/prototype_set.json",
        ))?;
        let baddies = load_json::<Vec<CharacterDefinition>>(asset_path(
            "assets/data/baddies/prototype_set.json",
        ))?;
        let story_cards = load_json::<Vec<StoryCardDefinition>>(asset_path(
            "assets/data/story_cards/prototype_set.json",
        ))?;
        let rules = load_json::<MatchRules>(asset_path("assets/data/rules/match_rules.json"))?;
        let deck_rules = load_json::<DeckRules>(asset_path("assets/data/rules/deck_rules.json"))?;
        let progression_rules =
            load_json::<ProgressionRules>(asset_path("assets/data/rules/progression_rules.json"))?;
        let starter_loadouts = load_json::<Vec<StarterLoadout>>(asset_path(
            "assets/data/starter_loadouts/prototype_starters.json",
        ))?;
        let campaign = load_json::<CampaignDefinition>(asset_path(
            "assets/data/campaigns/magical_girl_campaign.json",
        ))?;
        let card_visuals =
            load_json::<CardVisualSpec>(asset_path("assets/data/card_visuals.json"))?;
        let art_catalog = load_json::<ArtCatalog>(asset_path("assets/data/art_catalog.json"))?;
        Ok(Self {
            magical_girls,
            baddies,
            story_cards,
            rules,
            deck_rules,
            progression_rules,
            starter_loadouts,
            campaign,
            card_visuals,
            art_catalog,
        })
    }

    pub async fn load_async() -> Result<Self, LoadError> {
        let magical_girls = load_json_async::<Vec<CharacterDefinition>>(
            "assets/data/magical_girls/prototype_set.json",
        )
        .await?;
        let baddies =
            load_json_async::<Vec<CharacterDefinition>>("assets/data/baddies/prototype_set.json")
                .await?;
        let story_cards = load_json_async::<Vec<StoryCardDefinition>>(
            "assets/data/story_cards/prototype_set.json",
        )
        .await?;
        let rules = load_json_async::<MatchRules>("assets/data/rules/match_rules.json").await?;
        let deck_rules = load_json_async::<DeckRules>("assets/data/rules/deck_rules.json").await?;
        let progression_rules =
            load_json_async::<ProgressionRules>("assets/data/rules/progression_rules.json").await?;
        let starter_loadouts = load_json_async::<Vec<StarterLoadout>>(
            "assets/data/starter_loadouts/prototype_starters.json",
        )
        .await?;
        let campaign = load_json_async::<CampaignDefinition>(
            "assets/data/campaigns/magical_girl_campaign.json",
        )
        .await?;
        let card_visuals =
            load_json_async::<CardVisualSpec>("assets/data/card_visuals.json").await?;
        let art_catalog = load_json_async::<ArtCatalog>("assets/data/art_catalog.json").await?;
        Ok(Self {
            magical_girls,
            baddies,
            story_cards,
            rules,
            deck_rules,
            progression_rules,
            starter_loadouts,
            campaign,
            card_visuals,
            art_catalog,
        })
    }
}

impl UiText {
    pub fn load() -> Result<Self, LoadError> {
        let values = load_json::<HashMap<String, String>>(asset_path("assets/data/ui_text.json"))?;
        Ok(Self { values })
    }

    pub async fn load_async() -> Result<Self, LoadError> {
        let values = load_json_async::<HashMap<String, String>>("assets/data/ui_text.json").await?;
        Ok(Self { values })
    }
}

fn load_json<T>(path: PathBuf) -> Result<T, LoadError>
where
    T: DeserializeOwned,
{
    let text = std::fs::read_to_string(path)?;
    let value = serde_json::from_str::<T>(&text)?;
    Ok(value)
}

async fn load_json_async<T>(relative_path: &str) -> Result<T, LoadError>
where
    T: DeserializeOwned,
{
    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::data_loader::load_json_file(relative_path)
            .await
            .map_err(LoadError::Asset)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        load_json(asset_path(relative_path))
    }
}

fn asset_path(relative_path: &str) -> PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from(relative_path)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        macroquad_toolkit::data_loader::manifest_relative_path(
            env!("CARGO_MANIFEST_DIR"),
            relative_path,
        )
    }
}

#[cfg(test)]
mod tests;
