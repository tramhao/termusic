prog := termusic 

default: fmt 

fmt:
	cargo fmt --all
	cargo check
	cargo clippy 
	# cargo clippy -- -D warnings

run: 
	cargo run

release:
	cargo build --release

m: 
	cargo build --features mpris --release

c: 
	cargo build --features cover --release

f:
	cargo build --features mpris,cover,discord --release

mpv:
	cargo build --no-default-features --features mpris,cover,mpv --release

gst:
	cargo build --no-default-features --features mpris,cover,gst --release

mpris: m post

cover: c post

full: f post
# full: mpv post

minimal: release post

post:
	mkdir -p ~/.local/share/cargo/bin/
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/

install: release post





