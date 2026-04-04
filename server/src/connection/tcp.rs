use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::{Context as _, Result};
use termusiclib::config::SharedServerSettings;
use tokio::net::TcpStream;
use tokio_stream::Stream;
use tonic::transport::server::TcpIncoming;

use crate::connection::{ActiveConnections, ConnectionWrapper};

#[derive(Debug)]
pub struct TcpStreamWrapper {
    inner: TcpIncoming,
    active_connection_count: ActiveConnections,
}

impl TcpStreamWrapper {
    /// Create a new `TcpStreamWrapper`.
    pub fn new(listener: TcpIncoming, active_connection_count: ActiveConnections) -> Self {
        Self {
            inner: listener,
            active_connection_count,
        }
    }
}

impl Stream for TcpStreamWrapper {
    type Item = std::io::Result<ConnectionWrapper<TcpStream>>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<std::io::Result<ConnectionWrapper<TcpStream>>>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(stream))) => Poll::Ready(Some(Ok(ConnectionWrapper::new(
                stream,
                self.active_connection_count.clone(),
            )))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Create the TCP Stream for HTTP requests.
pub async fn tcp_stream(
    config: &SharedServerSettings,
    active_connection_count: ActiveConnections,
) -> Result<(TcpStreamWrapper, SocketAddr)> {
    let addr = SocketAddr::from(&config.read().settings.com);

    // workaround to print address once sever "actually" is started and address is known
    // see https://github.com/hyperium/tonic/issues/351
    let tcp_listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Error binding address: {addr}"))?;

    // workaround as "TcpIncoming" does not provide a function to get the address
    let socket_addr = tcp_listener.local_addr()?;

    let stream = TcpIncoming::from(tcp_listener).with_nodelay(Some(true));
    let stream = TcpStreamWrapper::new(stream, active_connection_count);

    Ok((stream, socket_addr))
}
