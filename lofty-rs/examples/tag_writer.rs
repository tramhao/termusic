use lofty::{Accessor, Probe, Tag};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "tag_writer", about = "A simple tag writer example")]
struct Opt {
	#[structopt(short, long)]
	title: Option<String>,

	#[structopt(short, long)]
	artist: Option<String>,

	#[structopt(short = "A", long)]
	album: Option<String>,

	#[structopt(short, long)]
	genre: Option<String>,

	#[structopt(short, long)]
	path: String,
}

fn main() {
	let opt = Opt::from_args();

	let mut tagged_file = Probe::open(opt.path.as_str())
		.expect("Error: Bad path provided!")
		.read(false)
		.expect("Error: Failed to read file!");

	let tag = match tagged_file.primary_tag_mut() {
		Some(primary_tag) => primary_tag,
		None => {
			if let Some(first_tag) = tagged_file.first_tag_mut() {
				first_tag
			} else {
				let tag_type = tagged_file.primary_tag_type();

				eprintln!(
					"WARN: No tags found, creating a new tag of type `{:?}`",
					tag_type
				);
				tagged_file.insert_tag(Tag::new(tag_type));

				tagged_file.primary_tag_mut().unwrap()
			}
		},
	};

	if let Opt {
		title: None,
		artist: None,
		album: None,
		genre: None,
		..
	} = opt
	{
		eprintln!("ERROR: No options provided!");
		std::process::exit(1);
	}

	if let Some(title) = opt.title {
		tag.set_title(title)
	}

	if let Some(artist) = opt.artist {
		tag.set_artist(artist)
	}

	if let Some(album) = opt.album {
		tag.set_album(album)
	}

	if let Some(genre) = opt.genre {
		tag.set_genre(genre)
	}

	tag.save_to_path(opt.path)
		.expect("ERROR: Failed to write the tag!");

	println!("INFO: Tag successfully updated!");
}
