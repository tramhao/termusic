use anyhow::{Context, Result};
use termusiclib::config::v2::server::LoopMode;
use termusiclib::player::music_player_client::MusicPlayerClient;
use termusiclib::player::playlist_helpers::{
    PlaylistAddTrack, PlaylistPlaySpecific, PlaylistRemoveTrackType, PlaylistSwapTrack,
};
use termusiclib::player::{
    Empty, GetProgressResponse, PlayerProgress, PlaylistSwapTracks, PlaylistTracks,
    PlaylistTracksToAdd, PlaylistTracksToRemove,
};
use termusicplayback::Status;
use tokio_stream::{Stream, StreamExt as _};
use tonic::transport::Channel;

/// Handle TUI Requests to the server.
#[derive(Debug)]
pub struct Playback {
    client: MusicPlayerClient<Channel>,
}

impl Playback {
    pub fn new(client: MusicPlayerClient<Channel>) -> Self {
        Self { client }
    }

    pub async fn toggle_pause(&mut self) -> Result<Status> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.toggle_pause(request).await?;
        let response = response.into_inner();
        let status = Status::from_u32(response.status);
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
        // info!("Got response from server: {:?}", response);
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

    pub async fn cycle_loop(&mut self) -> Result<LoopMode> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.cycle_loop(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        let as_u8 = u8::try_from(response.mode).context("Failed to convert u32 to u8")?;
        let loop_mode =
            LoopMode::tryfrom_discriminant(as_u8).context("Failed to get LoopMode from u8")?;
        Ok(loop_mode)
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

    pub async fn reload_config(&mut self) -> Result<()> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.reload_config(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn reload_playlist(&mut self) -> Result<()> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.reload_playlist(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn play_specific(&mut self, info: PlaylistPlaySpecific) -> Result<()> {
        let request = tonic::Request::new(info.into());
        let response = self.client.play_specific(request).await?;
        let response = response.into_inner();
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

    pub async fn subscribe_to_stream_updates(
        &mut self,
    ) -> Result<impl Stream<Item = Result<termusiclib::player::StreamUpdates>>> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.subscribe_server_updates(request).await?;
        let response = response.into_inner().map(|res| res.map_err(Into::into));
        info!("Got response from server: {response:?}");
        Ok(response)
    }

    pub async fn add_to_playlist(&mut self, info: PlaylistAddTrack) -> Result<()> {
        let request = tonic::Request::new(PlaylistTracksToAdd::from(info));
        let response = self.client.add_to_playlist(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }

    pub async fn remove_from_playlist(&mut self, info: PlaylistRemoveTrackType) -> Result<()> {
        let request = tonic::Request::new(PlaylistTracksToRemove::from(info));
        let response = self.client.remove_from_playlist(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }

    pub async fn swap_tracks(&mut self, info: PlaylistSwapTrack) -> Result<()> {
        let request = tonic::Request::new(PlaylistSwapTracks::from(info));
        let response = self.client.swap_tracks(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }

    pub async fn get_playlist(&mut self) -> Result<PlaylistTracks> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.get_playlist(request).await?;
        // This might be massively spamming the log
        info!("Got response from server: {response:?}");

        Ok(response.into_inner())
    }

    pub async fn shuffle_playlist(&mut self) -> Result<PlaylistTracks> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.shuffle_playlist(request).await?;
        // This might be massively spamming the log
        info!("Got response from server: {response:?}");

        Ok(response.into_inner())
    }

    pub async fn remove_deleted_tracks(&mut self) -> Result<()> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.remove_deleted_tracks(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }
}
