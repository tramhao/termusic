## ChangeLog

### next
- Unreleased
- Change: enable `log-to-file` by default.
- Change: updated MSRV to 1.75.
- Change: on backend rusty, require `clang` and `soundtouch` as system-dependencies.
- Change: update some dependencies.
- Change: use more specific versions instead of just the major version (lib.rs suggestion).
- Change: remove unused dependencies from all packages.
- Change: add config options to customize Important Popup colors (`style_color_symbol.important_popup_*`).
- Change: add config options to customize fallback colors (`style_color_symbol.fallback_*`).
- Change(tui): move version display to be the last instead of first element in the bottom-bar.
- Change(tui): rename previous feature `cover` to `cover-ueberzug`
- Feat: Add `TM_LOGTOFILE` and `TMS_LOGTOFILE` to control `--log-to-file` for tui and server respectively.
- Feat(tui): allow Sixel to be used for covers.
- Feat(tui): allow all cover providers to not be compiled in.
- Fix: update the build scripts to check that the repository is actually the `termusic` repository before using the git version.
- Fix: allow backends to be compiled in for `termusic-playback` but not in `termusic-server`.
- Fix: on backend mpv, clear media-title on EndOfFile.
- Fix: consistent media-title(/radio-title) handling across all backends.
- Fix: use async-reqwest in all places (instead of `reqwest::blocking`, fixes debug builds in some areas).
- Fix: colors from the config & yaml theme are now parsed at time of load, instead of on-demand.
- Fix(tui): ensure the Quit-Popup always has top-most focus.
- Fix(tui): also close a Error-Popup with global-quit(default `q`) key.
- Fix(tui): use a common code-path for all `No/Yes`-Popups.
- Fix(tui): change Popup titles to have consistent spacing on both sides.
- Fix(server): on gstreamer backend, try to correctly apply the speed on track start.
- Fix(server): on rusty backend, reset the decoder after a seek when codec is `mp3` (fixes audible noise while seeking for this codec).
- Fix(server): on rusty backend, more accureately seek to requested position.
- Fix(server): update Media-Controls volume on volume change instead of on tick.
- Fix(server): update Media-Controls with *new* progress instead of last-tick progress.
- Fix(server): dont start Media-Controls if not requested (like disabled via config).
- Fix(server): allow podcast feed import/export, and not just say so in the help.
- Refactor: a lot less cloning and conversions where not necessary.
- Refactor(server): on backend rusty, clean-up decoding & seeking.

