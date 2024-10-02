// using lower mod to restrict clippy
#[allow(clippy::pedantic)]
mod protobuf {
    tonic::include_proto!("player");
}

pub use protobuf::*;

// implement transform function for easy use
impl From<protobuf::Duration> for std::time::Duration {
    fn from(value: protobuf::Duration) -> Self {
        std::time::Duration::new(value.secs, value.nanos)
    }
}

impl From<std::time::Duration> for protobuf::Duration {
    fn from(value: std::time::Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}
