use lofty::{Accessor, Probe};

fn main() {
	let path = std::env::args().nth(1).expect("Error: No path specified!");

	let tagged_file = Probe::open(path)
		.expect("Error: Bad path provided!")
		.read(true)
		.expect("Error: Failed to read file!");

	let tag = match tagged_file.primary_tag() {
		Some(primary_tag) => primary_tag,
		None => tagged_file.first_tag().expect("Error: No tags found!"),
	};

	println!("--- Tag Information ---");
	println!("Title: {}", tag.title().unwrap_or("None"));
	println!("Artist: {}", tag.artist().unwrap_or("None"));
	println!("Album: {}", tag.album().unwrap_or("None"));
	println!("Album Artist: {}", tag.album_artist().unwrap_or("None"));
	println!("Genre: {}", tag.genre().unwrap_or("None"));

	let properties = tagged_file.properties();

	let duration = properties.duration();
	let seconds = duration.as_secs() % 60;

	let duration_display = format!("{:02}:{:02}", (duration.as_secs() - seconds) / 60, seconds);

	println!("--- Audio Properties ---");
	println!(
		"Bitrate (Audio): {}",
		properties.audio_bitrate().unwrap_or(0)
	);
	println!(
		"Bitrate (Overall): {}",
		properties.overall_bitrate().unwrap_or(0)
	);
	println!("Sample Rate: {}", properties.sample_rate().unwrap_or(0));
	println!("Channels: {}", properties.channels().unwrap_or(0));
	println!("Duration: {}", duration_display);
}
