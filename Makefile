prog := termusic
server :=termusic-server
ifeq ($(OS),Windows_NT)
# Windows 
	prog := termusic.exe
	server := termusic-server.exe 
endif
default_cargo_home = $(HOME)/.local/share/cargo
# define CARGO_HOME if not defined
ifndef CARGO_HOME
	CARGO_HOME=$(default_cargo_home)
	install_to = $(CARGO_HOME)/bin
endif
# needs to be after CARGO_HOME, otherwise the default is not ever added
# install_to = $(CARGO_HOME)/bin

ifeq ($(OS),Windows_NT)
	install_to = $(USERPROFILE)\.cargo\bin
endif

default: fmt 

fmt:
	cargo fmt --all
	cargo check --all --features cover,all-backends
	cargo clippy --all --features cover,all-backends
	# cargo clippy -- -D warnings

run: 
	cargo run 

# default backend, default features
release:
	cargo build --release --all

# backends + cover

rusty:
	cargo build --features cover --release --all

mpv:
	# disable "rusty" backend default
	cargo build --no-default-features --features cover,mpv --release --all

gst:
	# disable "rusty" backend default
	cargo build --no-default-features --features cover,gst --release --all

all-backends:
	cargo build  --features cover,all-backends --release --all

test: 
	cargo test --features cover,all-backends --release --all

# end backends + cover

full: all-backends post

minimal: release post

post: 
	echo $(install_to)
	cp -f target/release/$(prog) "$(install_to)"
	cp -f target/release/$(server) "$(install_to)"

install: release post

win:
	cargo build --all

winrelease:
	cargo build --release --all

winpost:
	powershell -noprofile -command "Write-Host $(install_to)"
	cp -f target/release/$(prog) "$(install_to)"
	cp -f target/release/$(server) "$(install_to)"

wininstall: winrelease winpost
