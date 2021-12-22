use lofty::PictureInformation;

use std::fs::File;
use std::io::Read;

#[test]
fn read_png() {
	// 640x628
	let mut f = File::open("tests/picture/assets/png_640x628.png").unwrap();

	let mut buf = Vec::new();
	f.read_to_end(&mut buf).unwrap();

	let information = PictureInformation::from_png(&*buf).unwrap();

	assert_eq!(information.width, 640);
	assert_eq!(information.height, 628);
	assert_eq!(information.color_depth, 32);

	// No PLTE chunk
	assert_eq!(information.num_colors, 0);
}

#[test]
fn read_png_plte() {
	// PNG image with a PLTE chunk (indexed color)
	let mut f = File::open("tests/picture/assets/png_640x628_plte.png").unwrap();

	let mut buf = Vec::new();
	f.read_to_end(&mut buf).unwrap();

	let information = PictureInformation::from_png(&*buf).unwrap();

	assert_eq!(information.width, 640);
	assert_eq!(information.height, 628);
	assert_eq!(information.color_depth, 8);

	// This field is actually filled since we
	// have a PLTE chunk
	assert_eq!(information.num_colors, 118);
}

#[test]
fn read_jpeg() {
	let mut f = File::open("tests/picture/assets/jpeg_640x628.jpg").unwrap();

	let mut buf = Vec::new();
	f.read_to_end(&mut buf).unwrap();

	let information = PictureInformation::from_jpeg(&*buf).unwrap();

	assert_eq!(information.width, 640);
	assert_eq!(information.height, 628);
	assert_eq!(information.color_depth, 24);

	// Always 0, not applicable for JPEG
	assert_eq!(information.num_colors, 0);
}
