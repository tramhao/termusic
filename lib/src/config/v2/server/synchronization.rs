use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};

/// Settings for the periodic podcast synchronization task.
///
/// This is a **top-level** config section in `ServerSettings` (not nested under `podcast`)
/// because synchronization is a server-level scheduling concern that orchestrates multiple
/// subsystems (feed fetching, downloading, playlist management). Nesting under `podcast`
/// would incorrectly imply it is solely a podcast configuration concern (AC-06, SCENARIO-030).
///
/// When absent from the config file, all fields use their defaults
/// due to `#[serde(default)]` on the struct.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct SynchronizationSettings {
    /// Whether automatic podcast synchronization is enabled.
    /// Default: true
    pub enable: bool,

    /// How often to check all subscribed feeds for new episodes.
    /// Accepts human-readable duration strings: "1h", "30m", "2h30m".
    /// Default: "1h" (3600 seconds)
    #[serde(with = "humantime_serde")]
    pub interval: Duration,

    /// Whether to run a full sync immediately on server startup
    /// before entering the periodic cycle.
    /// Default: true
    pub refresh_on_startup: bool,

    /// Maximum number of new episodes to download per podcast per sync pass.
    /// Only the newest N undownloaded episodes are fetched.
    /// Set to 0 for unlimited (download all missing episodes).
    /// Default: 5
    pub max_new_episodes: u32,

    /// Whether to automatically add downloaded episodes to the playlist.
    /// Default: true (backward compatible — existing users see no behavior change).
    pub auto_enqueue: bool,
}

impl Default for SynchronizationSettings {
    fn default() -> Self {
        Self {
            enable: true,
            interval: Duration::from_secs(3600),
            refresh_on_startup: true,
            max_new_episodes: 5,
            auto_enqueue: true,
        }
    }
}

/// Deserialization helper that supports both flat fields and a `[synchronization]` wrapper.
/// When parsed within `ServerSettings`, fields arrive flat. When parsed standalone from
/// a TOML document with a `[synchronization]` section header, fields arrive nested.
#[derive(Deserialize)]
#[serde(default)]
struct SyncSettingsHelper {
    enable: bool,
    #[serde(with = "humantime_serde")]
    interval: Duration,
    refresh_on_startup: bool,
    max_new_episodes: u32,
    auto_enqueue: bool,
    synchronization: Option<SyncSettingsFlat>,
}

/// Flat representation used when TOML has a `[synchronization]` section header.
#[derive(Deserialize)]
#[serde(default)]
struct SyncSettingsFlat {
    enable: bool,
    #[serde(with = "humantime_serde")]
    interval: Duration,
    refresh_on_startup: bool,
    max_new_episodes: u32,
    auto_enqueue: bool,
}

impl Default for SyncSettingsHelper {
    fn default() -> Self {
        let d = SynchronizationSettings::default();
        Self {
            enable: d.enable,
            interval: d.interval,
            refresh_on_startup: d.refresh_on_startup,
            max_new_episodes: d.max_new_episodes,
            auto_enqueue: d.auto_enqueue,
            synchronization: None,
        }
    }
}

impl Default for SyncSettingsFlat {
    fn default() -> Self {
        let d = SynchronizationSettings::default();
        Self {
            enable: d.enable,
            interval: d.interval,
            refresh_on_startup: d.refresh_on_startup,
            max_new_episodes: d.max_new_episodes,
            auto_enqueue: d.auto_enqueue,
        }
    }
}

impl<'de> Deserialize<'de> for SynchronizationSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = SyncSettingsHelper::deserialize(deserializer)?;
        if let Some(nested) = helper.synchronization {
            Ok(Self {
                enable: nested.enable,
                interval: nested.interval,
                refresh_on_startup: nested.refresh_on_startup,
                max_new_episodes: nested.max_new_episodes,
                auto_enqueue: nested.auto_enqueue,
            })
        } else {
            Ok(Self {
                enable: helper.enable,
                interval: helper.interval,
                refresh_on_startup: helper.refresh_on_startup,
                max_new_episodes: helper.max_new_episodes,
                auto_enqueue: helper.auto_enqueue,
            })
        }
    }
}
