mod player;
pub use player::Mpris;

mod metadata;
pub use metadata::Metadata;

mod status;
pub use status::Loop;
pub use status::Playback;

mod generated;
pub use generated::mediaplayer2::OrgMprisMediaPlayer2;
pub use generated::mediaplayer2_player::OrgMprisMediaPlayer2Player;
