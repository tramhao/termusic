use std::io;

use symphonia::core::{
    audio::SampleBuffer,
    codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL},
    errors::Error,
    formats::{FormatOptions, FormatReader, SeekMode, SeekTo, Track},
    io::{MediaSource, MediaSourceStream},
    meta::MetadataOptions,
    probe::Hint,
    units::Time,
};

use super::{AudioDecoder, AudioPacket, AudioPacketPosition, DecoderError, DecoderResult};

pub const SAMPLE_RATE: u32 = 44100;
pub const NUM_CHANNELS: u8 = 2;
pub const SAMPLES_PER_SECOND: u32 = SAMPLE_RATE * NUM_CHANNELS as u32;
pub const PAGES_PER_MS: f64 = SAMPLE_RATE as f64 / 1000.0;
pub const MS_PER_PAGE: f64 = 1000.0 / SAMPLE_RATE as f64;

#[derive(Copy, Clone)]
#[allow(unused)]
struct PlayTrackOptions {
    track_id: u32,
    seek_ts: u64,
}

fn first_supported_track(tracks: &[Track]) -> Option<&Track> {
    tracks
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
}

pub struct SymphoniaDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    sample_buffer: Option<SampleBuffer<f64>>,
}

impl SymphoniaDecoder {
    pub fn new<R>(input: R, hint: Hint) -> DecoderResult<Self>
    where
        R: MediaSource + 'static,
    {
        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(Box::new(input), Default::default());

        // Use the default options for format readers other than for gapless playback.
        let format_opts = FormatOptions {
            enable_gapless: false,
            ..Default::default()
        };

        // Use the default options for metadata readers.
        let metadata_opts: MetadataOptions = Default::default();

        let track: Option<usize> = None;

        // Probe the media source stream for metadata and get the format reader.
        match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
            Ok(probed) => {
                // Playback mode.
                // print_format(song, &mut probed);

                // Set the decoder options.
                let decode_opts = DecoderOptions {
                    verify: false,
                    // ..Default::default()
                };

                // Play it!
                // play(probed.format, track, seek_time, &decode_opts, no_progress);

                // If the user provided a track number, select that track if it exists, otherwise, select the
                // first track with a known codec.
                let track = track
                    .and_then(|t| probed.format.tracks().get(t))
                    .or_else(|| first_supported_track(probed.format.tracks()));

                let track_id = match track {
                    Some(track) => track.id,
                    _ => {
                        return Err(DecoderError::SymphoniaDecoder(
                            "No supported tracks found".to_string(),
                        ))
                    }
                };

                let seek_ts = 0;

                let track_info = PlayTrackOptions { track_id, seek_ts };

                // Get the selected track using the track ID.
                let track = match probed
                    .format
                    .tracks()
                    .iter()
                    .find(|track| track.id == track_info.track_id)
                {
                    Some(track) => track,
                    _ => {
                        return Err(DecoderError::SymphoniaDecoder(
                            "No supported tracks found".to_string(),
                        ))
                    }
                };

                // Create a decoder for the track.
                let decoder =
                    symphonia::default::get_codecs().make(&track.codec_params, &decode_opts)?;
                Ok(SymphoniaDecoder {
                    format: probed.format,
                    decoder,
                    sample_buffer: None,
                })
            }
            Err(err) => {
                // The input was not supported by any format reader.
                panic!("file not supported. reason? {}", err);
            }
        }
    }

    fn ts_to_ms(&self, ts: u64) -> u32 {
        let time_base = self.decoder.codec_params().time_base;
        let seeked_to_ms = match time_base {
            Some(time_base) => {
                let time = time_base.calc_time(ts);
                (time.seconds as f64 + time.frac) * 1000.
            }
            // Fallback in the unexpected case that the format has no base time set.
            None => ts as f64 * PAGES_PER_MS,
        };
        seeked_to_ms as u32
    }
}

impl AudioDecoder for SymphoniaDecoder {
    fn seek(&mut self, position_ms: u32) -> Result<u32, DecoderError> {
        let seconds = position_ms as u64 / 1000;
        let frac = (position_ms as f64 % 1000.) / 1000.;
        let time = Time::new(seconds, frac);

        // `track_id: None` implies the default track ID (of the container, not of Spotify).
        let seeked_to_ts = self.format.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time,
                track_id: None,
            },
        )?;

        // Seeking is a `FormatReader` operation, so the decoder cannot reliably
        // know when a seek took place. Reset it to avoid audio glitches.
        self.decoder.reset();

        Ok(self.ts_to_ms(seeked_to_ts.actual_ts))
    }

    fn next_packet(
        &mut self,
    ) -> DecoderResult<Option<(AudioPacketPosition, AudioPacket, u16, u32)>> {
        let mut skipped = false;

        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(Error::IoError(err)) => {
                    if err.kind() == io::ErrorKind::UnexpectedEof {
                        return Ok(None);
                    } else {
                        return Err(DecoderError::SymphoniaDecoder(err.to_string()));
                    }
                }
                Err(err) => {
                    return Err(err.into());
                }
            };

            let position_ms = self.ts_to_ms(packet.ts());
            let packet_position = AudioPacketPosition {
                position_ms,
                skipped,
            };

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let sample_buffer = match self.sample_buffer.as_mut() {
                        Some(buffer) => buffer,
                        None => {
                            let duration = decoded.capacity() as u64;
                            self.sample_buffer.insert(SampleBuffer::new(duration, spec))
                        }
                    };

                    sample_buffer.copy_interleaved_ref(decoded);
                    let samples = AudioPacket::Samples(sample_buffer.samples().to_vec());

                    return Ok(Some((
                        packet_position,
                        samples,
                        spec.channels.count() as u16,
                        spec.rate,
                    )));
                }
                Err(Error::DecodeError(_)) => {
                    // The packet failed to decode due to corrupted or invalid data, get a new
                    // packet and try again.
                    skipped = true;
                    continue;
                }
                Err(err) => return Err(err.into()),
            }
        }
    }
}
