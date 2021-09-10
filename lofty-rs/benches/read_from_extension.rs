use criterion::{criterion_group, criterion_main, Criterion};
use lofty::Tag;

macro_rules! test_read {
	($function:ident, $path:expr) => {
		fn $function() {
			Tag::new().read_from_path($path).unwrap();
		}
	};
}

test_read!(read_ape, "tests/assets/a.ape");
test_read!(read_flac, "tests/assets/a.flac");
test_read!(read_m4a, "tests/assets/a.m4a");
test_read!(read_mp3, "tests/assets/a.mp3");
test_read!(read_vorbis, "tests/assets/a.ogg");
test_read!(read_opus, "tests/assets/a.opus");
test_read!(read_riff, "tests/assets/a-id3.wav");

fn bench_ext(c: &mut Criterion) {
	let mut g = c.benchmark_group("From extension");
	g.bench_function("APE", |b| b.iter(read_ape));
	g.bench_function("FLAC", |b| b.iter(read_flac));
	g.bench_function("MP4", |b| b.iter(read_m4a));
	g.bench_function("MP3", |b| b.iter(read_mp3));
	g.bench_function("VORBIS", |b| b.iter(read_vorbis));
	g.bench_function("OPUS", |b| b.iter(read_opus));
	g.bench_function("RIFF", |b| b.iter(read_riff));
}

criterion_group!(benches, bench_ext);
criterion_main!(benches);
