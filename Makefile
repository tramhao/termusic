prog := termusic 

default: fmt 

fmt:
	cargo +nightly fmt
	cargo +nightly check
	cargo +nightly clippy

run: 
	cargo run

release:
	cargo build --release --frozen

m: 
	cargo build --features mpris --release --frozen

mpris: m post

post:
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/

install: release post





