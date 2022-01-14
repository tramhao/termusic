mod output;

use super::GeneralP;
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
// use mpv::{MpvHandler, MpvHandlerBuilder};
use anyhow::Result;
use std::cmp;
use std::fs::File;
use std::path::Path;
// use std::io::BufReader;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::{Error, Result as SymphoniaResult};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

// PlayerState is used to describe the status of player
#[derive(Clone)]
enum PlayerCommand {
    VolumeUp,
    VolumeDown,
    Stop,
    Play(String),
    Pause(bool),
    Progress,
    // Seek(i64),
}

pub struct Symphonia {
    volume: i32,
    sender: Sender<PlayerCommand>,
    progress_receiver: Receiver<i64>,
    // current_song: Option<Song>,
    paused: bool,
}

impl Default for Symphonia {
    fn default() -> Self {
        let (tx, rx): (Sender<PlayerCommand>, Receiver<PlayerCommand>) = mpsc::channel();
        let (progress_tx, progress_rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
        thread::spawn(move || loop {
            // let mut time_pos: i64 = 0;
            // let mut paused = false;
            loop {
                if let Ok(player_command) = rx.try_recv() {
                    match player_command {
                        PlayerCommand::Play(song) => {
                            // time_pos = 0;
                            // Create a hint to help the format registry guess what format reader is appropriate.
                            let mut hint = Hint::new();
                            let source = {
                                // Othwerise, get a Path from the path string.
                                let path = Path::new(&song);

                                // Provide the file extension as a hint.
                                if let Some(extension) = path.extension() {
                                    if let Some(extension_str) = extension.to_str() {
                                        hint.with_extension(extension_str);
                                    }
                                }

                                Box::new(File::open(path).unwrap())
                            };
                            // Create the media source stream using the boxed media source from above.
                            let mss = MediaSourceStream::new(
                                source,
                                symphonia::core::io::MediaSourceStreamOptions::default(),
                            );

                            // Use the default options for format readers other than for gapless playback.
                            let format_opts = FormatOptions {
                                ..symphonia::core::formats::FormatOptions::default()
                            };

                            // Use the default options for metadata readers.
                            let metadata_opts: MetadataOptions =
                                symphonia::core::meta::MetadataOptions::default();

                            // Get the value of the track option, if provided.
                            let track: Option<usize> = None;

                            // Probe the media source stream for metadata and get the format reader.
                            match symphonia::default::get_probe().format(
                                &hint,
                                mss,
                                &format_opts,
                                &metadata_opts,
                            ) {
                                Ok(probed) => {
                                    let result = {
                                        // Playback mode.
                                        // print_format(path_str, &mut probed);

                                        // If present, parse the seek argument.
                                        let seek_time = Some(0.0);

                                        // Set the decoder options.
                                        let decode_opts = DecoderOptions {
                                            ..symphonia::core::codecs::DecoderOptions::default()
                                        };

                                        // Play it!
                                        play(probed.format, track, seek_time, &decode_opts, false)
                                    };

                                    if let Err(err) = result {
                                        println!("error: {}", err);
                                    }
                                }
                                Err(err) => {
                                    // The input was not supported by any format reader.
                                    println!("file not supported. reason? {}", err);
                                }
                            }
                        }
                        PlayerCommand::Stop => {}
                        PlayerCommand::VolumeUp => {}
                        PlayerCommand::VolumeDown => {}
                        PlayerCommand::Pause(_pause_or_resume) => {}
                        PlayerCommand::Progress => {} // PlayerCommand::Seek(pos) => {}
                    }
                }
                // if !paused {
                //     time_pos += 1;
                // }
                sleep(Duration::from_secs(1));
            }
        });
        Self {
            sender: tx,
            progress_receiver: progress_rx,
            paused: false,
            volume: 50,
            // receiver: rx,
        }
    }
}

impl GeneralP for Symphonia {
    fn add_and_play(&mut self, song: &str) {
        self.sender.send(PlayerCommand::Play(song.to_string())).ok();
    }

    fn volume(&self) -> i32 {
        self.volume
    }

    fn volume_up(&mut self) {
        self.volume = cmp::min(self.volume + 5, 100);
    }

    fn volume_down(&mut self) {
        self.volume = cmp::max(self.volume - 5, 0);
    }
    fn set_volume(&mut self, mut volume: i32) {
        if volume > 100 {
            volume = 100;
        } else if volume < 0 {
            volume = 0;
        }
        self.volume = volume;
    }

    fn pause(&mut self) {}

    fn resume(&mut self) {}

    fn is_paused(&mut self) -> bool {
        false
    }

    fn seek(&mut self, _secs: i64) -> Result<()> {
        Ok(())
    }

