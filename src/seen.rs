use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SeenStatus {
    seen_items: HashSet<String>,
}

impl SeenStatus {
    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let status: SeenStatus = serde_yaml::from_str(&content)?;
        Ok(status)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let yaml = serde_yaml::to_string(&self)?;
        fs::write(&path, yaml)?;
        Ok(())
    }

    pub fn mark_seen(&mut self, identifier: &str) {
        self.seen_items.insert(identifier.to_string());
    }

    pub fn is_seen(&self, identifier: &str) -> bool {
        self.seen_items.contains(identifier)
    }

    fn get_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".miam").join("seen.yml"))
    }
}

pub fn get_item_identifier(link: &Option<String>, title: &str) -> String {
    link.clone().unwrap_or_else(|| title.to_string())
}
