use crate::config::Settings;
use anyhow::{anyhow, bail, Result};
use lazy_static::lazy_static;
use pinyin::ToPinyin;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::{
    ffi::OsStr,
    process::{Child, Command},
};
use tuirealm::props::Color;
use unicode_segmentation::UnicodeSegmentation;

lazy_static! {
    /**
     * Regex matches:
     * - group 1: Red
     * - group 2: Green
     * - group 3: Blue
     */
    static ref COLOR_HEX_REGEX: Regex = Regex::new(r"#(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})").unwrap();
}

pub fn get_pin_yin(input: &str) -> String {
    let mut b = String::new();
    for (index, f) in input.to_pinyin().enumerate() {
        match f {
            Some(p) => {
                b.push_str(p.plain());
            }
            None => {
                if let Some(c) = input.to_uppercase().chars().nth(index) {
                    b.push(c);
                }
            }
        }
    }
    b
}

/// # Panics
/// panics could happen when color parse failed
pub fn parse_hex_color(color: &str) -> Option<Color> {
    COLOR_HEX_REGEX.captures(color).map(|groups| {
        Color::Rgb(
            u8::from_str_radix(groups.get(1).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(2).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(3).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
        )
    })
}

pub fn filetype_supported(current_node: &str) -> bool {
    let p = Path::new(current_node);

    if p.starts_with("http") {
        return true;
    }

    #[cfg(any(feature = "mpv", feature = "gst"))]
    if let Some(ext) = p.extension() {
        if ext == "opus" {
            return true;
        }
        if ext == "aiff" {
            return true;
        }
    }

    match p.extension() {
        Some(ext) if ext == "mkv" || ext == "mka" => true,
        Some(ext) if ext == "mp3" => true,
        // Some(ext) if ext == "aiff" => true,
        Some(ext) if ext == "flac" => true,
        Some(ext) if ext == "m4a" => true,
        Some(ext) if ext == "aac" => true,
        // Some(ext) if ext == "opus" => true,
        Some(ext) if ext == "ogg" => true,
        Some(ext) if ext == "wav" => true,
        Some(ext) if ext == "webm" => true,
        Some(_) | None => false,
    }
}

pub fn is_playlist(current_node: &str) -> bool {
    let p = Path::new(current_node);

    match p.extension() {
        Some(ext) if ext == "m3u" => true,
        Some(ext) if ext == "m3u8" => true,
        Some(ext) if ext == "pls" => true,
        Some(ext) if ext == "asx" => true,
        Some(ext) if ext == "xspf" => true,
        Some(_) | None => false,
    }
}

pub fn get_parent_folder(filename: &str) -> String {
    let parent_folder: PathBuf;
    let path_old = Path::new(filename);

    if path_old.is_dir() {
        parent_folder = path_old.to_path_buf();
        return parent_folder.to_string_lossy().to_string();
    }
    match path_old.parent() {
        Some(p) => parent_folder = p.to_path_buf(),
        None => parent_folder = std::env::temp_dir(),
    }
    parent_folder.to_string_lossy().to_string()
}

pub fn get_app_config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().ok_or_else(|| anyhow!("failed to find os config dir."))?;
    path.push("termusic");

    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }
    Ok(path)
}

fn get_podcast_save_path(config: &Settings) -> Result<PathBuf> {
    let full_path = shellexpand::tilde(&config.podcast_dir).to_string();
    let full_path_pathbuf = PathBuf::from(full_path);
    if !full_path_pathbuf.exists() {
        std::fs::create_dir_all(&full_path_pathbuf)?;
    }
    Ok(full_path_pathbuf)
}

pub fn create_podcast_dir(config: &Settings, pod_title: String) -> Result<PathBuf> {
    match get_podcast_save_path(config) {
        Ok(mut download_path) => {
            download_path.push(pod_title);
            match std::fs::create_dir_all(&download_path) {
                Ok(()) => Ok(download_path),
                Err(e) => bail!("Error in creating podcast feeds download dir: {e}"),
            }
        }
        Err(e) => bail!("Error in creating podcast download dir: {e}"),
    }
}

pub fn playlist_get_vec(current_node: &str) -> Result<Vec<String>> {
    let p = Path::new(current_node);
    let p_base = p.parent().ok_or_else(|| anyhow!("cannot find path root"))?;
    let str = std::fs::read_to_string(p)?;
    let items =
        crate::playlist::decode(&str).map_err(|e| anyhow!("playlist decode error: {}", e))?;
    let mut vec = vec![];
    for item in items {
        if let Ok(pathbuf) = playlist_get_absolute_pathbuf(&item, p_base) {
            vec.push(pathbuf.to_string_lossy().to_string());
        }
    }
    Ok(vec)
}

fn playlist_get_absolute_pathbuf(item: &str, p_base: &Path) -> Result<PathBuf> {
    let mut url = urlencoding::decode(item)?.into_owned();
    if url.starts_with("http") {
        return Ok(PathBuf::from(url));
        // bail!("http not supported");
    }
    if url.starts_with("file") {
        url = url.replace("file://", "");
    }
    let pathbuf = if Path::new(&url).is_relative() {
        let mut pathbuf = PathBuf::from(p_base);
        pathbuf.push(url);

        pathbuf
    } else {
        PathBuf::from(url)
    };
    Ok(pathbuf)
}

/// Some helper functions for dealing with Unicode strings.
#[allow(clippy::module_name_repetitions)]
pub trait StringUtils {
    fn substr(&self, start: usize, length: usize) -> String;
    fn grapheme_len(&self) -> usize;
}

impl StringUtils for String {
    /// Takes a slice of the String, properly separated at Unicode
    /// grapheme boundaries. Returns a new String.
    fn substr(&self, start: usize, length: usize) -> String {
        return self
            .graphemes(true)
            .skip(start)
            .take(length)
            .collect::<String>();
    }

    /// Counts the total number of Unicode graphemes in the String.
    fn grapheme_len(&self) -> usize {
        return self.graphemes(true).count();
    }
}

/// Spawn a detached process
/// # Panics
/// panics when spawn server failed
pub fn spawn_process<A: IntoIterator<Item = S> + Clone, S: AsRef<OsStr>>(
    prog: &Path,
    superuser: bool,
    shout_output: bool,
    args: A,
) -> std::io::Result<Child> {
    let mut cmd = if superuser {
        let mut cmd_t = Command::new("sudo");
        cmd_t.arg(prog);
        cmd_t
    } else {
        Command::new(prog)
    };
    if !shout_output {
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
    }

    cmd.args(args);
    cmd.spawn()
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pin_yin() {
        assert_eq!(get_pin_yin("陈一发儿"), "chenyifaer".to_string());
        assert_eq!(get_pin_yin("Gala乐队"), "GALAledui".to_string());
        assert_eq!(get_pin_yin("乐队Gala乐队"), "leduiGALAledui".to_string());
        assert_eq!(get_pin_yin("Annett Louisan"), "ANNETT LOUISAN".to_string());
    }
}
