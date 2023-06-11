use anyhow::Result;
use termusicplayback::player::music_player_client::MusicPlayerClient;
use termusicplayback::player::{
    CycleLoopRequest, GetProgressRequest, GetProgressResponse, PlaySelectedRequest,
    ReloadConfigRequest, ReloadPlaylistRequest, SeekBackwardRequest, SeekForwardRequest,
    SkipNextRequest, SkipPreviousRequest, SpeedDownRequest, SpeedUpRequest, ToggleGaplessRequest,
    TogglePauseRequest, VolumeDownRequest, VolumeUpRequest,
};
use termusicplayback::Status;
use tonic::transport::Channel;

pub struct Playback {
    client: MusicPlayerClient<Channel>,
}

impl Playback {
    pub async fn new() -> Result<Self> {
        let client = MusicPlayerClient::connect("http://[::1]:50051").await?;
        Ok(Self { client })
    }
    pub async fn toggle_pause(&mut self) -> Result<Status> {
        let request = tonic::Request::new(TogglePauseRequest {});
        let response = self.client.toggle_pause(request).await?;
        let response = response.into_inner();
        let status = Status::from_u32(response.status);
        info!("Got response from server: {:?}", response);
        Ok(status)
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

    pub async fn seek_forward(&mut self) -> Result<(u32, u32)> {
        let request = tonic::Request::new(SeekForwardRequest {});
        let response = self.client.seek_forward(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok((response.position, response.duration))
    }

    pub async fn seek_backward(&mut self) -> Result<(u32, u32)> {
        let request = tonic::Request::new(SeekBackwardRequest {});
        let response = self.client.seek_backward(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok((response.position, response.duration))
    }

    pub async fn reload_config(&mut self) -> Result<()> {
        let request = tonic::Request::new(ReloadConfigRequest {});
        let response = self.client.reload_config(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(())
    }

    pub async fn reload_playlist(&mut self) -> Result<()> {
        let request = tonic::Request::new(ReloadPlaylistRequest {});
        let response = self.client.reload_playlist(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(())
    }
    pub async fn play_selected(&mut self) -> Result<()> {
        let request = tonic::Request::new(PlaySelectedRequest {});
        let response = self.client.play_selected(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(())
    }
    pub async fn skip_previous(&mut self) -> Result<()> {
        let request = tonic::Request::new(SkipPreviousRequest {});
        let response = self.client.skip_previous(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {:?}", response);
        Ok(())
    }
}
