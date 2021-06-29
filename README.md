# Terminal Music Player written in Rust

As a contributor of GOMU(https://github.com/issadarkthing/gomu), I met serious problems during development. The main problem is data race condition. So I basically rewrite the player in rust, and hope to solve the problem.

Currently the project is still young, but working.
![name](https://github.com/tramhao/termusic/blob/master/screenshots/main.png?raw=true)
![tageditor](https://github.com/tramhao/termusic/blob/master/screenshots/tageditor.png?raw=true)

## Requirement:
Need mpv installed.

## Installation:
```
git clone https://github.com/tramhao/termusic.git
cd termusic
make
make install
~/.local/share/cargo/bin/termusic
```

## Implemented features:
- [x] Music library below ~/Music
- [x] Pause/Skip
- [x] Seek forward/backward
- [x] USLT lyric
- [x] Album Photo display(only for kitty terminal)
- [x] Youtube-dl integration
- [x] lyric and tag download
- [x] yank and paste in playlist
- [x] Lyric offset adjustment 
- [ ] configuration
- [ ] more player backend

## License
MIT License
