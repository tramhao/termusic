use lofty::Probe;

use criterion::{criterion_group, criterion_main, Criterion};

use std::io::Cursor;

macro_rules! test_read_path {
	($function:ident, $path:expr) => {
		fn $function() {
			Probe::open($path).unwrap().read(true).unwrap();
		}
	};
}

test_read_path!(read_aiff_path, "tests/files/assets/a.aiff");
test_read_path!(read_ape_path, "tests/files/assets/a.ape");
test_read_path!(read_flac_path, "tests/files/assets/a.flac");
test_read_path!(read_m4a_path, "tests/files/assets/a.m4a");
test_read_path!(read_mp3_path, "tests/files/assets/a.mp3");
test_read_path!(read_vorbis_path, "tests/files/assets/a.ogg");
test_read_path!(read_opus_path, "tests/files/assets/a.opus");
test_read_path!(read_riff_path, "tests/files/assets/a.wav");

fn path_infer_read(c: &mut Criterion) {
	let mut g = c.benchmark_group("File reading (Inferred from Path)");
	g.bench_function("AIFF", |b| b.iter(read_aiff_path));
	g.bench_function("APE", |b| b.iter(read_ape_path));
	g.bench_function("FLAC", |b| b.iter(read_flac_path));
	g.bench_function("MP4", |b| b.iter(read_m4a_path));
	g.bench_function("MP3", |b| b.iter(read_mp3_path));
	g.bench_function("VORBIS", |b| b.iter(read_vorbis_path));
	g.bench_function("OPUS", |b| b.iter(read_opus_path));
	g.bench_function("RIFF", |b| b.iter(read_riff_path));
}

macro_rules! test_read_file {
	($function:ident, $name:ident, $path:expr) => {
		const $name: &[u8] = include_bytes!($path);

		fn $function() {
			Probe::new(Cursor::new($name))
				.guess_file_type()
				.unwrap()
				.read(true)
				.unwrap();
		}
	};
}

test_read_file!(read_aiff_file, AIFF, "../tests/files/assets/a.aiff");
test_read_file!(read_ape_file, APE, "../tests/files/assets/a.ape");
test_read_file!(read_flac_file, FLAC, "../tests/files/assets/a.flac");
test_read_file!(read_m4a_file, MP4, "../tests/files/assets/a.m4a");
test_read_file!(read_mp3_file, MP3, "../tests/files/assets/a.mp3");
test_read_file!(read_vorbis_file, VORBIS, "../tests/files/assets/a.ogg");
test_read_file!(read_opus_file, OPUS, "../tests/files/assets/a.opus");
test_read_file!(read_riff_file, RIFF, "../tests/files/assets/a.wav");

fn content_infer_read(c: &mut Criterion) {
	let mut g = c.benchmark_group("File reading (Inferred from Content)");
	g.bench_function("AIFF", |b| b.iter(read_aiff_file));
	g.bench_function("APE", |b| b.iter(read_ape_file));
	g.bench_function("FLAC", |b| b.iter(read_flac_file));
	g.bench_function("MP4", |b| b.iter(read_m4a_file));
	g.bench_function("MP3", |b| b.iter(read_mp3_file));
	g.bench_function("VORBIS", |b| b.iter(read_vorbis_file));
	g.bench_function("OPUS", |b| b.iter(read_opus_file));
	g.bench_function("RIFF", |b| b.iter(read_riff_file));
}

criterion_group!(benches, path_infer_read, content_infer_read);
criterion_main!(benches);
