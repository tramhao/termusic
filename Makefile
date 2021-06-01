prog := music_tui_rs

fmt:
	cargo fmt

default: fmt
	cargo build


release:
	cargo build --release

install: fmt release
	cp target/release/$(prog) ~/.local/share/cargo/bin/

