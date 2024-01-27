prog := termusic 
server := termusic-server 

default: fmt 

fmt:
	cargo fmt --all
	cargo check --all
	cargo clippy --all
	# cargo clippy -- -D warnings

run: 
	cargo run --all 

# default backend, default features
release:
	cargo build --release --all

# backends + cover

rusty:
	cargo build --features cover --release --all

mpv:
	# disable "rusty" backend default
	cargo build --no-default-features --features cover,mpv --release --all

gst:
	# disable "rusty" backend default
	cargo build --no-default-features --features cover,gst --release --all

all-backends:
	cargo build  --features cover,all-backends --release --all

# end backends + cover

full: all-backends post

minimal: release post

post:
	mkdir -p ~/.local/share/cargo/bin/
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/
	cp -f target/release/$(server) ~/.local/share/cargo/bin/

install: release post