    fn get_progress(&mut self) -> Result<(f64, i64, i64)> {
        Ok((0.9, 0, 100))
    }
}

#[derive(Copy, Clone)]
struct PlayTrackOptions {
    track_id: u32,
    seek_ts: u64,
}

fn play(
    mut reader: Box<dyn FormatReader>,
    track_num: Option<usize>,
    seek_time: Option<f64>,
    decode_opts: &DecoderOptions,
    no_progress: bool,
) -> SymphoniaResult<()> {
    // If the user provided a track number, select that track if it exists, otherwise, select the
    // first track with a known codec.
    let track = track_num
        .and_then(|t| reader.tracks().get(t))
        .or_else(|| first_supported_track(reader.tracks()));

    let mut track_id = match track {
        Some(track) => track.id,
        _ => return Ok(()),
    };

    // If there is a seek time, seek the reader to the time specified and get the timestamp of the
    // seeked position. All packets with a timestamp < the seeked position will not be played.
    //
    // Note: This is a half-baked approach to seeking! After seeking the reader, packets should be
    // decoded and *samples* discarded up-to the exact *sample* indicated by required_ts. The
    // current approach will discard excess samples if seeking to a sample within a packet.
    let seek_ts = seek_time.map_or(0, |time| {
        let seek_to = SeekTo::Time {
            time: Time::from(time),
            track_id: Some(track_id),
        };

        // Attempt the seek. If the seek fails, ignore the error and return a seek timestamp of 0 so
        // that no samples are trimmed.
        match reader.seek(SeekMode::Accurate, seek_to) {
            Ok(seeked_to) => seeked_to.required_ts,
            Err(Error::ResetRequired) => {
                // print_tracks(reader.tracks());
                track_id = first_supported_track(reader.tracks()).unwrap().id;
                0
            }
            Err(err) => {
                // Don't give-up on a seek error.
                println!("seek error: {}", err);
                0
            }
        }
    });

    // let seek_ts = 0;
    // The audio output device.
    let mut audio_output = None;

    let mut track_info = PlayTrackOptions { track_id, seek_ts };

    let result = loop {
        match play_track(
            &mut reader,
            &mut audio_output,
            track_info,
            decode_opts,
            no_progress,
        ) {
            Err(Error::ResetRequired) => {
                // The demuxer indicated that a reset is required. This is sometimes seen with
                // streaming OGG (e.g., Icecast) wherein the entire contents of the container change
                // (new tracks, codecs, metadata, etc.). Therefore, we must select a new track and
                // recreate the decoder.
                // print_tracks(reader.tracks());

                // Select the first supported track since the user's selected track number might no
                // longer be valid or make sense.
                let track_id = first_supported_track(reader.tracks()).unwrap().id;
                track_info = PlayTrackOptions {
                    track_id,
                    seek_ts: 0,
                };
            }
            res => break res,
        }
    };

    // Flush the audio output to finish playing back any leftover samples.
    if let Some(audio_output) = audio_output.as_mut() {
        audio_output.flush();
    }

    result
}

fn play_track(
    reader: &mut Box<dyn FormatReader>,
    audio_output: &mut Option<Box<dyn output::Audio>>,
    play_opts: PlayTrackOptions,
    decode_opts: &DecoderOptions,
    _no_progress: bool,
) -> SymphoniaResult<()> {
    // Get the selected track using the track ID.
    let track = match reader
        .tracks()
        .iter()
        .find(|track| track.id == play_opts.track_id)
    {
        Some(track) => track,
        _ => return Ok(()),
    };

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, decode_opts)?;

    // Get the selected track's timebase and duration.
    // let _tb = track.codec_params.time_base;
    let _dur = track
        .codec_params
        .n_frames
        .map(|frames| track.codec_params.start_ts + frames);

    // Decode and play the packets belonging to the selected track.
    let result = loop {
        // Get the next packet from the format reader.
        let packet = match reader.next_packet() {
            Ok(packet) => packet,
            Err(err) => break Err(err),
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != play_opts.track_id {
            continue;
        }

        //Print out new metadata.
        while !reader.metadata().is_latest() {
            reader.metadata().pop();

            if let Some(_rev) = reader.metadata().current() {
                // print_update(rev);
            }
        }

        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(decode_result) => {
                // If the audio output is not open, try to open it.
                if audio_output.is_none() {
                    // Get the audio buffer specification. This is a description of the decoded
                    // audio buffer's sample format and sample rate.
                    let spec = *decode_result.spec();

                    // Get the capacity of the decoded buffer. Note that this is capacity, not
                    // length! The capacity of the decoded buffer is constant for the life of the
                    // decoder, but the length is not.
                    let duration = decode_result.capacity() as u64;

                    // Try to open the audio output.
                    audio_output.replace(output::try_open(spec, duration).unwrap());
                } else {
                    // TODO: Check the audio spec. and duration hasn't changed.
                }

                // Write the decoded audio samples to the audio output if the presentation timestamp
                // for the packet is >= the seeked position (0 if not seeking).
                // if packet.ts() >= play_opts.seek_ts {
                //     if !no_progress {
                //         // print_progress(packet.ts(), dur, tb);
                //     }

                if let Some(audio_output) = audio_output {
                    audio_output.write(decode_result).unwrap();
                }
                // }
            }
            Err(Error::DecodeError(err)) => {
                // Decode errors are not fatal. Print the error message and try to decode the next
                // packet as usual.
                println!("decode error: {}", err);
            }
            Err(err) => break Err(err),
        }
    };

    // Regardless of result, finalize the decoder to get the verification result.
    // let finalize_result = decoder.finalize();

    // if let Some(verify_ok) = finalize_result.verify_ok {
    //     if verify_ok {
    //         println!("verification passed");
    //     } else {
    //         println!("verification failed");
    //     }
    // }

    result
}

fn first_supported_track(tracks: &[Track]) -> Option<&Track> {
    tracks
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
}
