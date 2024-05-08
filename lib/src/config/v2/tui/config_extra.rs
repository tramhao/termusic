use std::{borrow::Cow, path::Path};

use anyhow::Result;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

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
        // Note that those are marked "private", but are used in the derives, and there is no to me known public way, but saves some implementation complexity
        let content =
            <serde::__private::de::Content<'_> as serde::Deserialize>::deserialize(deserializer)?;
        let deserializer = serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content);

        let mut err_res = String::new();

        match <TuiConfigVersioned<'a>>::deserialize(deserializer)
            .map(TuiConfigVersionedDefaulted::Versioned)
        {
            Ok(val) => return Ok(val),
            Err(err) => err_res.push_str(&format!("{err:#}")),
        }
        match TuiSettings::deserialize(deserializer).map(TuiConfigVersionedDefaulted::Unversioned) {
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
impl<'a> TuiConfigVersionedDefaulted<'a> {
    /// Read a config file, needs to be toml formatted
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut data: Self = Figment::new().merge(Toml::file(path)).extract()?;

        match data {
            TuiConfigVersionedDefaulted::Versioned(ref mut v) => v.resolve_com(path)?,
            TuiConfigVersionedDefaulted::Unversioned(ref mut v) => v.resolve_com(path)?,
        }

        Ok(data)
    }

    /// Save type used by the application as a config file
    ///
    /// Will only save the latest version
    pub fn save_file<P: AsRef<Path>>(path: P, config: &'a ApplicationType) -> Result<()> {
        // wrap the data in the latest version for saving
        let data = Self::Versioned(TuiConfigVersioned::V2(Cow::Borrowed(config)));
        std::fs::write(path, toml::to_string(&data)?)?;

        Ok(())
    }

    /// Convert Into the type used by the application, instead of what is parsed
    ///
    /// Will convert any version into the latest
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
    // Cow data so that we can use a reference for saving instead of cloning
    // Starting at Version 2 as the old format is referred as v1, but lives in a different config file name
    #[serde(rename = "2")]
    V2(Cow<'a, TuiSettings>),
}

impl TuiConfigVersioned<'_> {
    /// Convert Into the type used by the application, instead of what is parsed
    ///
    /// Will convert any version into the latest
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
