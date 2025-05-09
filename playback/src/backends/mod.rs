use std::{error::Error, fmt::Display};

use termusiclib::config::{v2::server::Backend as ConfigBackend, ServerOverlay};

use crate::{PlayerCmdSender, PlayerTrait};

#[cfg(feature = "gst")]
mod gstreamer;
#[cfg(feature = "mpv")]
mod mpv;
// public for benching lower modules
pub(crate) mod rusty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendSelect {
    #[cfg(feature = "mpv")]
    Mpv,
    #[cfg(feature = "gst")]
    GStreamer,
    #[default]
    Rusty,
}

/// Error for when [`ThemeColor`] parsing fails
#[derive(Debug, Clone, PartialEq)]
pub enum BackendSelectConvertError {
    UnavailableBackend(String),
}

impl Display for BackendSelectConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendSelectConvertError::UnavailableBackend(backend) => {
                write!(f, "Backend {backend} is unavailable")
            }
        }
    }
}

impl Error for BackendSelectConvertError {}

impl TryFrom<ConfigBackend> for BackendSelect {
    type Error = BackendSelectConvertError;

    fn try_from(value: ConfigBackend) -> Result<Self, Self::Error> {
        Ok(match value {
            #[cfg(feature = "gst")]
            ConfigBackend::Gstreamer => Self::GStreamer,
            #[cfg(feature = "mpv")]
            ConfigBackend::Mpv => Self::Mpv,
            ConfigBackend::Rusty => Self::Rusty,
            #[allow(unreachable_patterns)] // allow as a catch-all because of feature gates
            _ => {
                return Err(BackendSelectConvertError::UnavailableBackend(
                    value.to_string(),
                ))
            }
        })
    }
}

/// Enum to choose backend at runtime
#[non_exhaustive]
pub enum Backend {
    #[cfg(feature = "mpv")]
    Mpv(mpv::MpvBackend),
    Rusty(rusty::RustyBackend),
    #[cfg(feature = "gst")]
    GStreamer(gstreamer::GStreamerBackend),
}

impl Backend {
    /// Create a new Backend based on `backend`([`BackendSelect`])
    pub(crate) fn new_select(
        backend: BackendSelect,
        config: &ServerOverlay,
        cmd_tx: PlayerCmdSender,
    ) -> Self {
        match backend {
            #[cfg(feature = "mpv")]
            BackendSelect::Mpv => Self::new_mpv(config, cmd_tx),
            #[cfg(feature = "gst")]
            BackendSelect::GStreamer => Self::new_gstreamer(config, cmd_tx),
            BackendSelect::Rusty => Self::new_rusty(config, cmd_tx),
        }
    }

    // /// Create a new Backend with default backend ordering
    // ///
    // /// For the order see [`BackendSelect::Default`]
    // #[allow(unreachable_code)]
    // fn new_default(config: &ServerOverlay, cmd_tx: PlayerCmdSender) -> Self {
    //     #[cfg(feature = "gst")]
    //     return Self::new_gstreamer(config, cmd_tx);
    //     #[cfg(feature = "mpv")]
    //     return Self::new_mpv(config, cmd_tx);
    //     return Self::new_rusty(config, cmd_tx);
    // }

    /// Explicitly choose Backend [`RustyBackend`](rusty::RustyBackend)
    fn new_rusty(config: &ServerOverlay, cmd_tx: PlayerCmdSender) -> Self {
        info!("Using Backend \"rusty\"");
        Self::Rusty(rusty::RustyBackend::new(config, cmd_tx))
    }

    /// Explicitly choose Backend [`GstreamerBackend`](gstreamer::GStreamerBackend)
    #[cfg(feature = "gst")]
    fn new_gstreamer(config: &ServerOverlay, cmd_tx: PlayerCmdSender) -> Self {
        info!("Using Backend \"GStreamer\"");
        Self::GStreamer(gstreamer::GStreamerBackend::new(config, cmd_tx))
    }

    /// Explicitly choose Backend [`MpvBackend`](mpv::MpvBackend)
    #[cfg(feature = "mpv")]
    fn new_mpv(config: &ServerOverlay, cmd_tx: PlayerCmdSender) -> Self {
        info!("Using Backend \"mpv\"");
        Self::Mpv(mpv::MpvBackend::new(config, cmd_tx))
    }

    #[must_use]
    pub fn as_player(&self) -> &dyn PlayerTrait {
        match self {
            #[cfg(feature = "mpv")]
            Backend::Mpv(v) => v,
            #[cfg(feature = "gst")]
            Backend::GStreamer(v) => v,
            Backend::Rusty(v) => v,
        }
    }

    #[must_use]
    pub fn as_player_mut(&mut self) -> &mut (dyn PlayerTrait + Send) {
        match self {
            #[cfg(feature = "mpv")]
            Backend::Mpv(v) => v,
            #[cfg(feature = "gst")]
            Backend::GStreamer(v) => v,
            Backend::Rusty(v) => v,
        }
    }
}
