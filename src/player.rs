use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct Player {
    _stream: OutputStream,
    sink: Sink,
    stream_handle: OutputStreamHandle,
    volume: f32,
}

impl Player {
    pub fn new(volume: f32) -> Result<Player, rodio::StreamError> {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        Ok(Player {
            _stream,
            sink,
            stream_handle,
            volume,
        })
    }
    pub fn play(&mut self, p: &Path) {
        self.reset_sink();
        // let fullpath = p.to_path_buf();
        // let mut player = Player::new(0.9).expect("error creating player");
        // player.play_song(fullpath).expect("error playing song");
        // Get a output stream handle to the default physical sound device
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = BufReader::new(File::open(p).unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();
        // Play the sound directly on the device
        // stream_handle
        //     .play_raw(source.convert_samples())
        //     .expect("error happened duign playing");

        // Add a dummy source of the sake of the example.
        self.sink.append(source);

        // The sound plays in a separate thread. This call will block the current thread until the sink
        // has finished playing all its queued sounds.
        self.sink.sleep_until_end();
        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        // std::thread::sleep(std::time::Duration::from_secs(50));
    }

    fn reset_sink(&mut self) {
        // FIXME: actually handle the error instead of just expecting
        self.sink = rodio::Sink::try_new(&self.stream_handle).expect("error opening sink");
        self.sink.set_volume(self.volume);
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = if v < 0f32 {
            0f32
        } else if v > 1f32 {
            1f32
        } else {
            v
        };
        self.sink.set_volume(self.volume);
    }
}
