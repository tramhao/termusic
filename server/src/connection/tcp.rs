use std::net::SocketAddr;

use anyhow::{Context as _, Result};
use termusiclib::config::SharedServerSettings;
use tonic::transport::server::TcpIncoming;

/// Create the TCP Stream for HTTP requests.
pub async fn tcp_stream(config: &SharedServerSettings) -> Result<(TcpIncoming, SocketAddr)> {
    let addr = SocketAddr::from(&config.read().settings.com);

    // workaround to print address once sever "actually" is started and address is known
    // see https://github.com/hyperium/tonic/issues/351
    let tcp_listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Error binding address: {addr}"))?;

    // workaround as "TcpIncoming" does not provide a function to get the address
    let socket_addr = tcp_listener.local_addr()?;

    let stream = TcpIncoming::from(tcp_listener).with_nodelay(Some(true));

    Ok((stream, socket_addr))
}
