use anyhow::Result;
use termusiclib::player::protobuf::{
    common::Empty, server::server_control_client::ServerControlClient,
};
use tonic::{Request, transport::Channel};

/// Handle TUI Requests to the server.
#[derive(Debug)]
pub struct ServerControlConsumer {
    client: ServerControlClient<Channel>,
}

impl ServerControlConsumer {
    pub fn new(raw_client: Channel) -> Self {
        let client = ServerControlClient::new(raw_client);
        Self { client }
    }

    pub async fn reload_config(&mut self) -> Result<()> {
        let request = Request::new(Empty {});
        let response = self.client.reload_config(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn quit_server(&mut self) -> Result<()> {
        let request = Request::new(Empty {});
        let response = self.client.quit_server(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }
}
