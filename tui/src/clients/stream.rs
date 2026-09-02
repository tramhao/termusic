use anyhow::Result;
use termusiclib::player::protobuf::{
    common::Empty,
    stream::{StreamUpdates, stream_events_client::StreamEventsClient},
};
use tokio_stream::{Stream, StreamExt as _};
use tonic::{Request, transport::Channel};

/// Handle TUI Requests to the server.
#[derive(Debug)]
pub struct StreamEventsConsumer {
    client: StreamEventsClient<Channel>,
}

impl StreamEventsConsumer {
    pub fn new(raw_client: Channel) -> Self {
        let client = StreamEventsClient::new(raw_client);
        Self { client }
    }

    pub async fn subscribe_to_stream_updates(
        &mut self,
    ) -> Result<impl Stream<Item = Result<StreamUpdates>> + use<>> {
        let request = Request::new(Empty {});
        let response = self.client.subscribe_server_updates(request).await?;
        let response = response.into_inner().map(|res| res.map_err(Into::into));
        info!("Got response from server: {response:?}");
        Ok(response)
    }
}
