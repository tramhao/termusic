// OGG
#[cfg(feature = "format-vorbis")]
pub const VORBIS_IDENT_HEAD: [u8; 7] = [1, 118, 111, 114, 98, 105, 115];
#[cfg(feature = "format-vorbis")]
pub const VORBIS_COMMENT_HEAD: [u8; 7] = [3, 118, 111, 114, 98, 105, 115];
#[cfg(feature = "format-vorbis")]
pub const VORBIS_SETUP_HEAD: [u8; 7] = [5, 118, 111, 114, 98, 105, 115];

#[cfg(feature = "format-opus")]
pub const OPUSTAGS: [u8; 8] = [79, 112, 117, 115, 84, 97, 103, 115];
#[cfg(feature = "format-opus")]
pub const OPUSHEAD: [u8; 8] = [79, 112, 117, 115, 72, 101, 97, 100];