### [v0.9.0]
- Released on: March 24, 2024.
- Big thanks to the contribution of hasezoey. A lot of improvements and refactors in this release. Especially now you can change backends without recompile.
- Change: updated MSRV to 1.74.
- Change: better Error messages if the server binary cannot be started.
- Change: small optimizations.
- Change: change almost all `eprintln` to be `log::error`.
- Change: change almost all `println` to be appropriate log levels.
- Feat: change logger to be `flexi_logger`, including logging to file.
- Feat: add cli arguments to the server binary.
- Feat: add a lot more metadata to media controls, like cover art, duration, seek, volume(RW), quit.
- Feat: support `mkv` & `webm` in backend `rusty` (no metadata, support depends on codec).
- Feat: in backend `rusty`, buffer files in 4Mb chunks.
- Feat: better version via `--version`.
- Feat: allow specifiying which interface (ip) to run on.
- Feat(server): allow compiling multiple backends via features and select at binary start (via `--backend` or `TMS_BACKEND`).
- Feat(server): for backend `mpv`, switch to use `libmpv-sirno` and use mpv API 2.0.
- Feat(server): for backend `mpv` and `gstreamer`, speed change without changing pitch is great for podcasts.
- Feat(tui): add a "currently playing" symbol to active track in playlist.
- Feat(tui): add search function for Podcast Episodes.
- Feat(tui): allow confirming quit-confirm choices with `Y` or `N`.
- Feat(tui): allow confirming config save confirm choices with `Y` or `N`.
- Fix: try to find the server binary adjacent to the TUI binary.
- Fix: change many panics to be results instead.
- Fix: dont panic if "music_dir" value is empty when entering config editor, fixes #161.
- Fix: log *something* if a file is not going to be added to the playlist.
- Fix: in backend `rusty`, skip all tracks (and packets) that are not the selected track in backend in decode.
- Fix: in backend `rusty`, correctly select a audio track (instead of symphonia's default which might be something else).
- Fix: in backend `rusty`, when using radio, always use overwrite the last radio title instead of appending.
- Fix: in backend `rusty`, when using radio, parse until `';` instead of just `'`, now things like `Don't` actually work correctly.
- Fix: in backend `rusty`, when using radio, dont infinitely save the stream.
- Fix: in backend `rusty`, when using radio, now only use 1 stream to get audio *and* metadata (instead of 2).
- Fix: in backend `gst`, fix gapless track change not being tracked correctly, fixes #192.
- Fix(tui): add panic hook to reset screen before printing backtrace.
- Fix(tui): dont extra clear screen on quit.
- Fix(tui): reset screen if a Error(Result) happens and exit properly.
- Fix(tui): wait until tonic is connected instead of static sleeps.
- Fix(tui): only display `ueberzug` "Not found" errors once.
- Fix(tui): blanket disable `ueberzug` for windows.
- Fix(tui): in `Database -> Tracks` view, display track title instead of filename.
- Fix(server): log port used.
- Fix(server): log on quit.
- Fix(server): properly exit on player thread crash (instead of being pseudo-zombie).
- Fix(server): potentially fix media display in windows.
- Fix(server): in backend `rusty`, fix radio not starting if `gapless` is enabled and the track changes to be radio (from something else).
- Fix(server): in backend `rusty`, fix radio streams not being stopped once they have been skipped.
- Fix(build): install to `$CARGO_HOME/bin` instead of always into a static path.
- a **bunch** of internal refactors.

### [v0.8.0]
- Released on: March 23, 2024
- Yanked as it had been a broken release, see [v0.9.0](#v090) instead.

### [v0.7.11]
- Released on: July 11, 2023.
- For packagers: This version add a binary termusic-server and remove some features flags, please kindly check Makefile for changes and update packaging script accordingly. Thanks so much.
- New: split the function of app to termusic-server and termusic. termusic-server can be run separately with `RUST_LOG=trace termusic-server` to debug.
- New: remove feature flag mpris and use a configuration option use_mpris(default is true) to control the function.
- New: remove feature flag discord and use a configuration option use_discord(default is true) to control the function.
- Change: loop mode change to single/playlist/random. Remove the option to add tracks in the front of playlist.
- New: termusic-server and termusic communicate through rpc, and the default port is `50101`. Can be configured as other values.
- New: can load .m3u file with live audio stream.

### [v0.7.10]
- Released on: April 09, 2023.
- Fix: don't panic if XDG_MUSIC_DIR is not set.

### [v0.7.9]
- Released on: February 16, 2023.
- Fix: don't panic if XDG_MUSIC_DIR is not set.
- Fix: bump lofty to v0.11 and solve build error caused by lofty v0.10 not found.
- Fix: don't create music dir.

### [v0.7.8]
- Released on: January 14, 2023.
- New: Podcast player. Import / Export opml file. Add feed. Sync feed. Download episode. Mark as played. For details, please check out the help dialogue.
- New: Seek step can be adjusted. Default is auto, means for audio longer than 10 mins, seek step is 30 seconds. Otherwise it's 5 seconds.
- New: Handle position, size and hide album photo with several new hotkeys.

### [v0.7.7]
- Released on: December 26, 2022.
- New: Save playlist by Ctrl+s.
- New: Change the random select album function to selecting an album with no less than 5 tracks. This quantity can be configured in config editor.
- New: Change configuration file management to figment. User defined values will not be overwritten during upgrades.
- Fix: Open root when start app. Change command line parser from lexopt to clap, to get a colorful help.

### [v0.7.6]
- Released on: December 20, 2022.
- New: Remember playing position. It's useful for long tracks especially audio books. It can be configured from
       config editor or config file. There are 3 values for this config. Yes means always remember. No means never.
       Default value is auto. This means for tracks longer than 10 minutes, it'll remember playing position.

### [v0.7.5]
- Released on: October 26, 2022.
- Fix: Change album cover tmp file to ~/.cache/termusic/termusic_cover.jpg and fall back to $TMP/termusic/termusic_cover.jpg .

### [v0.7.4]
- Released on: October 12, 2022.
- Fix: Build error under MacOS and probably Windows as well.
- Fix: MSRV changed to rust v1.61.0 because a dependent package quick-xml upgraded and refuse to build below this version.

### [v0.7.3]
- Released on: August 18, 2022.
- Fix: Windows compile warnings.
- Fix: Clippy warning for rust v1.63.0.
- Fix: Compile error for lofty 0.8.
- Fix: tag editor delete error.

### [v0.7.2]
- Released on: August 06, 2022.
- New: Add album and genre in tag editor.
- Fix: Running sync database in background, to speed up start of program.
- Fix: Import cpal to suppress warning from alsa.

### [v0.7.1]
- Released on: July 28th, 2022.
- Fix: `invalid main_data offset` error from symphonia 0.5.1.
- Fix: invalid color for key config.
- Fix: losing focus when popup mounted.
- Fix: improve database sync speed.
- Fix: don't output alsa buffer underrun warning.

### [v0.7.0]
- Released on: July 24th, 2022.
- New: support configure function keys like F1 or f1 in config editor.
- New: add command line option `-c` to disable cover art, and `-d` to disable discord rpc.
- New: add command line option `-m` to set max_depth of folder. Default to 4.
- New: configure multiple root directory separated by `;` in config editor, and `o` hotkey to switch among them.
- New: `a` hotkey to add root, `A` to remove root.
- Fix: improve sync database to speed up loading.
- Fix: improve discord rpc to speed up loading.
- Fix: avoid none error when searching youtube, by fixing invidious error return when pressing next page.

### [v0.6.19]
- Released on: July 15th, 2022.
- New: replace color editor and key editor with new config editor.
- New: duplicate keys will not be saved.

### [v0.6.18]
- Released on: July 8th, 2022.
- New: Add lqueue and tqueue function similar to cmus.
- New: include theme files in binary because I saw they are not included in the aur package.
- New: Fetch invidious instance from website, so that they'll not expire and search youtube will always works.
- Fix: When playing mp3 encoded by iTunes under gapless mode, symphonia backend will panic.

### [v0.6.17]
- Released on: July 6th, 2022.
- New: Search for database. Triggered by `/` key when focusing database.
- New: Gapless playback for symphonia/mpv/gstreamer backend. Toggle by `Ctrl+g` and enabled by default.
- Fix: Youtube download mirrors are all broken. Replace them with new mirrors.
- Fix: After download from youtube, the prompt message will not disappear if error happens.

### [v0.6.16]
- Released on: May 21, 2022.
- New: support loading of m3u,m3u8,xspf,pls,asx playlists. Only local url supported.
- New: sqlite3 integration. Filter database by different criteria. Triggered by `2` key.

### [v0.6.15]
- Released on: May 9th, 2022.
- Fix: ignore hidden folder and files in music library.
- Fix: n key to stop playing when playlist is empty.
- Add: ctrl+j and ctrl+k to move playlist item down and up.
- Fix: ogg file duration is 0 with symphonia backend.
- Fix: seeking during pause with symphonia backend.

### [v0.6.14]
- Released on: April 29th, 2022.
- New: adjust playback speed by key 'ctrl + f' and 'ctrl + b'.
- New: discord rpc support. Can display the info of current playing song in your discord profile. Under feature gate `discord`. application id: 968407067889131520.
- Fix: cannot play when volume is 0. issue #63.

### [v0.6.13]
- Released on: April 19th, 2022.
- Max depth level of library changed from 3 to 4.
- Library behavior: left key will go to upper dir if a file is selected.

### [v0.6.12]
- Released on: March 31st, 2022.
- Add album photo function. It'll show the jpg or png file under the same folder of the playing track, if the track doesn't have embedded photo.
- Fix pause bug.
- Fix error embedding lrc after downloading from youtube.
- Filter unsupported file extension when adding to playlist(based on backend).

### [v0.6.11]
- Released on: March 8th, 2022.
- Fix ueberzug vertical position.
- Fix gstreamer compilation error with gstreamer version 0.18.

### [v0.6.10]
- Released on: Feb 10th, 2022.
- Make yt-dlp as default download program for youtube thus remove feature yt-dlp.
- Fix issue #39, repeating one song occasionally hangs.

### [v0.6.9]
- Released on: Jan 28th, 2022.
- Fix: panic when progress is bigger than 1.0.

### [v0.6.8]
- Released on: Jan 28th, 2022.
- Fix: progress display is wrong for symphonia backend(default). It should be 100 times bigger.

### [v0.6.7]
- Released on: Jan 24th, 2022.
- New: rust decoding backend! Previously supported backend changed to feature gate `gst` and `mpv`.
- Fix: issue #37. Add a new configuration option: playlist_display_symbol. Default is true.
- Remove dependency: humantime. Format the display of duration by self.
- Fix: issue #38. Small dialogues are cut off when window is too small.

### [v0.6.6]
- Released on: Jan 17th, 2022.
- New: add all key configuration for global, library and playlist(huge work).
- Minor Fix: Don't close search dialogue after add to playlist.
- New: new player backend mpv. If you prefer mpv, you can build with feature gate mpv. My testing result: gstreamer doesn't work for ape file, mpv works for everything but flac may seem buggy.

### [v0.6.5]
- Released on: Jan 3rd, 2022 .
- New: key configuration. To configure a value, please note the modifier bits value: `Shift=1`,`Ctrl=2`,`Alt=4`. You can combine them for example 6 is `Ctrl+Alt`. and 7 is `Ctrl+Alt+Shift`. Please note, whenever shift is configured, the args for char letter should be capital at the same time, for example `Q`.
- New: option to disable confirmation message box for quitting.
- New: aiff metadata supported by `lofty-rs`.
- New: shift_tab works in tag editor and color editor to switch focus.
- Fix: configuration for album photo position and size. Please note, default align for photo is BottomRight, means the x and y specifies bottom right corner of the photo. Supported align: BottomRight,BottomLeft,TopRight,TopLeft. Also, width should be between 1-100 because it's a relative number compared to terminal size. We don't specify height and it's calculated from width and the photo ratio is kept. Meanwhile, when x,y lead to display outside of terminal, app will correct it and try to draw on the terminal.
- Fix: wrong mime-type for embedded album photo.

### [v0.6.4]
- Released on: Dec 24, 2021.
- New feature: using [yt-dlp](https://github.com/yt-dlp/yt-dlp/) for downloading because youtube-dl is slower caused by throttle problem. For details please check [this reddit thread](https://www.reddit.com/r/youtubedl/comments/qfbyal/read_slow_youtube_downloads/). To use it, it's under feature gate yt-dlp. `make full` will enable all features including this one.
- New: opus format support. Metadata is supported by `lofty-rs`.
- New: configuration for album photo size and position.
- Fix: youtube search next page doesn't work.
- Fix: color editor playlist highlight symbol doesn't work.
- Fix: focus issue after exit tag editor.
- Fix: focus issue after download.
- Fix: command line open music dir not working.

### [v0.6.3]
- Released on: Dec 19, 2021.
- New: color theme support. Shift+C to open color editor. You can change the whole theme, or edit the specific color. The themes are from alacritty-themes, and are located in `~/.config/termusic/themes/` folder. If you open color editor, and found no themes, please copy them manually.

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
