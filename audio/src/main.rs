pub mod decoder;
pub mod fetch;
pub mod range_set;

use decoder::{symphonia_decoder::SymphoniaDecoder, AudioDecoder};
use fetch::{AudioFile, Subfile};
use symphonia::core::probe::Hint;

type Decoder = Box<dyn AudioDecoder + Send>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://raw.githubusercontent.com/tsirysndr/music-player/master/fixtures/audio/06%20-%20J.%20Cole%20-%20Fire%20Squad(Explicit).m4a";
    // let url = "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a";
    let bytes_per_second = 40 * 1024; // 320kbps
    let audio_file = match AudioFile::open(url, bytes_per_second).await {
        Ok(audio_file) => audio_file,
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    };

    let stream_loader_controller = audio_file.get_stream_loader_controller()?;
    stream_loader_controller.set_stream_mode();
    let audio_file = match Subfile::new(audio_file, 0, stream_loader_controller.len() as u64) {
        Ok(audio_file) => audio_file,
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    };

    let symphonia_decoder = |audio_file, format| {
        SymphoniaDecoder::new(audio_file, format).map(|decoder| Box::new(decoder) as Decoder)
    };

    let mut format = Hint::new();
    format.mime_type(&AudioFile::get_mime_type(url).await?);

    let decoder_type = symphonia_decoder(audio_file, format);

    let mut decoder = match decoder_type {
        Ok(decoder) => decoder,
        Err(e) => {
            panic!("Failed to create decoder: {}", e);
        }
    };

    loop {
        match decoder.next_packet() {
            Ok(result) => {
                if let Some((ref packet_position, packet, _channels, _sample_rate)) = result {
                    match packet.samples() {
                        Ok(_samples) => {
                            println!("Packet: {:?}", packet_position);
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                } else {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
