//! WAV/AIFF specific items
pub(crate) mod aiff;
pub(crate) mod chunk;
pub(crate) mod wav;

pub use crate::iff::aiff::AiffFile;
pub use crate::iff::wav::WavFile;

#[cfg(feature = "aiff_text_chunks")]
pub use crate::iff::aiff::tag::AiffTextChunks;
#[cfg(feature = "riff_info_list")]
pub use crate::iff::wav::tag::RiffInfoList;
pub use wav::{WavFormat, WavProperties};
