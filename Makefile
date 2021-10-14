prog := termusic 

default: fmt 

fmt:
	cargo +nightly fmt
	cargo check
	cargo clippy

run: 
	cargo run

release:
	cargo build --release

m: 
	cargo build --features mpris --release

mpris: m post

post:
	mkdir -p ~/.local/share/cargo/bin/
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/

install: release post





