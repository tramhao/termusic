# Terminal Music and Podcast Player written in Rust

[![Build status](https://github.com/tramhao/termusic/actions/workflows/build.yml/badge.svg)](https://github.com/tramhao/termusic/actions)
[![crates.io](https://img.shields.io/crates/v/termusic.svg)](https://crates.io/crates/termusic)
[![dependency status](https://deps.rs/repo/github/tramhao/termusic/status.svg)](https://deps.rs/repo/github/tramhao/termusic)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-blue)](https://releases.rs/docs/1.85.0/)

Listen to music and podcasts freely as both in freedom and free of charge!

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
Complicated copyright issues make things worse.
If my favorite song cannot be found on a website, I'll probably just not listen to them for years.

**Free of charge**: You can download from YouTube, NetEase, Migu and KuGou for free.
No need to register for monthly paid memberships.

As a contributor of [GOMU](https://github.com/issadarkthing/gomu), I met serious problems during development. The main problem is data race condition.
So I rewrote the player in rust, and hope to solve the problem.

## Supported Formats

Below are the audio formats supported by the various backends.

In the case that metadata is not supported, an attempt will still be made to play the file.

| Format (`feature`) | Symphonia (`rusty`)     | Mpv (`mpv`) | Gstreamer (`gst`) | Metadata |
| ------------------ | ----------------------- | ----------- | ----------------- | -------- |
| ADTS               | Yes                     | Yes         | Yes               | No       |
| AIFF               | Yes                     | Yes         | Yes               | Yes      |
| FLAC               | Yes                     | Yes         | Yes               | Yes      |
| M4a                | Yes                     | Yes         | Yes               | Yes      |
| MP3                | Yes                     | Yes         | Yes               | Yes      |
| Opus               | No                      | Yes         | Yes               | Yes      |
| Ogg Vorbis         | Yes                     | Yes         | Yes               | Yes      |
| Wav                | Yes                     | Yes         | Yes               | Yes      |
| WebM               | Yes(opus not supported) | Yes         | Yes               | No       |
| MKV                | Yes(depends on codec)   | Yes         | Yes               | No       |

Default backend: `rusty`

## Installation

### Requirements

#### MSRV
You will need to build with the stable rust toolchain. Minimal Supported Rust Version 1.85.0.

#### git

`git` will be required to build the package.

#### Backends

| Backend   | Requirements |
| :-------: | :----------- |
| Symphonia(rusty) | On Linux [`libasound2-dev`](https://launchpad.net/ubuntu/noble/+package/libasound2-dev) is required for building.<br/>When using `rusty-soundtouch` additionally `soundtouch` and `clang`(build only) are required. |
| GStreamer | [GStreamer](https://gstreamer.freedesktop.org) |
| MPV       | [MPV](https://mpv.io/) |

#### Protobuf

This is required to build and run termusic. For ubuntu: `protobuf-compiler`, For arch: `protobuf`.

#### Dbus

DBus is required for MPRIS control. For ubuntu: `libdbus-1-dev`, For arch: `dbus`.

#### Yt-dlp support

You can optionally install [yt-dlp](https://github.com/yt-dlp/yt-dlp/) and [FFmpeg](https://www.ffmpeg.org/download.html) to download from various providers.

#### Album cover support

To display covers in the terminal itself, feature `cover` can be enabled.
To only enable specific protocols for cover support, see [tui/Cargo.toml#features](./tui/Cargo.toml).

Feature `cover-ueberzug` will require some ueberzug implementation to be present at runtime.

### Pre-Compiled Packages

Do note that these will be compiled with the **symphonia** backend.

#### Arch Linux

Arch Linux users can install `termusic` from the [official repositories](https://archlinux.org/packages/extra/x86_64/termusic) using [pacman](https://wiki.archlinux.org/title/pacman).

```bash
pacman -S termusic
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
cargo install termusic termusic-server --locked
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

By default, termusic can display album covers in Kitty or iTerm2 (mac, not tested).
If you need album covers displayed on other terminals, you can enable the `sixel` protocol or use a ueberzug implementation(x11/xwayland only).

To build with all backends and all cover protocols:

```bash
make full
```

Finally, you can run it with:

```bash
~/.local/share/cargo/bin/termusic
```

You can copy it anywhere in your `$PATH`. The configuration file for the TUI is located in `~/.config/termusic/tui.toml`, and the configuration file for the server is located in `~/.config/termusic/server.toml` (or on macOS, `~/Library/Application Support/termusic/tui.toml`, `~/Library/Application Support/termusic/server.toml`, respectively). <!---The MacOS, i assume it has the same rules as linux, so its a good idea to check, as i lack and macOS machine.-->
However, as this is a minimalistic program, you don't need to edit the configuration file and almost everything can be set from the app.

## TODO

- [ ] Better interface to adjust timestamp of lyric.
- [ ] Rating and sync support.
- [x] Multiple root and easy switch.
- [x] Save playlists.
- [x] Listen to rss feeds/Podcasts. Need a new layout.

## Contributing and issues 🤝🏻

Contributions, bug reports, new features and questions are welcome! 😉
If you have any question or concern, or you want to suggest a new feature, or you want just want to improve termusic, feel free to open an issue or a PR.

Please follow [our contributing guidelines](CONTRIBUTING.md)

## Contributors

hasezoey

## Thanks

- [tui-realm](https://github.com/veeso/tui-realm)
- [termscp](https://github.com/veeso/termscp)
- [netease-cloud-music-gtk](https://github.com/gmg137/netease-cloud-music-gtk)
- [alacritty-themes](https://github.com/rajasegar/alacritty-themes)
- [shellcaster](https://github.com/jeff-hughes/shellcaster)
- [stream-download](https://github.com/aschey/stream-download-rs)

## License

MIT License for main part of code.
GPLv3 for Podcast code under `lib/src/podcast/mod.rs`. Comes from shellcaster.
