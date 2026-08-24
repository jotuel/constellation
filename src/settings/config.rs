use cosmic_config::{CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// Schema version of the COSMIC config entry. Bump whenever `Config` changes
/// shape; older stored entries still load (missing keys fall back to
/// `Default`), and the next save rewrites them in the new schema.
pub const CONFIG_VERSION: u64 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, CosmicConfigEntry)]
#[version = 2]
pub struct Config {
    pub show_sync_indicator: bool,
    pub send_typing_notifications: bool,
    pub render_markdown: bool,
    pub compact_mode: bool,
    pub hide_threaded_messages: bool,
    pub media_previews_display_policy: bool,
    pub invite_avatars_display_policy: bool,
    pub autoplay_videos: bool,
    pub sidebar_ratio: f32,
    /// User-rebindable keyboard shortcut overrides. Only entries that differ
    /// from the shipped default are present; an empty serialized binding
    /// marks an action as intentionally unbound.
    pub key_bindings: HashMap<
        crate::constellation::keybind::ShortcutAction,
        crate::constellation::keybind::SerializedKeyBind,
    >,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_sync_indicator: false,
            send_typing_notifications: false,
            render_markdown: false,
            compact_mode: false,
            hide_threaded_messages: true,
            media_previews_display_policy: true,
            invite_avatars_display_policy: true,
            autoplay_videos: true,
            sidebar_ratio: 0.25,
            key_bindings: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Ok(config_handler) =
            cosmic_config::Config::new("fi.joonastuomi.Constellation", CONFIG_VERSION)
        {
            match Self::get_entry(&config_handler) {
                Ok(config) => config,
                Err((errors, fallback)) => {
                    for err in errors {
                        tracing::warn!("Failed to load config from COSMIC Config: {:?}", err);
                    }
                    fallback
                }
            }
        } else {
            tracing::warn!("Failed to create COSMIC Config handler, using default config");
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        if let Ok(config_handler) =
            cosmic_config::Config::new("fi.joonastuomi.Constellation", CONFIG_VERSION)
        {
            self.write_entry(&config_handler)
                .map_err(|e| format!("Failed to save config to COSMIC Config: {:?}", e))
        } else {
            Err("Failed to create COSMIC Config handler".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_serialization() -> Result<(), serde_json::Error> {
        let config = Config {
            show_sync_indicator: true,
            send_typing_notifications: true,
            render_markdown: true,
            compact_mode: true,
            hide_threaded_messages: true,
            media_previews_display_policy: false,
            invite_avatars_display_policy: false,
            autoplay_videos: false,
            sidebar_ratio: 0.30,
            key_bindings: HashMap::new(),
        };

        let serialized = serde_json::to_string(&config)?;
        let deserialized: Config = serde_json::from_str(&serialized)?;

        assert_eq!(config, deserialized);
        Ok(())
    }

    #[test]
    #[serial_test::serial]
    fn test_config_save_load() {
        let tmp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("HOME", tmp_dir.path());
            std::env::set_var("XDG_CONFIG_HOME", tmp_dir.path());
            std::env::set_var("APPDATA", tmp_dir.path());
        }

        let config = Config {
            show_sync_indicator: true,
            ..Default::default()
        };

        config.save().expect("Failed to save config");

        let loaded = Config::load();
        assert_eq!(config, loaded);
    }

    #[test]
    #[serial_test::serial]
    fn test_config_load_nonexistent() {
        let tmp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("HOME", tmp_dir.path());
            std::env::set_var("XDG_CONFIG_HOME", tmp_dir.path());
            std::env::set_var("APPDATA", tmp_dir.path());
        }

        let loaded = Config::load();
        assert_eq!(loaded, Config::default());
    }

    #[test]
    fn test_key_bindings_serialization_round_trip() {
        use crate::constellation::keybind::{SerializedKeyBind, ShortcutAction};

        let kb = ShortcutAction::ToggleSpaceSettings
            .default_keybind()
            .unwrap();
        let mut overrides = HashMap::new();
        overrides.insert(ShortcutAction::ToggleSpaceSettings, (&kb).into());

        let config = Config {
            key_bindings: overrides.clone(),
            ..Default::default()
        };
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            deserialized
                .key_bindings
                .get(&ShortcutAction::ToggleSpaceSettings),
            Some(&SerializedKeyBind::from(&kb))
        );
    }
}
