# Terminal Music Player written in Rust

Listen to music freely as both in freedom and free of charge!

Freedom: As time goes by, online service providers controls pretty much everything we listen.
Complicated copyright issues make things worse. If my favorite song cannot be found in a website, 
probably I'll just not listen to them for years.

Free of charge: you can download from youtube, netease,migu and kugou for free. No need to 
register monthly paid membership for several websites.

As a contributor of GOMU(https://github.com/issadarkthing/gomu), I met serious problems during 
development. The main problem is data race condition. So I basically rewrite the player in rust, 
and hope to solve the problem.

As for now, mp3, m4a, flac, wav and ogg/vorbis are supported.

By the way, for mobile devices, I recommend sync your music library with mobile by verysync and 
listen to them with vinyl(which I contributed also).

![main](https://github.com/tramhao/termusic/blob/master/screenshots/main.png?raw=true)
![tageditor](https://github.com/tramhao/termusic/blob/master/screenshots/tageditor.png?raw=true)

## Requirement:
Need [gstreamer](https://gstreamer.freedesktop.org/documentation/installing/on-linux.html?gi-language=c) and related plugins installed to play music. Please check below:
```
gstreamer
gst-libav
gstreamer-plugins-base(gst-plugins-base)
gstreamer-plugins-good(gst-plugins-good)
gstreamer-plugins-bad(gst-plugins-bad)
gstreamer-plugins-ugly(gst-plugins-ugly)
(Additonally for gentoo:)
gst-plugins-meta
(Additionally for macos:)
gstreamer-audio
gstreamer-sys
```
Optionally you need [youtube-dl](https://ytdl-org.github.io/youtube-dl/download.html) and ffmpeg installed to download mp3 from youtube.

On windows, please download and install gstreamer development package from https://gstreamer.freedesktop.org/download/.

## Installation:
```
cargo install termusic
```
Or manually:
```
git clone https://github.com/tramhao/termusic.git
cd termusic
make
```
Then install with:
```
make install
```
Or if you need dbus mpris support(need to have dbus installed):
```
make mpris
```
By default, termusic can display album cover in kitty or iterm2(mac, not tested). If you need album cover displayed on other terminals, please install ueberzug(https://github.com/seebye/ueberzug) by `pip3 install ueberzug` or `paru -S ueberzug`, then:
```
make cover
```
If you need both mpris and cover, you can:
```
make full
```
Finally, you can run it with:
```
~/.local/share/cargo/bin/termusic
```
You can copy it anywhere in your `$PATH`. The configuration file is located in `~/.config/termusic/config.toml`. However, as this is a minimalistic program, you don't need to edit the configuration file and everything can be set from the app.
For example, you can change music dir with enter and backspace, and it'll be saved to config. You can change loop mode with m key and it'll be saved too. You can change volume with +/-, and it'll be saved also.

You need stable branch rust toolchain installed to build it. I'm building with 1.56(rust edition 2021). According to 
user feedback, version less than 1.52 is not working.

### Distro Packages

#### Arch Linux

Arch Linux users can install `termusic` from the [AUR](https://aur.archlinux.org/) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example:

```
paru termusic
```
#### NetBSD

NetBSD users can install `termusic` from the official repositories.
```
pkgin install termusic
```

## ChangeLog

### [v0.6.3]
- Released on: Dec , 2021.
- New: color theme support. Shift+C to open color editor. You can change the whole theme, or edit the specific color. The themes are from alacritty-themes, and are localed in `~/.config/termusic/themes/` folder.


### [v0.6.2]
- Released on: Dec 12, 2021.
- change icons on playlist title.
- New: search works in playlist also.

### [v0.6.1]
- Release on: Dec 3, 2021.
- fix: bug when using kitty, and there is a 1/5 chances that will lead to freeze when start the app.
- fix: when start the app, the first song in playlist will be skipped.

### [v0.6.0]
- Released on: Dec 1,2021.
- Update to tui-realm 1.0. Please note, as it's basically a rewrite, small bugs are inevitable. Please report it in issues and I'll respond very fast.
- Hotkey to quit changed from `Q` to `q`, as now there will show a popup confirmation so it's unlikely to quit accidentally.
- Can add a song to the beginning of playlist or the end. Switch by `a`. Note: I add this to configuration file, and it'll reset your configuration file to default values. Please backup if you need. Basically it's not necessary as all options can be set from inside termusic.

### [v0.5.0]
- Released on: Oct 15, 2021.
- New: album photo for all kinds of terminals. Alacritty,kitty and st tested. Require install ueberzug.

### [v0.3.17]
- Released on: Oct 10, 2021.
- Added rust-toolchain.toml to avoid using nightly toolchain.
- iterm2 album photo support. 
- Minor fix: playingbar title length limit.
- Refactor mpris to operate faster(100ms).

### [v0.3.16]
- Released on: Oct 1, 2021.
- Rename playlist to library, and queue to playlist.
- New: loop mode configuration. Default is queue mode(aka consume mode), can switch to loop mode and single loop mode by pressing "m" key when focusing Playlist. In queue mode, previous song cannot be played as it's already consumed from the playlist. In single loop mode, previous song will be ignored.
- Show volume in progress bar title.


### [v0.3.15]
- Released on: Sep 27, 2021.
- Revert mpris to optional as some users don't have dbus installed( NetBSD and MacOs).

### [v0.3.14]
- Released on: Sep 27 , 2021.
- Minor fix: popup message will display for 5 seconds. no message overlapping each other.
- New: search in playlist. Key binding: "/".
- New: wav file support.
- Fix: All lrc files was merged into mp3 after downloading. Should be distinguished by file name.
- Fix: play any folder with command line args.
- Fix: spamming mpris propertieschanged messages. Thus mpris is default now.

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
- [alacritty-themes](https://github.com/rajasegar/alacritty-themes)

## License
GPLv3 for netease api code under src/lyric/netease.
MIT License for other code.
