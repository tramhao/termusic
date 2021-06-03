prog := termusic 

default: fmt debug

fmt:
	cargo fmt

debug: 
	cargo build

release:
	cargo build --release


install: release
	cp target/release/$(prog) ~/.local/share/cargo/bin/

