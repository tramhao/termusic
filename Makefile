prog := termusic 

default: fmt 

fmt:
	cargo fmt
	cargo check
	cargo clippy
	# cargo +nightly clippy

run: 
	cargo run

release:
	cargo build --release

m: 
	cargo build --features mpris --release

mpris: m post

post:
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/

install: release post





