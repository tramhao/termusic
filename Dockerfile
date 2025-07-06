FROM fedora:41 AS builder
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app
COPY . .
RUN dnf install -y gcc openssl-devel glib2-devel alsa-lib-devel gstreamer1-devel protobuf-devel && \
	cargo build --no-default-features --features gst --release --workspace

FROM scratch AS final
COPY --from=builder /app/target/release/termusic /app/target/release/termusic-server /
