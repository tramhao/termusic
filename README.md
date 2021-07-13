# Terminal Music Player written in Rust

As a contributor of GOMU(https://github.com/issadarkthing/gomu), I met serious problems during development. The main problem is data race condition. So I basically rewrite the player in rust, and hope to solve the problem.

Currently the project is still young, but working.
![name](https://github.com/tramhao/termusic/blob/master/screenshots/main.png?raw=true)
![tageditor](https://github.com/tramhao/termusic/blob/master/screenshots/tageditor.png?raw=true)

## Requirement:
Need mpv installed to play mp3.
Optionally need youtube-dl installed to download mp3 from youtube.

## Installation:
```
cargo install termusic
```
Or install manually:
```
git clone https://github.com/tramhao/termusic.git
cd termusic
make
make install
~/.local/share/cargo/bin/termusic
```

## ChangeLog/Implemented features:
- [x] Music library below ~/Music, can be changed via editing $HOME/.config/termusic/config.toml
- [x] Pause/Skip
- [x] Seek forward/backward
- [x] USLT lyric
- [x] Album Photo display(only for kitty terminal)
- [x] Youtube-dl integration
- [x] lyric and tag download
- [x] yank and paste in playlist
- [x] Lyric offset adjustment 
- [x] Local service to fetch lyrics
- [x] Download song in tag editor
- [x] Configuration v0.2.6
- [ ] Local service for kugou v0.2.7
- [ ] Youtube-dl progress indication
- [ ] Youtube search by invidious
- [ ] Database instead of id3
- [ ] more player backend

## License
MIT License
