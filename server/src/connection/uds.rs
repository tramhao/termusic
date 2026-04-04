use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::{Context as _, Result};
use termusiclib::config::SharedServerSettings;
use tokio::net::{UnixListener, UnixStream};
use tokio_stream::Stream;

/// Create the UDS Stream for UDS requests.
pub async fn uds_stream(config: &SharedServerSettings) -> Result<(UnixListenerStream, String)> {
    let path = &config.read().settings.com.socket_path;

    // if the file already exists, tokio will error with "Address already in use"
    // not using async here because of MutexGuard and being before anything important
    if path.exists() {
        warn!("Socket Path {} already exists, unlinking!", path.display());
        let _ = std::fs::remove_file(path);
    }

    let path_str = path.display().to_string();
    let uds = UnixListener::bind(path).with_context(|| path_str.clone())?;

    let stream = UnixListenerStream::new(uds);

    Ok((stream, path_str))
}

/// A wrapper around [`UnixListener`] that implements [`Stream`].
///
/// Copied from [`tokio_stream::wrappers::UnixListenerStream`], which is licensed MIT.
///
/// Modified because the normal implementation does not remove the socket on drop.
#[derive(Debug)]
#[cfg_attr(docsrs, doc(cfg(all(unix, feature = "net"))))]
pub struct UnixListenerStream {
    inner: UnixListener,
}

impl UnixListenerStream {
    /// Create a new `UnixListenerStream`.
    pub fn new(listener: UnixListener) -> Self {
        Self { inner: listener }
    }
}

impl Stream for UnixListenerStream {
    type Item = io::Result<UnixStream>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<io::Result<UnixStream>>> {
        match self.inner.poll_accept(cx) {
            Poll::Ready(Ok((stream, _))) => Poll::Ready(Some(Ok(stream))),
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsRef<UnixListener> for UnixListenerStream {
    fn as_ref(&self) -> &UnixListener {
        &self.inner
    }
}

impl AsMut<UnixListener> for UnixListenerStream {
    fn as_mut(&mut self) -> &mut UnixListener {
        &mut self.inner
    }
}

impl Drop for UnixListenerStream {
    fn drop(&mut self) {
        // unlink socket file as it is not done so by default
        let tmp = self.inner.local_addr().ok();
        if let Some(val) = tmp.as_ref().and_then(|v| v.as_pathname()) {
            let _ = std::fs::remove_file(val);
        }
    }
}
