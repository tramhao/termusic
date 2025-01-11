use serde::Deserialize;

/// A Theme parsed from a theme file
#[derive(Debug, Deserialize, PartialEq)]
pub struct YAMLTheme {
    pub colors: YAMLThemeColors,
}

type YAMLThemeColor = String;

#[derive(Debug, Deserialize, PartialEq)]
pub struct YAMLThemeColors {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub primary: YAMLThemePrimary,
    #[serde(default)]
    pub cursor: YAMLThemeCursor,
    #[serde(default)]
    pub normal: YAMLThemeNormal,
    #[serde(default)]
    pub bright: YAMLThemeBright,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct YAMLThemePrimary {
    pub background: YAMLThemeColor,
    pub foreground: YAMLThemeColor,
}

impl Default for YAMLThemePrimary {
    fn default() -> Self {
        Self {
            background: default_000(),
            foreground: default_fff(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct YAMLThemeCursor {
    pub text: YAMLThemeColor,
    pub cursor: YAMLThemeColor,
}

impl Default for YAMLThemeCursor {
    fn default() -> Self {
        Self {
            text: default_fff(),
            cursor: default_fff(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct YAMLThemeNormal {
    pub black: YAMLThemeColor,
    pub red: YAMLThemeColor,
    pub green: YAMLThemeColor,
    pub yellow: YAMLThemeColor,
    pub blue: YAMLThemeColor,
    pub magenta: YAMLThemeColor,
    pub cyan: YAMLThemeColor,
    pub white: YAMLThemeColor,
}

impl Default for YAMLThemeNormal {
    fn default() -> Self {
        Self {
            black: default_000(),
            red: "#ff0000".to_string(),
            green: "#00ff00".to_string(),
            yellow: "#ffff00".to_string(),
            blue: "#0000ff".to_string(),
            magenta: "#ff00ff".to_string(),
            cyan: "#00ffff".to_string(),
            white: default_fff(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct YAMLThemeBright {
    pub black: YAMLThemeColor,
    pub red: YAMLThemeColor,
    pub green: YAMLThemeColor,
    pub yellow: YAMLThemeColor,
    pub blue: YAMLThemeColor,
    pub magenta: YAMLThemeColor,
    pub cyan: YAMLThemeColor,
    pub white: YAMLThemeColor,
}

impl Default for YAMLThemeBright {
    fn default() -> Self {
        Self {
            black: "#777777".to_string(),
            red: default_000(),
            green: default_000(),
            yellow: default_000(),
            blue: default_000(),
            magenta: default_000(),
            cyan: default_000(),
            white: default_000(),
        }
    }
}

#[inline]
fn default_name() -> String {
    "empty name".to_string()
}

#[inline]
fn default_author() -> String {
    "empty author".to_string()
}

#[inline]
fn default_000() -> YAMLThemeColor {
    "#00000".to_string()
}

#[inline]
fn default_fff() -> YAMLThemeColor {
    "#FFFFFF".to_string()
}

#[cfg(test)]
mod test {
    use std::{ffi::OsStr, fs::File, io::BufReader, path::PathBuf};

    use crate::config::v2::tui::theme::ThemeColors;

    use super::*;

    /// First test one theme for better debugging
    #[test]
    fn should_parse_one_theme() {
        let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
        let reader = BufReader::new(
            File::open(format!("{cargo_manifest_dir}/themes/Afterglow.yml")).unwrap(),
        );
        let parsed: YAMLTheme = serde_yaml::from_reader(reader).unwrap();
        assert_eq!(
            parsed,
            YAMLTheme {
                colors: YAMLThemeColors {
                    name: default_name(),
                    author: default_author(),
                    primary: YAMLThemePrimary {
                        background: "#2c2c2c".to_string(),
                        foreground: "#d6d6d6".to_string()
                    },
                    cursor: YAMLThemeCursor {
                        text: "#2c2c2c".to_string(),
                        cursor: "#d9d9d9".to_string(),
                    },
                    normal: YAMLThemeNormal {
                        black: "#1c1c1c".to_string(),
                        red: "#bc5653".to_string(),
                        green: "#909d63".to_string(),
                        yellow: "#ebc17a".to_string(),
                        blue: "#7eaac7".to_string(),
                        magenta: "#aa6292".to_string(),
                        cyan: "#86d3ce".to_string(),
                        white: "#cacaca".to_string(),
                    },
                    bright: YAMLThemeBright {
                        black: "#636363".to_string(),
                        red: "#bc5653".to_string(),
                        green: "#909d63".to_string(),
                        yellow: "#ebc17a".to_string(),
                        blue: "#7eaac7".to_string(),
                        magenta: "#aa6292".to_string(),
                        cyan: "#86d3ce".to_string(),
                        white: "#f7f7f7".to_string(),
                    },
                },
            }
        );
    }

    /// Test that all themes in /lib/themes/ can be loaded
    #[test]
    fn should_parse_all_themes() {
        let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(format!("{cargo_manifest_dir}/themes/"));
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();

            if entry.path().extension() != Some(OsStr::new("yml")) {
                continue;
            }

            println!(
                "Theme: {}",
                entry.path().file_name().unwrap().to_string_lossy()
            );

            let reader = BufReader::new(File::open(entry.path()).unwrap());
            let parsed: std::result::Result<YAMLTheme, _> = serde_yaml::from_reader(reader);

            let parsed = parsed.unwrap();

            let actual_theme = ThemeColors::try_from(parsed);

            let _actual_theme = actual_theme.unwrap();
        }
    }
}
