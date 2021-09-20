#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(unused)]
pub enum Playback {
    Playing,
    Paused,
    Stopped,
}

impl Playback {
    pub fn value(self) -> String {
        match self {
            Playback::Playing => "Playing".to_string(),
            Playback::Paused => "Paused".to_string(),
            Playback::Stopped => "Stopped".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Loop {
    None,
    Track,
    Playlist,
}

impl Loop {
    pub fn value(self) -> String {
        match self {
            Loop::None => "None".to_string(),
            Loop::Track => "Track".to_string(),
            Loop::Playlist => "Playlist".to_string(),
        }
    }
}
