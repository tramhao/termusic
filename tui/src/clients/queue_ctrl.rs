use anyhow::{Context, Result};
use termusiclib::{
    config::v2::server::LoopMode,
    player::{
        playlist_helpers::{
            PlaylistAddTrack, PlaylistPlaySpecific, PlaylistRemoveTrackType, PlaylistSwapTrack,
        },
        protobuf::{
            common::Empty,
            queue::{
                PlaylistState, PlaylistSwapTracksRequest, PlaylistTracksToAddRequest,
                PlaylistTracksToRemoveRequest, SortCriterion, SortDirection, SortPlaylistRequest,
                queue_control_client::QueueControlClient,
            },
        },
    },
};
use tonic::{Request, transport::Channel};

/// Handle TUI Requests to the server.
#[derive(Debug)]
pub struct QueueControlConsumer {
    client: QueueControlClient<Channel>,
}

impl QueueControlConsumer {
    pub fn new(raw_client: Channel) -> Self {
        let client = QueueControlClient::new(raw_client);
        Self { client }
    }

    pub async fn cycle_loop(&mut self) -> Result<LoopMode> {
        let request = Request::new(Empty {});
        let response = self.client.cycle_loop(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        let as_u8 = u8::try_from(response.mode).context("Failed to convert u32 to u8")?;
        let loop_mode =
            LoopMode::tryfrom_discriminant(as_u8).context("Failed to get LoopMode from u8")?;
        Ok(loop_mode)
    }

    pub async fn play_specific(&mut self, info: PlaylistPlaySpecific) -> Result<()> {
        let request = Request::new(info.into());
        let response = self.client.play_specific(request).await?;
        let response = response.into_inner();
        info!("Got response from server: {response:?}");
        Ok(())
    }

    pub async fn add_to_playlist(&mut self, info: PlaylistAddTrack) -> Result<()> {
        let request = Request::new(PlaylistTracksToAddRequest::from(info));
        let response = self.client.add_to_playlist(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }

    pub async fn remove_from_playlist(&mut self, info: PlaylistRemoveTrackType) -> Result<()> {
        let request = Request::new(PlaylistTracksToRemoveRequest::from(info));
        let response = self.client.remove_from_playlist(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }

    pub async fn swap_tracks(&mut self, info: PlaylistSwapTrack) -> Result<()> {
        let request = Request::new(PlaylistSwapTracksRequest::from(info));
        let response = self.client.swap_tracks(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }

    pub async fn get_playlist(&mut self) -> Result<PlaylistState> {
        let request = Request::new(Empty {});
        let response = self.client.get_playlist(request).await?;
        // This might be massively spamming the log
        info!("Got response from server: {response:?}");

        Ok(response.into_inner())
    }

    pub async fn shuffle_playlist(&mut self) -> Result<()> {
        let request = Request::new(Empty {});
        let response = self.client.shuffle_playlist(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }

    pub async fn sort_playlist(
        &mut self,
        criterion: SortCriterion,
        direction: SortDirection,
    ) -> Result<()> {
        let request = Request::new(SortPlaylistRequest {
            criterion: criterion.into(),
            direction: direction.into(),
        });
        let response = self.client.sort_playlist(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }

    pub async fn remove_deleted_tracks(&mut self) -> Result<()> {
        let request = Request::new(Empty {});
        let response = self.client.remove_deleted_tracks(request).await?;
        info!("Got response from server: {response:?}");

        Ok(())
    }
}
