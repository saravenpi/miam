use crate::feed::FeedSource;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const INVIDIOUS_INSTANCES: &[&str] = &[
    "yewtu.be",
    "vid.puffyan.us",
    "invidious.flokinet.to",
    "invidious.privacydev.net",
    "iv.melmac.space",
];

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub invidious: bool,
    #[serde(default)]
    pub invidious_instance: Option<String>,
    #[serde(default = "default_show_tooltips")]
    pub show_tooltips: bool,
}

fn default_show_tooltips() -> bool {
    true
}

pub struct Config {
    pub sources: Vec<FeedSource>,
    pub settings: Settings,
}

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    feeds: IndexMap<String, String>,
    #[serde(default)]
    settings: Settings,
}

impl Config {
    fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|p| p.join(".miam.yml"))
    }

    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let content = fs::read_to_string(path).ok()?;

        if let Ok(config_file) = serde_yaml::from_str::<ConfigFile>(&content) {
            let sources = config_file
                .feeds
                .into_iter()
                .map(|(name, url)| FeedSource { name, url })
                .collect();
            return Some(Config {
                sources,
                settings: config_file.settings,
            });
        }

        let map: IndexMap<String, String> = serde_yaml::from_str(&content).ok()?;
        let sources = map
            .into_iter()
            .map(|(name, url)| FeedSource { name, url })
            .collect();
        Some(Config {
            sources,
            settings: Settings::default(),
        })
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            let feeds: IndexMap<String, String> = self
                .sources
                .iter()
                .map(|s| (s.name.clone(), s.url.clone()))
                .collect();
            let config_file = ConfigFile {
                feeds,
                settings: Settings {
                    invidious: self.settings.invidious,
                    invidious_instance: self.settings.invidious_instance.clone(),
                    show_tooltips: self.settings.show_tooltips,
                },
            };
            if let Ok(content) = serde_yaml::to_string(&config_file) {
                let _ = fs::write(path, content);
            }
        }
    }

    pub fn get_invidious_instance(&self) -> &str {
        self.settings
            .invidious_instance
            .as_deref()
            .unwrap_or(INVIDIOUS_INSTANCES[0])
    }
}
