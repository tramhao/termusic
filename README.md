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
| Opus               | No*1                    | Yes         | Yes               | Yes      |
| Ogg Vorbis         | Yes                     | Yes         | Yes               | Yes      |
| Wav                | Yes                     | Yes         | Yes               | Yes      |
| WebM               | Yes(opus not supported) | Yes         | Yes               | No       |
| MKV                | Yes(depends on codec)   | Yes         | Yes               | No       |

*1: `Opus` codec is supported in rusty backend if feature `rusty-libopus` is enabled.

Default backend: `rusty`

## Installation

### Requirements

#### MSRV

The minimal Rust version required to build this project is `1.85.0`.

#### Dependencies

| Package name (ubuntu) | Package name (arch) | Required | Build-time-only |       Feature       |                      Description                      |
| :-------------------: | :-----------------: | :------: | :-------------: | :-----------------: | :---------------------------------------------------: |
|         `git`         |        `git`        |    X     |        X        |                     |                    version control                    |
|        `clang`        |       `clang`       |    X     |        X        |                     |       General Build tools (and sqlite compile)        |
|  `protobuf-compiler`  |     `protobuf`      |    X     |        X        |                     | communication protocol between server and client(tui) |
|    `libdbus-1-dev`    |       `dbus`        |    X     |     unknown     |                     |                  MPRIS media control                  |
|   `libasound2-dev`    |        None         |    X     |     unknown     |                     |                     ALSA headers                      |
|       `yt-dlp`        |      `yt-dlp`       |          |                 |                     |                 Download some tracks                  |
|         `mpv`         |        `mpv`        |          |                 |        `mpv`        |                      MPV Backend                      |
|      `gstreamer`      |     `gstreamer`     |          |                 |        `gst`        |                   Gstreamer Backend                   |
|       `libopus`       |      `libopus`      |    X     |                 |   `rusty-libopus`   |          Opus codec support in rusty backend          |
|      `libsixel`       |     `libsixel`      |    X     |                 | `cover-viuer-sixel` |                Sixel protocol support                 |
|     `ueberzugpp`      |    `ueberzugpp`     |          |                 |  `cover-ueberzug`   |               Ueberzug protocol support               |

#### Backends

|     Backend      | Requirements                                                                                                      |
| :--------------: | :---------------------------------------------------------------------------------------------------------------- |
| Symphonia(rusty) | On Linux [`libasound2-dev`](https://launchpad.net/ubuntu/noble/+package/libasound2-dev) is required for building. |
|    GStreamer     | [GStreamer](https://gstreamer.freedesktop.org)                                                                    |
|       MPV        | [MPV](https://mpv.io/)                                                                                            |

There are extra features for some backends:
Note that they are not enabled by default and potentially increase non-rust dependencies.

|      Feature       | Backend |                            Description                            | Extra Dependencies |
| :----------------: | :-----: | :---------------------------------------------------------------: | :----------------: |
|    `rusty-simd`    | `rusty` |                     Enable SIMD instructions                      |                    |
| `rusty-soundtouch` | `rusty` | Enable `soundtouch` compilation and use as default speed-modifier |                    |
|  `rusty-libopus`   | `rusty` |         Enable `libopus` support to support `opus` files          |     `libopus`      |

#### Album cover support

To display covers in the terminal itself, feature `cover` can be enabled.
To only enable specific protocols for cover support, see [tui/Cargo.toml#features](./tui/Cargo.toml).

Feature `cover-ueberzug` will require some ueberzug implementation to be present at runtime.

### Official Install Sources

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

By default, termusic can display album covers in Kitty or iTerm2.
If you need album covers displayed on other terminals, you can enable the `sixel` protocol or use a ueberzug implementation(x11/xwayland only).

To build all backends and all cover protocols and install them in your home:

```bash
make full
```

Finally, you can run it with:

```bash
~/.local/share/cargo/bin/termusic
```

To build with all backends and all cover protocols without copying binaries elsewhere:

```bash
make all-backends
```

### Unofficial Install Sources

The following are ways to install termusic, but may differ in configuration and support.

They are not maintained by the termusic project itself.

#### Arch Linux

Arch Linux users can install `termusic` from the [official repositories](https://archlinux.org/packages/extra/x86_64/termusic) using [pacman](https://wiki.archlinux.org/title/pacman).

```bash
pacman -S termusic
```

#### Arch Linux GIT (AUR)

Arch Linux users can install [`termusic-git` from the AUR](https://aur.archlinux.org/packages/termusic-git) using [pamac](https://aur.archlinux.org/packages/pamac-cli).

```bash
pamac install termusic-git
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
