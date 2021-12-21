prog := termusic 

default: fmt 

fmt:
	cargo fmt --all
	cargo check
	cargo clippy

run: 
	cargo run

release:
	cargo build --release

m: 
	cargo build --features mpris --release

c: 
	cargo build --features cover --release

f:
	cargo build --features mpris,cover,yt-dlp --release

mpris: m post

cover: c post

full: f post

post:
	mkdir -p ~/.local/share/cargo/bin/
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/
	mkdir -p ~/.config/termusic/themes
	cp -nr themes/* ~/.config/termusic/themes/

install: release post





