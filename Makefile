prog := termusic 

default: fmt 

fmt:
	cargo +nightly fmt
	cargo +nightly check
	cargo +nightly clippy

run: 
	cargo run

release:
	cargo build --release

no_m: 
	cargo build --no-default-features --release

no_mpris: no_m post

post:
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/

install: release post





