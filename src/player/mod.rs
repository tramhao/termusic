pub mod gst;
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
// mod mpv;
// mod rodio_player;
// mod vlc_player;
use crate::song::Song;
// use mpv::MPVAudioPlayer;
// use rodio_player::RodioPlayer;
use gst::GSTPlayer;
// use vlc_player::VLCAudioPlayer;
// use anyhow::{anyhow, Result};
use anyhow::Result;

#[allow(non_camel_case_types, unused, clippy::upper_case_acronyms)]
pub enum PlayerType {
    // MPV,
    // VLC,
    // RODIO,
    GST,
}

// pub fn choose_player(song: Song) -> Result<PlayerType> {
//     match song.ext.as_ref().unwrap().as_str() {
//         "mp3" => return Ok(PlayerType::mp3),
//         "m4a" => return Ok(PlayerType::m4a),
//         &_ => return Err(anyhow!("Unsupported")),
//     }
// }

pub trait AudioPlayer {
    fn queue_and_play(&mut self, new: Song);
    fn volume(&mut self) -> i64;
    fn volume_up(&mut self);
    fn volume_down(&mut self);
    fn pause(&mut self);
    fn resume(&mut self);
    fn is_paused(&mut self) -> bool;
    fn seek(&mut self, secs: i64) -> Result<()>;
    fn get_progress(&mut self) -> (f64, i64, i64);
}

pub struct Player {
    // pub mpv_player: MPVAudioPlayer,
    // pub vlc_player: VLCAudioPlayer,
    pub gst_player: GSTPlayer,
    pub player_type: PlayerType,
    // pub rodio_player: RodioPlayer,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            // mpv_player: MPVAudioPlayer::new(),
            // vlc_player: VLCAudioPlayer::new(),
            // rodio_player: RodioPlayer::new(),
            gst_player: GSTPlayer::new(),
            // player_type: PlayerType::VLC,
            // player_type: PlayerType::MPV,
            player_type: PlayerType::GST,
            // player_type: PlayerType::RODIO,
        }
    }
}

// impl Player {
//     pub fn new(song: Song) -> Result<dyn AudioPlayer> {
//         let player_type = choose_player(song)?;
//         match player_type {
//             PlayerType::mp3 => {return MPVAudioPlayer::new()}
//             PlayerType::m4a => {}
//         }

//     }
// }
impl AudioPlayer for Player {
    fn queue_and_play(&mut self, new: Song) {
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.queue_and_play(new),
            // PlayerType::VLC => self.vlc_player.queue_and_play(new),
            // PlayerType::RODIO => self.rodio_player.queue_and_play(new),
            PlayerType::GST => self.gst_player.queue_and_play(new),
        }
    }
    fn volume(&mut self) -> i64 {
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.volume(),
            // PlayerType::RODIO => self.rodio_player.volume(),
            // _ => 0,
            // PlayerType::VLC => self.vlc_player.volume(),
            PlayerType::GST => self.gst_player.volume(),
        }
    }
    fn volume_up(&mut self) {
        #[allow(clippy::single_match)]
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.volume_up(),
            // PlayerType::RODIO => self.rodio_player.volume_up(),
            // PlayerType::VLC => self.vlc_player.volume_up(),
            PlayerType::GST => self.gst_player.volume_up(),
            // _ => {}
        }
    }
    fn volume_down(&mut self) {
        #[allow(clippy::single_match)]
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.volume_down(),
            // PlayerType::RODIO => self.rodio_player.volume_down(),
            // _ => {}
            // PlayerType::VLC => self.vlc_player.volume_down(),
            PlayerType::GST => self.gst_player.volume_down(),
        }
    }
    fn pause(&mut self) {
        #[allow(clippy::single_match)]
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.pause(),
            // PlayerType::RODIO => self.rodio_player.pause(),
            // _ => {}
            // PlayerType::VLC => self.vlc_player.pause(),
            PlayerType::GST => self.gst_player.pause(),
        }
    }
    fn resume(&mut self) {
        #[allow(clippy::single_match)]
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.resume(),
            // PlayerType::RODIO => self.rodio_player.resume(),
            // _ => {}
            // PlayerType::VLC => self.vlc_player.resume(),
            PlayerType::GST => self.gst_player.resume(),
        }
    }
    fn is_paused(&mut self) -> bool {
        #[allow(clippy::single_match)]
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.is_paused(),
            // PlayerType::RODIO => self.rodio_player.is_paused(),
            // _ => true,
            // PlayerType::VLC => self.vlc_player.is_paused(),
            PlayerType::GST => self.gst_player.is_paused(),
        }
    }
    fn seek(&mut self, secs: i64) -> Result<()> {
        #[allow(clippy::single_match)]
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.seek(secs),
            // PlayerType::RODIO => self.rodio_player.seek(secs),
            // _ => Ok(()),
            // PlayerType::VLC => self.vlc_player.seek(secs),
            PlayerType::GST => self.gst_player.seek(secs),
        }
    }
    fn get_progress(&mut self) -> (f64, i64, i64) {
        match self.player_type {
            // PlayerType::MPV => self.mpv_player.get_progress(),
            // PlayerType::VLC => self.vlc_player.get_progress(),
            // PlayerType::RODIO => self.rodio_player.get_progress(),
            PlayerType::GST => self.gst_player.get_progress(),
        }
    }
}
