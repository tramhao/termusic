[![Build status](https://github.com/nirusu99/ytd-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/nirusu99/ytd-rs/actions)
[![crates.io](https://img.shields.io/crates/v/ytd-rs.svg)](https://crates.io/crates/termusic)
[![dependency status](https://deps.rs/repo/github/tramhao/termusic/status.svg)](https://deps.rs/repo/github/tramhao/termusic)

# Terminal Music Player written in Rust

Listen to music freely as both in freedom and free of charge!

**Freedom**: As time goes by, online service providers control pretty much everything we listen to.
Complicated copyright issues make things worse. If my favorite song cannot be found on a website, 
I'll probably just not listen to them for years.

**Free of charge**: You can download from Youtube, NetEase, Migu and KuGou for free. No need to 
register for monthly paid memberships.

As a contributor of [GOMU](https://github.com/issadarkthing/gomu), I met serious problems during 
development. The main problem is data race condition. So I rewrote the player in rust, and hope to
solve the problem.

As for now, MP3, M4A, FLAC, AIFF,WAV, Opus and OGG Vorbis are supported. For some formats not supported, 
will still try to play without metadata showing. Default decoding backend is symphonia, which is a
pure rust implementation. Alternatively, you can use mpv or gst as backend when compiling.

| Format                   |  Default(symphonia) | Mpv     | Gstreamer | Metadata  |
|--------------------------|---------------------|---------|-----------|-----------|
| AAC-LC                   |  Yes                | Yes     | Yes       | Yes       |
| AIFF                     |  No                 | Yes     | Yes       | Yes       |
| ALAC                     |  No                 | Yes     | Yes       | Yes       |
| FLAC                     |  Yes                | Yes     | Yes       | Yes       |
| M4a                      |  Yes                | Yes     | Yes       | Yes       |
| MP3                      |  Yes                | Yes     | Yes       | Yes       |
| Opus                     |  No                 | Yes     | Yes       | Yes       |
| PCM                      |  Yes                | Yes     | Yes       | Yes       |
| Ogg Vorbis               |  Yes                | Yes     | Yes       | Yes       |
| Wav                      |  Yes                | Yes     | Yes       | Yes       |
| WebM                     |  No                 | Yes     | Yes       | No        |


By the way, for mobile devices, I recommend sync your music library with mobile with `verysync` and 
listen to them with [Vinyl Music Player](https://github.com/AdrienPoupa/VinylMusicPlayer).

![main](https://github.com/tramhao/termusic/blob/master/screenshots/main.png?raw=true)
![tageditor](https://github.com/tramhao/termusic/blob/master/screenshots/tageditor.png?raw=true)

## Installation

### Requirements

You will need to build with the stable rust toolchain. Version 1.58 is tested, and according to
user feedback, versions below 1.52 do not work.

You will need alsa installed to support decoding with symphonia. Note that on Linux, the ALSA development files are required. These are provided as part of the `libasound2-dev` package on Debian and Ubuntu distributions and `alsa-lib-devel` on Fedora.

Optionally, if you build with feature gate gst, you will need [GStreamer](https://gstreamer.freedesktop.org) and related plugins installed to play music.

Optionally, if you build with feature gate mpv, you will need [MPV](https://mpv.io/) installed to compile and play music. In this case, you don't need gstreamer.

#### Linux

##### Ubuntu

See [here](https://gstreamer.freedesktop.org/documentation/installing/on-linux.html?gi-language=c#install-gstreamer-on-ubuntu-or-debian)

##### Arch Linux

```bash
pacman -S gstreamer gst-libav gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly
```

##### Gentoo

```bash
emerge gstreamer gst-plugins-libav gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-plugins-meta
```

#### MacOS

See [here](https://gstreamer.freedesktop.org/download/#macos)

#### Windows

See [here](https://gstreamer.freedesktop.org/download/#windows).

#### Yt-dlp support

You can optionally install [yt-dlp](https://github.com/yt-dlp/yt-dlp/) and [FFmpeg](https://www.ffmpeg.org/download.html) to download MP3s from Youtube.


### Distro Packages

#### Arch Linux

Arch Linux users can install `termusic` from the [AUR](https://aur.archlinux.org/) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example:

```bash
paru termusic
```

#### NetBSD

NetBSD users can install `termusic` from the official repositories.

```bash
pkgin install termusic
```

### Cargo

```bash
cargo install termusic
```

### From Source

```bash
git clone https://github.com/tramhao/termusic.git
cd termusic
make
```

Then install with:

```bash
make install
```

Or if you need dbus mpris support (you will need to have dbus installed):

```bash
make mpris
```

By default, termusic can display album covers in Kitty or iTerm2 (mac, not tested).
If you need album covers displayed on other terminals, please install [ueberzug](https://github.com/seebye/ueberzug), then:

```bash
make cover
```

If you need mpris, cover, and yt-dlp, do:

```bash
make full
```

Finally, you can run it with:

```bash
~/.local/share/cargo/bin/termusic
```

You can copy it anywhere in your `$PATH`. The configuration file is located in `~/.config/termusic/config.toml`.
However, as this is a minimalistic program, you don't need to edit the configuration file and almost everything can be set from the app.

## TODO
- [x] key editor.
- [x] symphonia backend.
- [ ] Better interface to adjust timestamp of lyric.
- [ ] Database?.


## Thanks
- [tui-realm](https://github.com/veeso/tui-realm) 
- [termscp](https://github.com/veeso/termscp)
- [netease-cloud-music-gtk](https://github.com/gmg137/netease-cloud-music-gtk)
- [alacritty-themes](https://github.com/rajasegar/alacritty-themes)

## License

GPLv3 for NetEase api code under `src/lyric/netease`.
MIT License for other code.
