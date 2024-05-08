use std::{borrow::Cow, path::Path};

use anyhow::Result;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use super::ServerSettings;

/// The type used by the application / the latest config version
///
/// This type exists so that it is easier to differentiate when the explicit type is meant, or later meant to be changed as a whole
type ApplicationType = ServerSettings;

// TODO: implement a custom deserializer instead of "serde(untagged)" because of VERY bad errors, see https://github.com/serde-rs/serde/pull/1544

/// Top-Level struct that wraps [`ServerConfigVersioned`] and a default version thereof if no `version` field exists.
///
/// This is required as serde does not have a concept of `default_tag` yet, see <https://github.com/serde-rs/serde/issues/2231>
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ServerConfigVersionedDefaulted<'a> {
    /// Case if config contains a `version` field
    Versioned(ServerConfigVersioned<'a>),
    /// Case if the config does not contain a `version` field, assume type of [`ServerConfigVersioned::V2`]
    Unversioned(ServerSettings),
}

// Note: for saving, see
impl<'a> ServerConfigVersionedDefaulted<'a> {
    /// Read a config file, needs to be toml formatted
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data: Self = Figment::new().merge(Toml::file(path)).extract()?;

        Ok(data)
    }

    /// Save type used by the application as a config file
    ///
    /// Will only save the latest version
    pub fn save_file<P: AsRef<Path>>(path: P, config: &'a ApplicationType) -> Result<()> {
        // wrap the data in the latest version for saving
        let data = Self::Versioned(ServerConfigVersioned::V2(Cow::Borrowed(config)));
        std::fs::write(path, toml::to_string(&data)?)?;

        Ok(())
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
