use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use suzu_platform::WindowConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameConfig {
    pub window: WindowConfig,
    pub script_entry: String,
    #[serde(default)]
    pub title_screen: TitleScreenConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            script_entry: "script/main.szs".to_owned(),
            title_screen: TitleScreenConfig::default(),
        }
    }
}

impl GameConfig {
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        read_json(path)
    }

    pub fn write_json_file(&self, path: impl AsRef<Path>) -> Result<()> {
        write_json(path, self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleScreenConfig {
    pub enabled: bool,
    pub title: String,
    pub subtitle: String,
    #[serde(default)]
    pub background_texture: Option<String>,
    #[serde(default)]
    pub labels: TitleScreenLabels,
}

impl Default for TitleScreenConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            title: "Project Suzu".to_owned(),
            subtitle: "Galgame Engine".to_owned(),
            background_texture: None,
            labels: TitleScreenLabels::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitleScreenLabels {
    pub menu_heading: String,
    pub load_heading: String,
    pub settings_heading: String,
    pub start: String,
    pub continue_game: String,
    pub load: String,
    pub settings: String,
    pub quit: String,
    pub back: String,
    pub autosave: String,
    pub empty_slot: String,
}

impl Default for TitleScreenLabels {
    fn default() -> Self {
        Self {
            menu_heading: "Title".to_owned(),
            load_heading: "Load Game".to_owned(),
            settings_heading: "Settings".to_owned(),
            start: "Start".to_owned(),
            continue_game: "Continue".to_owned(),
            load: "Load".to_owned(),
            settings: "Settings".to_owned(),
            quit: "Quit".to_owned(),
            back: "Back".to_owned(),
            autosave: "Autosave".to_owned(),
            empty_slot: "Empty".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserSettings {
    pub audio: AudioSettings,
    pub text: TextSettings,
    pub window: WindowSettings,
}

impl UserSettings {
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        read_json(path)
    }

    pub fn write_json_file(&self, path: impl AsRef<Path>) -> Result<()> {
        write_json(path, self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub voice_volume: f32,
    pub se_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            bgm_volume: 1.0,
            voice_volume: 1.0,
            se_volume: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSettings {
    pub speed_chars_per_second: f32,
    pub auto_advance_delay_ms: u32,
}

impl Default for TextSettings {
    fn default() -> Self {
        Self {
            speed_chars_per_second: 60.0,
            auto_advance_delay_ms: 1200,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowSettings {
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
        }
    }
}

fn read_json<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let path = path.as_ref();
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&source).with_context(|| format!("failed to parse {}", path.display()))
}

fn write_json<T>(path: impl AsRef<Path>, value: &T) -> Result<()>
where
    T: Serialize,
{
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

pub fn default_user_settings_path() -> PathBuf {
    PathBuf::from("settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_config_round_trips_json() {
        let path = test_path("suzu-game-config.json");
        let config = GameConfig {
            window: WindowConfig {
                title: "Suzu Test".to_owned(),
                logical_size: suzu_core::Vec2::new(800.0, 600.0),
                resizable: false,
            },
            script_entry: "script/prologue.szs".to_owned(),
            title_screen: TitleScreenConfig {
                enabled: true,
                title: "Test Title".to_owned(),
                subtitle: "Subtitle".to_owned(),
                background_texture: Some("title_bg".to_owned()),
                labels: TitleScreenLabels::default(),
            },
        };

        config.write_json_file(&path).unwrap();
        let restored = GameConfig::from_json_file(&path).unwrap();

        assert_eq!(restored, config);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn user_settings_round_trips_json() {
        let path = test_path("suzu-user-settings.json");
        let settings = UserSettings {
            audio: AudioSettings {
                master_volume: 0.8,
                bgm_volume: 0.6,
                voice_volume: 0.7,
                se_volume: 0.5,
            },
            text: TextSettings {
                speed_chars_per_second: 90.0,
                auto_advance_delay_ms: 800,
            },
            window: WindowSettings {
                fullscreen: true,
                vsync: false,
            },
        };

        settings.write_json_file(&path).unwrap();
        let restored = UserSettings::from_json_file(&path).unwrap();

        assert_eq!(restored, settings);
        let _ = fs::remove_file(path);
    }

    fn test_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        path
    }
}
