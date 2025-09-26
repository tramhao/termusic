use std::{borrow::Cow, fmt::Write as _, path::Path};

use anyhow::Result;
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::{Deserialize, Serialize};

use crate::utils::get_app_config_path;

use super::TuiSettings;

/// The filename of the tui config
pub const FILE_NAME: &str = "tui.toml";

/// The type used by the application / the latest config version
///
/// This type exists so that it is easier to differentiate when the explicit type is meant, or later meant to be changed as a whole
type ApplicationType = TuiSettings;

/// Top-Level struct that wraps [`TuiConfigVersioned`] and a default version thereof if no `version` field exists.
///
/// This is required as serde does not have a concept of `default_tag` yet, see <https://github.com/serde-rs/serde/issues/2231>
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum TuiConfigVersionedDefaulted<'a> {
    /// Case if config contains a `version` field
    Versioned(TuiConfigVersioned<'a>),
    /// Case if the config does not contain a `version` field, assume type of [`TuiConfigVersioned::V2`]
    Unversioned(TuiSettings),
}

// Manual implementation because deserialize "serde(untagged)" error are *really* bad
impl<'a, 'de> Deserialize<'de> for TuiConfigVersionedDefaulted<'a> {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let content = serde_content::Value::deserialize(deserializer)?;
        let deserializer = serde_content::Deserializer::new(content);

        let mut err_res = String::new();

        match <TuiConfigVersioned<'a>>::deserialize(deserializer.clone())
            .map(TuiConfigVersionedDefaulted::Versioned)
        {
            Ok(val) => return Ok(val),
            Err(err) => {
                let _ = write!(err_res, "{err:#}");
            }
        }
        match Intermediate::deserialize(deserializer)
            .map(|v| TuiConfigVersionedDefaulted::Unversioned(v.into_settings()))
        {
            Ok(val) => return Ok(val),
            // no need to check if "err_res" is empty, as this code can only be executed if the above has failed
            Err(err) => {
                let err_str = err.to_string();
                // only add if the error is different; otherwise you get duplicated errors
                if err_str != err_res {
                    let _ = write!(err_res, "\n{err_str:#}");
                }
            }
        }

        Err(<D::Error as serde::de::Error>::custom(err_res))
    }
}

// Note: for saving, see
impl TuiConfigVersionedDefaulted<'_> {
    /// Read a config file, needs to be toml formatted
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut data: Option<Self> = None;
        {
            use anyhow::Context;
            let v1_config_path = path
                .parent()
                .context("expected tui config path to have a parent")?
                .join(super::super::super::v1::FILE_NAME);
            if !path.exists() && v1_config_path.exists() {
                info!("New config file does not exist, but old one does exist.");
                data = Some(Self::migrate_from_v1(&v1_config_path, path)?);
            }
        }

        let mut data: Self = if let Some(data) = data {
            data
        } else if !path.exists() {
            // if path does not exist, create default instance and save it
            let config = TuiSettings::default();
            Self::save_file(path, &config)?;
            Self::Unversioned(config)
        } else {
            Figment::new().merge(Toml::file(path)).extract()?
        };

        match data {
            TuiConfigVersionedDefaulted::Versioned(ref mut v) => v.resolve_com(path)?,
            TuiConfigVersionedDefaulted::Unversioned(ref mut v) => v.resolve_com(path)?,
        }

        Ok(data)
    }

    /// Read a config file from the default set app-path
    pub fn from_config_path() -> Result<Self> {
        let tui_config_path = get_app_config_path()?.join(FILE_NAME);

        Self::from_file(tui_config_path)
    }

    /// Load the old settings, then transform them into the new settings (both server & TUI)
    ///
    /// expects that `v2_path` has a parent
    fn migrate_from_v1(v1_path: &Path, v2_path: &Path) -> Result<Self> {
        use super::super::super::v1::Settings;

        // do the server config first as the TUI would try to load the server config to resolve the com-settings later, also to speed it up instead of having the server do it later
        use super::super::server::config_extra as server_config_extra;
        // "unwrap" here is safe as we expect the calling function to have already checked it
        let v2_server_path = v2_path
            .parent()
            .unwrap()
            .join(server_config_extra::FILE_NAME);

        if !v2_server_path.exists() {
            server_config_extra::ServerConfigVersionedDefaulted::migrate_from_v1(v1_path, v2_path)?;
        }

        info!("Migrating tui config from v1 format to v2");

        let old_settings = {
            let mut settings = Settings::default();
            settings.load()?;

            settings
        };

        let new_settings = TuiSettings::from(old_settings);

        // save the file directly to not have to re-do the convertion again, even if config does not change
        Self::save_file(v2_path, &new_settings)?;

        Ok(Self::Unversioned(new_settings))
    }

    /// Save type used by the application as a config file
    ///
    /// Will only save the latest version
    pub fn save_file<'b, P: AsRef<Path>>(path: P, config: &'b ApplicationType) -> Result<()> {
        // wrap the data in the latest version for saving
        let data = TuiConfigVersionedDefaulted::<'b>::Versioned(TuiConfigVersioned::V2(
            Cow::Borrowed(config),
        ));
        std::fs::write(path, toml::to_string(&data)?)?;

        Ok(())
    }

    /// Save the given config to the default set app-path
    pub fn save_config_path(config: &ApplicationType) -> Result<()> {
        let server_config_path = get_app_config_path()?.join(FILE_NAME);

        Self::save_file(server_config_path, config)
    }

    /// Convert Into the type used by the application, instead of what is parsed
    ///
    /// Will convert any version into the latest
    #[must_use]
    pub fn into_settings(self) -> ApplicationType {
        let versioned = match self {
            TuiConfigVersionedDefaulted::Versioned(versioned) => versioned,
            TuiConfigVersionedDefaulted::Unversioned(v) => return v,
        };

        versioned.into_settings()
    }
}

/// Enum that contains all versions for the tui config
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "version")]
pub enum TuiConfigVersioned<'a> {
    /// V2 is considered stable / non-backwards breaking changes allowed after b0daaddf58ec7f46b364fcf999c50d3aa043308f
    // Cow data so that we can use a reference for saving instead of cloning
    // Starting at Version 2 as the old format is referred as v1, but lives in a different config file name
    #[serde(rename = "2")]
    V2(Cow<'a, TuiSettings>),
}

impl TuiConfigVersioned<'_> {
    /// Convert Into the type used by the application, instead of what is parsed
    ///
    /// Will convert any version into the latest
    #[must_use]
    pub fn into_settings(self) -> ApplicationType {
        match self {
            TuiConfigVersioned::V2(v) => v.into_owned(),
        }
    }

    /// Resolve the TUI's `com` Settings, depending on version
    fn resolve_com(&mut self, tui_path: &Path) -> Result<()> {
        match self {
            TuiConfigVersioned::V2(v) => v.to_mut().resolve_com(tui_path),
        }
    }
}

/// This type exists due to a bug in `serde-content`, where without it fails to properly parse untagged enum values
/// see <https://github.com/rushmorem/serde-content/issues/27>.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Intermediate<'a> {
    Some(Cow<'a, ApplicationType>),
}

impl Intermediate<'_> {
    /// Convert Into the type used by the application, instead of what is parsed
    ///
    /// Will convert any version into the latest
    #[must_use]
    pub fn into_settings(self) -> ApplicationType {
        match self {
            Intermediate::Some(v) => v.into_owned(),
        }
    }
}
