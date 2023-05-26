prog := termusic 
daemon := termusicd 

default: fmt 

fmt:
	cargo fmt --all
	cargo check --all
	cargo clippy --all
	# cargo clippy -- -D warnings

run: 
	cargo run --all 

release:
	cargo build --release --all


f:
	cargo build --features cover --release --all

mpv:
	cargo build --no-default-features --features cover,mpv --release --all

gst:
	cargo build --no-default-features --features cover,gst --release --all


full: f post
# full: mpv post
# full: gst post

minimal: release post

post:
	mkdir -p ~/.local/share/cargo/bin/
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/
	cp -f target/release/$(daemon) ~/.local/share/cargo/bin/

install: release post





