use super::{Status, TermusicActivity};
use crate::souvlaki::MediaControlEvent;

impl TermusicActivity {
    pub fn mpris_handler(&mut self, e: MediaControlEvent) {
        match e {
            MediaControlEvent::Next => {
                self.next_song();
            },
            MediaControlEvent::Previous => {
                self.previous_song();
            },
            MediaControlEvent::Pause => {
                self.player.pause();
            },
            MediaControlEvent::Toggle => {
                if self.player.is_paused() {
                    self.status = Some(Status::Running);
                    self.player.resume();
                } else {
                    self.status = Some(Status::Paused);
                    self.player.pause();
                }
            },
            MediaControlEvent::Play => {
                self.player.resume();
            },
            // MediaControlEvent::Seek(x) => match x {
            //     SeekDirection::Forward => activity.player.seek(5).ok(),
            //     SeekDirection::Backward => activity.player.seek(-5).ok(),
            // },
            // MediaControlEvent::SetPosition(position) => {
            //     let _position = position. / 1000;
            // }
            MediaControlEvent::OpenUri(uri) => {
                self.player.add_and_play(&uri);
            },
            _ => {},
        }
    }

    pub fn update_mpris(&mut self) {
        if let Ok(m) = self.player.rx.try_recv() {
            self.mpris_handler(m);
        }
    }
}
