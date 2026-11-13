# Termusic Playback

This library handles the music playback by interfacing with various backends.

Currently supported backends:
- `rusty`: the default, rust native decoder & playback. Uses symphonia for decode and cpal for output.
- `mpv`: handle interfacing with libmpv for decoding & playback.
- `gst`: handle interfacing with gstreamer for decoding & playback.

This crate is not intended for usage outside of termusic itself.
Due to that, changes are not considered breaking here.
