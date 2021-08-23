# Terminal Music Player written in Rust

Nowadays listen to favorite songs are not easy. For online services, the copyrights
are owned by several different softwares and websites. Local player becomes the best choice.

As a contributor of GOMU(https://github.com/issadarkthing/gomu), I met serious problems during 
development. The main problem is data race condition. So I basically rewrite the player in rust, 
and hope to solve the problem.

As for now, mp3 and m4a are supported. m4a is not fully tested as I have no itune musics downloaded.
Please help testing it if possible.

![main](https://github.com/tramhao/termusic/blob/master/screenshots/main.png?raw=true)
![tageditor](https://github.com/tramhao/termusic/blob/master/screenshots/tageditor.png?raw=true)

## Requirement:
Need [gstreamer](https://gstreamer.freedesktop.org/documentation/installing/on-linux.html?gi-language=c) and related plugins installed to play mp3/m4a. Please check below:
```
gstreamer
gstreamer-plugins-base(gst-plugins-base)
gstreamer-plugins-good(gst-plugins-good)
gstreamer-plugins-bad(gst-plugins-bad)
gstreamer-plugins-ugly(gst-plugins-ugly)
```
Optionally you need [youtube-dl](https://ytdl-org.github.io/youtube-dl/download.html) installed to download mp3 from youtube.

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

## ChangeLog

### [0.3.4] 
- Released on: . 
- Refactoring lyric mod to songtag mod.

## Implemented features:
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
- [x] Local service for kugou v0.2.10
- [x] Youtube-dl progress indication(indicated by status line)
- [x] Youtube search by invidious V0.2.7(from the same dialogue of download)
- [x] Local service for migu v0.2.8
- [x] m4a format support v0.2.12
- [x] switch to Gstreamer playing backend in order to support m4a v0.2.12
- [x] m4a meta support v0.3.0
- [x] Invidious servers are random selected and verified, thus no configuration is needed.v0.3.2

## Thanks for
- [tui-realm](https://github.com/veeso/tui-realm) 
- [termscp](https://github.com/veeso/termscp)
- [netease-cloud-music-gtk](https://github.com/gmg137/netease-cloud-music-gtk)

## License
GPLv3 for netease api code under src/lyric/netease.
MIT License for other code.
