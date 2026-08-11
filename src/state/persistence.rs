//! Local save loading and writing.

use std::path::PathBuf;

use thiserror::Error;

use super::{CampaignSaveBundle, CollectionSave, DecksSave, ProfileSave, SettingsSave};
use macroquad_toolkit::persistence::SaveRoot;

const PROFILE_FILE: &str = "profile.json";
const COLLECTION_FILE: &str = "collection.json";
const DECKS_FILE: &str = "decks.json";
const CAMPAIGNS_FILE: &str = "campaigns.json";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Clone, Debug, Default)]
pub struct PersistenceBundle {
    pub profile: ProfileSave,
    pub collection: CollectionSave,
    pub decks: DecksSave,
    pub campaigns: CampaignSaveBundle,
    pub settings: SettingsSave,
}

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("failed to access save path: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse save json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to access save bundle: {0}")]
    Storage(String),
}

#[derive(Clone, Debug)]
pub struct PersistenceManager {
    root: SaveRoot,
}

impl PersistenceManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: SaveRoot::new("eclipse_heart", root.into()),
        }
    }

    pub fn default_local() -> Self {
        Self::new("save")
    }

    pub fn load_all(&self) -> Result<PersistenceBundle, PersistenceError> {
        Ok(PersistenceBundle {
            profile: self
                .root
                .load_json_or_default(PROFILE_FILE)
                .map_err(PersistenceError::Storage)?,
            collection: self
                .root
                .load_json_or_default(COLLECTION_FILE)
                .map_err(PersistenceError::Storage)?,
            decks: self
                .root
                .load_json_or_default(DECKS_FILE)
                .map_err(PersistenceError::Storage)?,
            campaigns: self
                .root
                .load_json_or_default(CAMPAIGNS_FILE)
                .map_err(PersistenceError::Storage)?,
            settings: self
                .root
                .load_json_or_default(SETTINGS_FILE)
                .map_err(PersistenceError::Storage)?,
        })
    }

    pub fn save_all(&self, bundle: &PersistenceBundle) -> Result<(), PersistenceError> {
        self.root
            .save_json(PROFILE_FILE, &bundle.profile)
            .map_err(PersistenceError::Storage)?;
        self.root
            .save_json(COLLECTION_FILE, &bundle.collection)
            .map_err(PersistenceError::Storage)?;
        self.root
            .save_json(DECKS_FILE, &bundle.decks)
            .map_err(PersistenceError::Storage)?;
        self.root
            .save_json(CAMPAIGNS_FILE, &bundle.campaigns)
            .map_err(PersistenceError::Storage)?;
        self.root
            .save_json(SETTINGS_FILE, &bundle.settings)
            .map_err(PersistenceError::Storage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
