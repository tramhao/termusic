use lofty::Probe;
use std::io::Write;

fn main() {
	let path = std::env::args().nth(1).expect("Error: No path specified!");

	let tagged_file = Probe::open(path.as_str())
		.expect("Error: Bad path provided!")
		.read(false)
		.expect("Error: Failed to read file!");

	let tags = tagged_file.tags();

	if tags.is_empty() {
		eprintln!("No tags found, exiting.");
		std::process::exit(0);
	}

	let mut available_tag_types = Vec::new();

	println!("Available tags:");

	for (num, tag) in tags.iter().enumerate() {
		let tag_type = tag.tag_type();

		println!("{}: {:?}", num, tag_type);
		available_tag_types.push(*tag_type);
	}

	let mut to_remove = None;
	let mut input = String::new();

	while to_remove.is_none() {
		print!("\nNumber to remove: ");
		std::io::stdout().flush().unwrap();

		if std::io::stdin().read_line(&mut input).is_ok() {
			if let Ok(num) = str::parse::<usize>(&*input.trim()) {
				if num < available_tag_types.len() {
					to_remove = Some(num);
					println!();
					break;
				}
			}
		}

		input.clear();
		eprintln!("ERROR: Unexpected input")
	}

	let tag_remove = available_tag_types[to_remove.unwrap()];

	if tag_remove.remove_from_path(path) {
		println!("INFO: Removed tag: `{:?}`", tag_remove);
	} else {
		eprintln!("ERROR: Failed to remove the tag")
	}
}
