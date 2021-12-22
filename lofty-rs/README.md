# Lofty
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Serial-ATA/lofty-rs/CI?style=for-the-badge&logo=github)](https://github.com/Serial-ATA/lofty-rs/actions/workflows/ci.yml)
[![Downloads](https://img.shields.io/crates/d/lofty?style=for-the-badge&logo=rust)](https://crates.io/crates/lofty)
[![Version](https://img.shields.io/crates/v/lofty?style=for-the-badge&logo=rust)](https://crates.io/crates/lofty)
[![Documentation](https://img.shields.io/badge/docs.rs-lofty-informational?style=for-the-badge&logo=read-the-docs)](https://docs.rs/lofty/)

Parse, convert, and write metadata to various audio formats.

## Supported Formats

| File Format | Extensions                                      | Read  | Write | Metadata Format(s)                             |
|-------------|-------------------------------------------------|-------|-------|------------------------------------------------|
| Ape         | `ape`                                           | **X** | **X** | `APEv2`, `APEv1`, `ID3v2` (Read only), `ID3v1` |
| AIFF        | `aiff`, `aif`                                   | **X** | **X** | `ID3v2`, `Text Chunks`                         |
| FLAC        | `flac`                                          | **X** | **X** | `Vorbis Comments`                              |
| MP3         | `mp3`                                           | **X** | **X** | `ID3v2`, `ID3v1`, `APEv2`, `APEv1`             |
| MP4         | `mp4`, `m4a`, `m4b`, `m4p`, `m4r`, `m4v`, `3gp` | **X** | **X** | `iTunes-style ilst`                            |
| Opus        | `opus`                                          | **X** | **X** | `Vorbis Comments`                              |
| Ogg Vorbis  | `ogg`                                           | **X** | **X** | `Vorbis Comments`                              |
| WAV         | `wav`, `wave`                                   | **X** | **X** | `ID3v2`, `RIFF INFO`                           |

## Examples

* [Tag reader](examples/tag_reader.rs)
* [Tag stripper](examples/tag_stripper.rs)
* [Tag writer](examples/tag_writer.rs)

To try them out, run:

```bash
cargo run --example tag_reader /path/to/file
cargo run --example tag_stripper /path/to/file
cargo run --example tag_writer <options> --path /path/to/file
```

## Documentation

Available [here](https://docs.rs/lofty)

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
