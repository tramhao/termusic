## ChangeLog

### [v0.7.2]
- Released on: August, 2022.
- New: Add album and genre in tag editor.
- Fix: Running sync database in background, to speed up start of program.
- Fix: Import cpal to supress warning from alsa.

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
- Add album photo function. It'll show the jpg or png file under the same folder of the playing track, if the track doesn't have embeded photo.
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
- Fix: wrong mime-type for embeded album photo.

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
- New: color theme support. Shift+C to open color editor. You can change the whole theme, or edit the specific color. The themes are from alacritty-themes, and are localed in `~/.config/termusic/themes/` folder. If you open color editor, and found no themes, please copy them manually.


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
