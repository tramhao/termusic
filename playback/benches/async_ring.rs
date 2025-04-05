use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use symphonia::core::audio::Channels;
use termusicplayback::__bench::async_ring::AsyncRingSource;
use tokio::runtime::Builder;

fn criterion_benchmark(c: &mut Criterion) {
    let runtime = Builder::new_current_thread().build().unwrap();

    let spec = symphonia::core::audio::SignalSpec {
        rate: 48000,
        channels: Channels::FRONT_LEFT | Channels::FRONT_RIGHT,
    };
    let total_duration = Some(Duration::from_secs(0));
    let current_frame_len = 0;

    let (mut prod, mut cons) = AsyncRingSource::new(
        spec,
        total_duration,
        current_frame_len,
        0,
        runtime.handle().clone(),
    );

    // spawn a infinitely producing producer
    std::thread::spawn(move || {
        runtime.block_on(async {
            loop {
                let data = [1u8; 256];
                if prod.write_data(&data).await.is_err() {
                    break;
                }
            }
        });
    });

    let mut group = c.benchmark_group("async-ring-read");
    // take many samples as we produce much data, this should cover reading multiple full data messages in length
    group.sample_size(2000);
    // because it is a iterator that returns 1 value, each iteration only operates on 1 value
    group.throughput(Throughput::Elements(1));

    // not using "black_box" as that is only necessary for inputs to a function, the routine itself is already wrapped in a black-box
    group.bench_function("read", |b| b.iter(|| cons.next()));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
