prog := termusic 
server := termusic-server 
default_cargo_home := ~/.local/share/cargo

# define CARGO_HOME if not defined
ifndef CARGO_HOME
	CARGO_HOME=$(default_cargo_home)
endif

# needs to be after CARGO_HOME, otherwise the default is not ever added
install_to := $(CARGO_HOME)/bin

default: fmt 

fmt:
	cargo fmt --all
	cargo check --all
	cargo clippy --all
	# cargo clippy -- -D warnings

run: 
	cargo run --all 

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

# end backends + cover

full: all-backends post

minimal: release post

post: $(install_to)
	echo $(install_to)
	cp -f target/release/$(prog) "$(install_to)"
	cp -f target/release/$(server) "$(install_to)"

install: release post
