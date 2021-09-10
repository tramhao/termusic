use crate::Album;

/// Used to convert between tags
#[derive(Default, Debug)]
pub struct AnyTag<'a> {
	/// The track title
	pub title: Option<&'a str>,
	/// The track artists
	pub artist: Option<&'a str>,
	/// The track [`Album`]
	pub album: Album<'a>,
	/// Collection of user comments
	pub comments: Option<Vec<&'a str>>,
	/// The track year
	pub year: Option<i32>,
	/// The track date
	pub date: Option<String>,
	/// The track number
	pub track_number: Option<u32>,
	/// The total tracks
	pub total_tracks: Option<u32>,
	/// The disc number
	pub disc_number: Option<u32>,
	/// The total discs
	pub total_discs: Option<u32>,
}

impl<'a> AnyTag<'a> {
	/// Create an empty [`AnyTag`]
	pub fn new() -> Self {
		Self::default()
	}
	/// Returns `title`.
	pub fn title(&self) -> Option<&str> {
		self.title.as_deref()
	}
	/// Replaces `title`.
	pub fn set_title(&mut self, title: &'a str) {
		self.title = Some(title);
	}
	/// Returns `artist`
	pub fn artist(&self) -> Option<&str> {
		self.artist
	}
	/// Returns `album`
	pub fn album(&self) -> Album {
		self.album.clone()
	}
	/// Replaces `album`
	pub fn set_album(&mut self, album: Album<'a>) {
		self.album = album
	}
	/// Returns `year`
	pub fn year(&self) -> Option<i32> {
		self.year
	}
	/// Replaces `year`
	pub fn set_year(&mut self, year: i32) {
		self.year = Some(year);
	}
	/// Returns `track number`
	pub fn track_number(&self) -> Option<u32> {
		self.track_number
	}
	/// Returns `total_tracks`
	pub fn total_tracks(&self) -> Option<u32> {
		self.total_tracks
	}
	/// Returns `disc_number`
	pub fn disc_number(&self) -> Option<u32> {
		self.disc_number
	}
	/// Returns `total_discs`
	pub fn total_discs(&self) -> Option<u32> {
		self.total_tracks
	}
}
