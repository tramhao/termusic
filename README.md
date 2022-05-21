[![Build status](https://github.com/tramhao/termusic/actions/workflows/build.yml/badge.svg)](https://github.com/tramhao/termusic/actions)
[![crates.io](https://img.shields.io/crates/v/termusic.svg)](https://crates.io/crates/termusic)
[![dependency status](https://deps.rs/repo/github/tramhao/termusic/status.svg)](https://deps.rs/repo/github/tramhao/termusic)

# Terminal Music Player written in Rust

Listen to music freely as both in freedom and free of charge!

<table>
    <tr>
        <td>
            <img src="https://github.com/tramhao/termusic/blob/master/screenshots/main.png?raw=true" alt="Main view" style="width: 500px;"/>
        </td>
        <td>
            <img src="https://github.com/tramhao/termusic/blob/master/screenshots/tageditor.png?raw=true" alt="Tag editor" style="width: 500px;"/>
        </td>
    </tr>
</table>

**Freedom**: As time goes by, online service providers control pretty much everything we listen to.
Complicated copyright issues make things worse. If my favorite song cannot be found on a website, 
I'll probably just not listen to them for years.

**Free of charge**: You can download from YouTube, NetEase, Migu and KuGou for free. No need to 
register for monthly paid memberships.

As a contributor of [GOMU](https://github.com/issadarkthing/gomu), I met serious problems during 
development. The main problem is data race condition. So I rewrote the player in rust, and hope to
solve the problem.

## Supported Formats

Below are the audio formats supported by the various backends.

In the case that metadata is not supported, an attempt will still be made to play the file.

| Format (`feature`) | Symphonia (`default`) | Mpv (`mpv`) | Gstreamer (`gst`) | Metadata |
|--------------------|-----------------------|-------------|-------------------|----------|
| ADTS               | Yes                   | Yes         | Yes               | No       |
| AIFF               | No                    | Yes         | Yes               | Yes      |
| FLAC               | Yes                   | Yes         | Yes               | Yes      |
| M4a                | Yes                   | Yes         | Yes               | Yes      |
| MP3                | Yes                   | Yes         | Yes               | Yes      |
| Opus               | No                    | Yes         | Yes               | Yes      |
| Ogg Vorbis         | Yes                   | Yes         | Yes               | Yes      |
| Wav                | Yes                   | Yes         | Yes               | Yes      |
| WebM               | No                    | Yes         | Yes               | No       |

## Installation

### Requirements

You will need to build with the stable rust toolchain. Minimal Supported Rust Version 1.59.0.


=======
| Backend   | Requirements                                                                                                                                                                                                                                                                       |
|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Symphonia | You will need [ALSA](https://alsa-project.org) installed to support decoding with symphonia.<br />Note that the ALSA development files are required. These are provided as part of the `libasound2-dev` package on Debian and Ubuntu distributions and `alsa-lib-devel` on Fedora. |
| GStreamer | [GStreamer](https://gstreamer.freedesktop.org)                                                                                                                                                                                                                                     |
| MPV       | [MPV](https://mpv.io/)                                                                                                                                                                                                                                                             |


#### Yt-dlp support

You can optionally install [yt-dlp](https://github.com/yt-dlp/yt-dlp/) and [FFmpeg](https://www.ffmpeg.org/download.html) to download MP3s from Youtube.

### Packages

Do note that these will be compiled with the **symphonia** backend.

#### Arch Linux

Arch Linux users can install `termusic` from the [AUR](https://aur.archlinux.org/) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers).

```bash
paru termusic
```

#### NetBSD

NetBSD users can install `termusic` from the official repositories.

```bash
pkgin install termusic
```
#### Nix/NixOS

Either in the user's environment:

```bash
nix-env --install termusic
```

Or declaratively in `/etc/nixos/configuration.nix`:

```nix
{
    environment.systemPackagess = with pkgs; [
      ...
      termusic
    ];
}
```

#### Cargo

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
- [x] Database.

## Contributing and issues 🤝🏻

Contributions, bug reports, new features and questions are welcome! 😉
If you have any question or concern, or you want to suggest a new feature, or you want just want to improve termusic, feel free to open an issue or a PR.

Please follow [our contributing guidelines](CONTRIBUTING.md)


## Thanks
- [tui-realm](https://github.com/veeso/tui-realm) 
- [termscp](https://github.com/veeso/termscp)
- [netease-cloud-music-gtk](https://github.com/gmg137/netease-cloud-music-gtk)
- [alacritty-themes](https://github.com/rajasegar/alacritty-themes)

## License

GPLv3 for NetEase api code under `src/lyric/netease`.
MIT License for other code.
