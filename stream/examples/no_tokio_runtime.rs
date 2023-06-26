use stream_download::StreamDownload;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::default().add_directive("stream_download=trace".parse().unwrap()),
        )
        .with_line_number(true)
        .with_file(true)
        .init();
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let reader = StreamDownload::new_http(
        "https://dl.espressif.com/dl/audio/ff-16b-2c-44100hz.flac"
            .parse()
            .unwrap(),
    )
    .unwrap();

    sink.append(rodio::Decoder::new(reader).unwrap());

    sink.sleep_until_end();
}
