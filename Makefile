prog := music_tui_rs

default:
	cargo build

release:
	cargo build --release

install: release
	cp target/release/$(prog) ~/.local/share/cargo/bin/

