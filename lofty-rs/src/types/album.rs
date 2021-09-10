use super::picture::Picture;

/// A struct for representing an album for convenience.
#[derive(Debug, Clone)]
pub struct Album<'a> {
	/// The title of the album
	pub title: Option<&'a str>,
	/// A `Vec` of the album artists
	pub artist: Option<&'a str>,
	/// The album's covers (Front, Back)
	pub covers: (Option<Picture>, Option<Picture>),
}

impl<'a> Default for Album<'a> {
	fn default() -> Self {
		Self {
			title: None,
			artist: None,
			covers: (None, None),
		}
	}
}

impl<'a> Album<'a> {
	/// Create a new `Album`
	pub fn new(
		title: Option<&'a str>,
		artist: Option<&'a str>,
		covers: (Option<Picture>, Option<Picture>),
	) -> Self {
		Self {
			title,
			artist,
			covers,
		}
	}
	/// Sets the album title
	pub fn set_title(&mut self, title: &'a str) {
		self.title = Some(title)
	}
	/// Clears the `title` field
	pub fn remove_title(&mut self) {
		self.title = None
	}
	/// Set the album artists
	pub fn set_artist(&mut self, artist_str: &'a str) {
		self.artist = Some(artist_str);
	}
	/// Clears the `artists` field
	pub fn remove_artist(&mut self) {
		self.artist = None
	}
	/// Sets the album's front cover
	pub fn set_front_cover(&mut self, cover: Picture) {
		self.covers.0 = Some(cover)
	}
	/// Sets the album's back cover
	pub fn set_back_cover(&mut self, cover: Picture) {
		self.covers.1 = Some(cover)
	}
	/// Clears the `covers` field
	pub fn remove_covers(&mut self) {
		self.covers = (None, None)
	}
}
