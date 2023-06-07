use termusicplayback::player::music_player_client::MusicPlayerClient;
use termusicplayback::player::TogglePauseRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MusicPlayerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(TogglePauseRequest {});
    let response = client.toggle_pause(request).await?;
    println!("Got response from server: {:?}", response);
    Ok(())
}
