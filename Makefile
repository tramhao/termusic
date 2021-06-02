prog := music_tui_rs

default: fmt debug

fmt:
	cargo fmt

debug: 
	cargo build

release:
	cargo build --release


install: fmt release
	cp target/release/$(prog) ~/.local/share/cargo/bin/

