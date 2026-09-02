use termusiclib::player::{
    ChangeRunningState, ChangeSpeed, ChangeVolume, RunningStatus, SeekReq,
    protobuf::{
        common::Empty,
        player::{GetProgressResponse, player_control_client::PlayerControlClient},
    },
};
use tonic::{Result, transport::Channel};

/// Handle TUI Requests to the server.
#[derive(Debug)]
pub struct PlayerControlConsumer {
    client: PlayerControlClient<Channel>,
}

impl PlayerControlConsumer {
    pub fn new(raw_client: Channel) -> Self {
        let client = PlayerControlClient::new(raw_client);
        Self { client }
    }

    pub async fn toggle_pause(&mut self) -> Result<RunningStatus> {
        let request = tonic::Request::new(ChangeRunningState::Toggle.into());
        let response = self.client.change_running_state(request).await?;
        let response = response.into_inner();
        let status = RunningStatus::from_u32(response.status);
        info!("Got response from server: {response:?}");
        Ok(status)
    }

    pub async fn get_progress(&mut self) -> Result<GetProgressResponse> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.get_progress(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response)
    }

    pub async fn volume_up(&mut self) -> Result<u16> {
        let request = tonic::Request::new(ChangeVolume::Steps(1).into());
        let response = self.client.change_volume(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        // clamped to u16::MAX, also send is a u16, but protobuf does not support u16 directly
        #[allow(clippy::cast_possible_truncation)]
        Ok(response.volume.min(u32::from(u16::MAX)) as u16)
    }

    pub async fn volume_down(&mut self) -> Result<u16> {
        let request = tonic::Request::new(ChangeVolume::Steps(-1).into());
        let response = self.client.change_volume(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        // clamped to u16::MAX, also send is a u16, but protobuf does not support u16 directly
        #[allow(clippy::cast_possible_truncation)]
        Ok(response.volume.min(u32::from(u16::MAX)) as u16)
    }

    pub async fn speed_up(&mut self) -> Result<i32> {
        let request = tonic::Request::new(ChangeSpeed::Steps(1).into());
        let response = self.client.change_speed(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response.speed)
    }

    pub async fn speed_down(&mut self) -> Result<i32> {
        let request = tonic::Request::new(ChangeSpeed::Steps(-1).into());
        let response = self.client.change_speed(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response.speed)
    }

    pub async fn toggle_gapless(&mut self) -> Result<bool> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.toggle_gapless(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response.gapless)
    }

    pub async fn restart_track(&mut self) -> Result<()> {
        let request = tonic::Request::new(SeekReq::RestartTrack.into());
        let response = self.client.seek(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn seek_forward(&mut self) -> Result<()> {
        let request = tonic::Request::new(SeekReq::Steps(1).into());
        let response = self.client.seek(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn seek_backward(&mut self) -> Result<()> {
        let request = tonic::Request::new(SeekReq::Steps(-1).into());
        let response = self.client.seek(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn skip_next(&mut self) -> Result<()> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.skip_next(request).await?;
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn skip_previous(&mut self) -> Result<()> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.skip_previous(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }
}
