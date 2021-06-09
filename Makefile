prog := termusic 

default: fmt run 

fmt:
	cargo fmt

run: 
	cargo run

release:
	cargo build --release


install: release
	cp target/release/$(prog) ~/.local/share/cargo/bin/

