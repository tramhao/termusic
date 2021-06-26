prog := termusic 

default: fmt 

fmt:
	cargo +nightly fmt
	cargo +nightly check

run: 
	cargo run

release:
	cargo build --release


install: release
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/

