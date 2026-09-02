use anyhow::Result;
use termusiclib::player::protobuf::common::Empty;
use termusiclib::player::protobuf::player::GetProgressResponse;
use termusiclib::player::protobuf::player::player_control_client::PlayerControlClient;
use termusiclib::player::{PlayerProgress, RunningStatus};
use tonic::transport::Channel;

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
        let request = tonic::Request::new(Empty {});
        let response = self.client.toggle_pause(request).await?;
        let response = response.into_inner();
        let status = RunningStatus::from_u32(response.status);
        info!("Got response from server: {response:?}");
        Ok(status)
    }

    pub async fn skip_next(&mut self) -> Result<()> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.skip_next(request).await?;
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn get_progress(&mut self) -> Result<GetProgressResponse> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.get_progress(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response)
    }

    pub async fn volume_up(&mut self) -> Result<u16> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.volume_up(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        // clamped to u16::MAX, also send is a u16, but protobuf does not support u16 directly
        #[allow(clippy::cast_possible_truncation)]
        Ok(response.volume.min(u32::from(u16::MAX)) as u16)
    }

    pub async fn volume_down(&mut self) -> Result<u16> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.volume_down(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        // clamped to u16::MAX, also send is a u16, but protobuf does not support u16 directly
        #[allow(clippy::cast_possible_truncation)]
        Ok(response.volume.min(u32::from(u16::MAX)) as u16)
    }

    pub async fn speed_up(&mut self) -> Result<i32> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.speed_up(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response.speed)
    }

    pub async fn speed_down(&mut self) -> Result<i32> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.speed_down(request).await?;
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

    pub async fn restart_track(&mut self) -> Result<PlayerProgress> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.restart_track(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response.into())
    }

    pub async fn seek_forward(&mut self) -> Result<PlayerProgress> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.seek_forward(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response.into())
    }

    pub async fn seek_backward(&mut self) -> Result<PlayerProgress> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.seek_backward(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(response.into())
    }

    pub async fn skip_previous(&mut self) -> Result<()> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.skip_previous(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }
}
