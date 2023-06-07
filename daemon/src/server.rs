use termusicplayback::player::music_player_server::{MusicPlayer, MusicPlayerServer};
use termusicplayback::player::{TogglePauseRequest, TogglePauseResponse};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct MusicPlayerService {}

#[tonic::async_trait]
impl MusicPlayer for MusicPlayerService {
    async fn toggle_pause(
        &self,
        request: Request<TogglePauseRequest>,
    ) -> Result<Response<TogglePauseResponse>, Status> {
        println!("got a request: {:?}", request);
        // let req = request.into_inner();
        let reply = TogglePauseResponse {};
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let music_player_service: MusicPlayerService = MusicPlayerService::default();

    Server::builder()
        .add_service(MusicPlayerServer::new(music_player_service))
        .serve(addr)
        .await?;

    Ok(())
}
