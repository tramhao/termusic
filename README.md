# Terminal Music Player written in Rust

Nowadays listen to favorite songs are not easy. For online services, the copyrights
are owned by several different softwares and websites. Local player becomes the best choice.

As a contributor of GOMU(https://github.com/issadarkthing/gomu), I met serious problems during 
development. The main problem is data race condition. So I basically rewrite the player in rust, 
and hope to solve the problem.

As for now, only mp3 is supported. It's basically because mp3 has id3 tags and these tags played 
an important role in the app.

![main](https://github.com/tramhao/termusic/blob/master/screenshots/main.png?raw=true)
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
- [x] Download song in tag editor(only for netease, not for kugou)
- [x] Configuration v0.2.6
- [ ] Local service for kugou v0.2.7(only finished lyric, album_photo and song is not finished)
- [x] Youtube-dl progress indication(indicated by status line)
- [x] Youtube search by invidious V0.2.7(from the same dialogue of download)
- [x] Local service for migu v0.2.8
- [ ] Database instead of id3
- [ ] more player backend

## Thanks for
- [tui-realm](https://github.com/veeso/tui-realm) 
- [termscp](https://github.com/veeso/termscp)
- [netease-cloud-music-gtk](https://github.com/gmg137/netease-cloud-music-gtk)

## License
GPLv3 for netease api code under src/lyric/netease.
MIT License for other code.
