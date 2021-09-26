# Terminal Music Player written in Rust

Nowadays listen to favorite songs are not easy. For online services, the copyrights
are owned by several different softwares and websites. Local player becomes the best choice.

As a contributor of GOMU(https://github.com/issadarkthing/gomu), I met serious problems during 
development. The main problem is data race condition. So I basically rewrite the player in rust, 
and hope to solve the problem.

As for now, mp3, m4a, flac and ogg/vorbis are supported.

![main](https://github.com/tramhao/termusic/blob/master/screenshots/main.png?raw=true)
![tageditor](https://github.com/tramhao/termusic/blob/master/screenshots/tageditor.png?raw=true)

## Requirement:
Need [gstreamer](https://gstreamer.freedesktop.org/documentation/installing/on-linux.html?gi-language=c) and related plugins installed to play musics. Please check below:
```
gstreamer
gst-libav
gstreamer-plugins-base(gst-plugins-base)
gstreamer-plugins-good(gst-plugins-good)
gstreamer-plugins-bad(gst-plugins-bad)
gstreamer-plugins-ugly(gst-plugins-ugly)
for gentoo:
gst-plugins-meta
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
or if you need mpris support:
```
make mpris
~/.local/share/cargo/bin/termusic
```

### Distro Packages

#### Arch Linux

Arch Linux users can install `termusic` from the [AUR](https://aur.archlinux.org/) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example:

```
paru termusic
```

## ChangeLog

### [v0.3.14]
- Released on: Sep , 2021.
- Minor fix: popup message will display for 5 seconds. no message overlapping each other.
- New: search function in playlist and queue. keybinding: "/".
- Fix: All lrc files was merged into mp3 after downloading. Should be distinguished by file name.
- Fix: play any folder with command line args.
- Fix: spamming mpris propertieschanged messages.

### [v0.3.13]
- Released on: Sep 23, 2021.
- New: mpris support(optional). use "make mpris" to compile and install it.
- Show a message when start playing a song.
- Remove the usage of msgbox component and use paragraph instead.
- press "N" for previous song.


### [v0.3.12]
- Released on: Sep 15, 2021.
- Minor fix: wrong hints for empty queue.
- Load queue faster.
- Remove dependency of openssl.
- Remove dependency of urlqstring.

### [v0.3.11]
- Released on: Sep 13, 2021.
- Load faster by loading queue after app start.
- Remove dependency of ogg-metadata.
- Display version info in both tui and cli.
- Could override music directory with command line arguments.

### [v0.3.10]
- Released on: Sep 11, 2021.
- New: ogg vorbis format support.

### [v0.3.9]
- Released on: Sep 7, 2021.
- New: flac format support.
- Refactor invidious.rs.

### [v0.3.8]
- Released on: Sep 4, 2021.
- Remove the usage of configr, also make the app minimal.
- Change hotkey for help to Ctrl+h.
- Rearrange components in tag editor.
- Embed duration into tag. Load faster.
- Remove dependency of mp3-duration.
- Minor fix: lyric lang_ext was set to wrong names.

### [v0.3.7] 
- Released on: Sep 2, 2021.
- Fix table focus issue.
- Fix invidious search slow down the whole program.
- Change http client from reqwest to ureq, to make the app minimal, thus speed up compilation.

### [v0.3.6] 
- Released on: Aug 31, 2021.
- Remove the dependency of unicode truncate, as tui-realm-stdlib implemented width for table.
- Fix playlist sorting with characters of mixed languages.
- Speed up load_queue and sort playlist.
- Minor fix: duration display in queue.

### [v0.3.5] 
- Released on: Aug 26, 2021.
- Refactoring status line, to show download success or fail message.
- Parsing output of youtube-dl to select downloaded song in playlist.
- Embed all lyrics after youtube download. Switch lyric with "T" key while playing.
- Show popup messages on top right corner.
- Sort file name(including chinese) in tree.
- Can delete single lyric from tag editor.
- Currently only mp3 support several lyrics.


### [v0.3.4] 
- Released on: Aug 24 2021. 
- Refactoring lyric mod to songtag mod.
- Run songtag search in threads so it'll not block tageditor.
- Refactoring youtube_options and no more search error with youtube.

### [v0.3.3] 
- Released on: August 21, 2021.
- Run songtag search in parallel to save some time in tageditor.

## Implemented features(changelog before v0.3.3):
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
