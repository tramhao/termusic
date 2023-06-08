use anyhow::Result;
use termusicplayback::player::music_player_client::MusicPlayerClient;
use termusicplayback::player::{GetProgressRequest, SkipNextRequest, TogglePauseRequest};
use tonic::transport::Channel;

pub struct Playback {
    client: MusicPlayerClient<Channel>,
}

impl Playback {
    pub async fn new() -> Result<Self> {
        let client = MusicPlayerClient::connect("http://[::1]:50051").await?;
        Ok(Self { client })
    }
    pub async fn toggle_pause(&mut self) -> Result<()> {
        let request = tonic::Request::new(TogglePauseRequest {});
        let response = self.client.toggle_pause(request).await?;
        info!("Got response from server: {:?}", response);
        Ok(())
    }

    pub async fn skip_next(&mut self) -> Result<()> {
        let request = tonic::Request::new(SkipNextRequest {});
        let response = self.client.skip_next(request).await?;
        info!("Got response from server: {:?}", response);
        Ok(())
    }

    pub async fn get_progress(&mut self) -> Result<()> {
        let request = tonic::Request::new(GetProgressRequest {});
        let response = self.client.get_progress(request).await?;
        info!("Got response from server: {:?}", response);
        Ok(())
    }
}
