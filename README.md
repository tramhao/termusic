# Terminal Music Player written in Rust

As a contributor of GOMU(https://github.com/issadarkthing/gomu), I met serious problems during development. The main problem is data race condition. So I basically rewrite the player in rust, and hope to solve the problem.

Currently the project is still young, but working.

## Requirement:
Need mpv installed.

## Installation:
```
git clone https://github.com/tramhao/termusic.git
make
make install
~/.local/share/cargo/bin/termusic
```

## Implemented features:
- [x] Music library below ~/Music
- [x] Pause/Skip
- [x] Seek forward/backward
- [x] USLT lyric
- [ ] Album Photo display
- [ ] Youtube-dl integration
- [ ] lyric and tag download
- [ ] configuration
- [ ] more player backend

