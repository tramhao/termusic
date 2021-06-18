prog := termusic 

default: fmt 

fmt:
	cargo fmt
	cargo check

run: 
	cargo run

release:
	cargo build --release


install: release
	cp -f target/release/$(prog) ~/.local/share/cargo/bin/

