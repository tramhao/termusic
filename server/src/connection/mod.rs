mod tcp;
#[cfg(unix)]
mod uds;

use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

pub use tcp::tcp_stream;
use tokio::io::{AsyncRead, AsyncWrite};
use tonic::transport::server::Connected;
#[cfg(unix)]
pub use uds::uds_stream;

pub type ActiveConnections = Arc<ActiveConnectionData>;

#[derive(Debug, Default)]
pub struct ActiveConnectionData {
    count: AtomicUsize,
    had_first_connection: AtomicBool,
}

impl ActiveConnectionData {
    /// Get whether there are still active connections or not.
    pub fn has_active_connections(&self) -> bool {
        self.count.load(Ordering::SeqCst) != 0
    }

    /// Get wheter we had at least one connection.
    pub fn had_first_connection(&self) -> bool {
        self.had_first_connection.load(Ordering::SeqCst)
    }
}

pub struct ConnectionWrapper<C> {
    inner: C,
    active_connection_data: ActiveConnections,
}

impl<C> ConnectionWrapper<C> {
    pub fn new(socket: C, active_connection_data: ActiveConnections) -> Self {
        info!("New client connection");
        active_connection_data.count.fetch_add(1, Ordering::AcqRel);
        active_connection_data
            .had_first_connection
            .store(true, Ordering::SeqCst);
        Self {
            inner: socket,
            active_connection_data,
        }
    }
}

impl<C> Connected for ConnectionWrapper<C> {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {}
}

impl<C> AsyncRead for ConnectionWrapper<C>
where
    C: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<C> AsyncWrite for ConnectionWrapper<C>
where
    C: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl<C> Drop for ConnectionWrapper<C> {
    fn drop(&mut self) {
        info!("Connection dropped");
        self.active_connection_data
            .count
            .fetch_sub(1, Ordering::AcqRel);
    }
}
