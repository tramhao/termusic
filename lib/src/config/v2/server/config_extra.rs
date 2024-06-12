use std::{borrow::Cow, path::Path};

use anyhow::{Context, Result};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::utils::get_app_config_path;

use super::ServerSettings;

/// The filename of the server config
pub const FILE_NAME: &str = "server.toml";

/// The type used by the application / the latest config version
///
/// This type exists so that it is easier to differentiate when the explicit type is meant, or later meant to be changed as a whole
type ApplicationType = ServerSettings;

/// Top-Level struct that wraps [`ServerConfigVersioned`] and a default version thereof if no `version` field exists.
///
/// This is required as serde does not have a concept of `default_tag` yet, see <https://github.com/serde-rs/serde/issues/2231>
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ServerConfigVersionedDefaulted<'a> {
    /// Case if config contains a `version` field
    Versioned(ServerConfigVersioned<'a>),
    /// Case if the config does not contain a `version` field, assume type of [`ServerConfigVersioned::V2`]
    Unversioned(ServerSettings),
}

// Manual implementation because deserialize "serde(untagged)" error are *really* bad
impl<'a, 'de> Deserialize<'de> for ServerConfigVersionedDefaulted<'a> {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Note that those are marked "private", but are used in the derives, and there is no to me known public way, but saves some implementation complexity
        let content =
            <serde::__private::de::Content<'_> as serde::Deserialize>::deserialize(deserializer)?;
        let deserializer = serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content);

        let mut err_res = String::new();

        match <ServerConfigVersioned<'a>>::deserialize(deserializer)
            .map(ServerConfigVersionedDefaulted::Versioned)
        {
            Ok(val) => return Ok(val),
            Err(err) => err_res.push_str(&format!("{err:#}")),
        }
        match ServerSettings::deserialize(deserializer)
            .map(ServerConfigVersionedDefaulted::Unversioned)
        {
            Ok(val) => return Ok(val),
            // no need to check if "err_res" is empty, as this code can only be executed if the above has failed
            Err(err) => {
                let err_str = err.to_string();
                // only add if the error is different; otherwise you get duplicated errors
                if err_str != err_res {
                    err_res.push_str(&format!("\n{err_str:#}"));
                }
            }
        }

        Err(<D::Error as serde::de::Error>::custom(err_res))
    }
}

// Note: for saving, see
impl<'a> ServerConfigVersionedDefaulted<'a> {
    /// Read a config file, needs to be toml formatted
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        {
            let v1_config_path = path
                .parent()
                .context("expected server config path to have a parent")?
                .join(super::super::super::v1::FILE_NAME);
            if !path.exists() && v1_config_path.exists() {
                info!("New config file does not exist, but old one does exist.");
                return Self::migrate_from_v1(&v1_config_path, path);
            }
        }

        let data: Self = Figment::new().merge(Toml::file(path)).extract()?;

        Ok(data)
    }

    /// Read a config file from the default set app-path
    pub fn from_config_path() -> Result<Self> {
        let server_config_path = get_app_config_path()?.join(FILE_NAME);

        Self::from_file(server_config_path)
    }

    /// Load the old settings, then transform them into the new settings
    // public in config_v2 module so that the TUI can migrate the server config before itself
    pub(in super::super) fn migrate_from_v1(_v1_path: &Path, v2_path: &Path) -> Result<Self> {
        use super::super::super::v1::Settings;

        info!("Migrating server config from v1 format to v2");

        let old_settings = {
            let mut settings = Settings::default();
            settings.load()?;

            settings
        };

        let new_settings = ServerSettings::try_from(old_settings)?;

        // save the file directly to not have to re-do the convertion again, even if config does not change
        Self::save_file(v2_path, &new_settings)?;

        Ok(Self::Unversioned(new_settings))
    }

    /// Save type used by the application as a config file
    ///
    /// Will only save the latest version
    pub fn save_file<'b, P: AsRef<Path>>(path: P, config: &'b ApplicationType) -> Result<()> {
        // wrap the data in the latest version for saving
        let data = ServerConfigVersionedDefaulted::<'b>::Versioned(ServerConfigVersioned::V2(
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
    pub fn into_settings(self) -> ApplicationType {
        let versioned = match self {
            ServerConfigVersionedDefaulted::Versioned(versioned) => versioned,
            ServerConfigVersionedDefaulted::Unversioned(v) => return v,
        };

        versioned.into_settings()
    }
}

/// Enum that contains all versions for the server config
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "version")]
pub enum ServerConfigVersioned<'a> {
    /// V2 is considered stable / non-backwards breaking changes allowed after b0daaddf58ec7f46b364fcf999c50d3aa043308f
    // Cow data so that we can use a reference for saving instead of cloning
    // Starting at Version 2 as the old format is referred as v1, but lives in a different config file name
    #[serde(rename = "2")]
    V2(Cow<'a, ServerSettings>),
}

impl ServerConfigVersioned<'_> {
    /// Convert Into the type used by the application, instead of what is parsed
    ///
    /// Will convert any version into the latest
    pub fn into_settings(self) -> ApplicationType {
        match self {
            ServerConfigVersioned::V2(v) => v.into_owned(),
        }
    }
}
