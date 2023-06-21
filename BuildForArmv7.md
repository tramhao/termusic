## Install Docker
Please refer to https://docs.docker.com/desktop/install/linux-install/

## Install cross-rs
`cargo install cross --git https://github.com/cross-rs/cross`

## Build customized docker image to support external dependency
```
$ mkdir cross-armv7-docker
$ cd cross-armv7-docker
$ cat > DockerFile << EOF
# base pre-built cross image
FROM ghcr.io/cross-rs/armv7-unknown-linux-gnueabihf:edge

# add our foreign architecture and install our dependencies
RUN apt-get update && apt-get install -y --no-install-recommends apt-utils
RUN dpkg --add-architecture armhf
RUN apt-get update && apt-get -y install libasound2-dev:armhf

# add our linker search paths and link arguments
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS="-L /usr/lib/arm-linux-gnueabihf -C link-args=-Wl,-rpath-link,/usr/lib/arm-linux-gnueabihf $CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS"
EOF

$ docker build -t cross/armv7:latest .
```

## Prepare Cross.toml
```
$ cd termusic
$ cat > Cross.toml << EOF
[target.armv7-unknown-linux-gnueabihf]
image = "cross/armv7:latest"
EOF
```

## Build binary for armv7
> Need to apply pull request at first. Please see https://github.com/tramhao/termusic/pull/129

```
$ cd termusic
cross build --target armv7-unknown-linux-gnueabihf --release --verbose
```
> You can find binary from <termusic root>/target/armv7-unknown-linux-gnueabihf/release/
