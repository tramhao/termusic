use anyhow::Result;
use termusicplayback::player::music_player_client::MusicPlayerClient;
use termusicplayback::player::{
    CycleLoopRequest, GetProgressRequest, GetProgressResponse, SkipNextRequest, SpeedDownRequest,
    SpeedUpRequest, ToggleGaplessRequest, TogglePauseRequest, VolumeDownRequest, VolumeUpRequest,
};
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

    pub async fn get_progress(&mut self) -> Result<GetProgressResponse> {
        let request = tonic::Request::new(GetProgressRequest {});
        let response = self.client.get_progress(request).await?;
        let response = response.into_inner();
        // info!("Got response from server: {:?}", response);
        Ok(response)
    }

    pub async fn volume_up(&mut self) -> Result<i32> {
        let request = tonic::Request::new(VolumeUpRequest {});
        let response = self.client.volume_up(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(response.volume)
    }

    pub async fn volume_down(&mut self) -> Result<i32> {
        let request = tonic::Request::new(VolumeDownRequest {});
        let response = self.client.volume_down(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(response.volume)
    }

    pub async fn cycle_loop(&mut self) -> Result<()> {
        let request = tonic::Request::new(CycleLoopRequest {});
        let response = self.client.cycle_loop(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(())
    }

    pub async fn speed_up(&mut self) -> Result<i32> {
        let request = tonic::Request::new(SpeedUpRequest {});
        let response = self.client.speed_up(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(response.speed)
    }

    pub async fn speed_down(&mut self) -> Result<i32> {
        let request = tonic::Request::new(SpeedDownRequest {});
        let response = self.client.speed_down(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(response.speed)
    }

    pub async fn toggle_gapless(&mut self) -> Result<bool> {
        let request = tonic::Request::new(ToggleGaplessRequest {});
        let response = self.client.toggle_gapless(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(response.gapless)
    }
}
