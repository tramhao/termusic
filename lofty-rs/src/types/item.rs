use crate::types::tag::TagType;

#[allow(unused_imports)]
use std::collections::HashMap;

#[allow(unused)]
macro_rules! first_key {
	($key:tt $(| $remaining:expr)*) => {
		$key
	};
}

#[allow(unused)]
pub(crate) use first_key;

// This is used to create the key/ItemKey maps
//
// First comes the feature attribute, followed by the name of the map.
// Ex:
//
// #[cfg(feature = "ape")]
// APE_MAP;
//
// This is followed by the key value pairs separated by `=>`, with the key being the
// format-specific key and the value being the appropriate ItemKey variant.
// Ex. "Artist" => Artist
//
// Some formats have multiple keys that map to the same ItemKey variant, which can be added with '|'.
// The standard key(s) **must** come before any popular non-standard keys.
// Keys should appear in order of popularity.
macro_rules! gen_map {
	($(#[$meta:meta])? $NAME:ident; $($($key:literal)|+ => $variant:ident),+) => {
		$(#[$meta])?
		lazy_static::lazy_static! {
			static ref $NAME: HashMap<&'static str, ItemKey> = {
				let mut map = HashMap::new();
				$(
					$(
						map.insert($key, ItemKey::$variant);
					)+
				)+
				map
			};
		}

		$(#[$meta])?
		impl $NAME {
			pub(crate) fn get_item_key(&self, key: &str) -> Option<ItemKey> {
				self.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.clone())
			}

			pub(crate) fn get_key(&self, item_key: &ItemKey) -> Option<&str> {
				match item_key {
					$(
						ItemKey::$variant => Some(first_key!($($key)|*)),
					)+
					_ => None
				}
			}
		}
	}
}

gen_map!(
	#[cfg(feature = "aiff_text_chunks")]
	AIFF_TEXT_MAP;

	"NAME" => TrackTitle,
	"AUTH" => TrackArtist,
	"(c) " => CopyrightMessage
);

gen_map!(
	#[cfg(feature = "ape")]
	APE_MAP;

	"Album" 	   	 			   => AlbumTitle,
	"DiscSubtitle" 	   			   => SetSubtitle,
	"Grouping"	  	  			   => ContentGroup,
	"Title"		   	  			   => TrackTitle,
	"Subtitle"	   	  			   => TrackSubtitle,
	"ALBUMSORT"	   	  			   => AlbumTitleSortOrder,
	"ALBUMARTISTSORT" 			   => AlbumArtistSortOrder,
	"TITLESORT"					   => TrackTitleSortOrder,
	"ARTISTSORT"	 			   => TrackArtistSortOrder,
	"Album Artist" | "ALBUMARTIST" => AlbumArtist,
	"Artist"					   => TrackArtist,
	"Arranger"					   => Arranger,
	"Writer"					   => Writer,
	"Composer"					   => Composer,
	"Conductor"					   => Conductor,
	"Engineer"					   => Engineer,
	"Lyricist"					   => Lyricist,
	"DjMixer"					   => MixDj,
	"Mixer"						   => MixEngineer,
	"Performer"					   => Performer,
	"Producer"					   => Producer,
	"Label"						   => Label,
	"MixArtist"					   => Remixer,
	"Disc"						   => DiscNumber,
	"Disc"						   => DiscTotal,
	"Track"						   => TrackNumber,
	"Track"						   => TrackTotal,
	"Year"						   => Year,
	"ISRC"						   => ISRC,
	"Barcode"					   => Barcode,
	"CatalogNumber"				   => CatalogNumber,
	"Compilation"				   => FlagCompilation,
	"Media"						   => OriginalMediaType,
	"EncodedBy"					   => EncodedBy,
	"Genre"						   => Genre,
	"Mood"						   => Mood,
	"Copyright"					   => CopyrightMessage,
	"Comment"					   => Comment,
	"language"					   => Language,
	"Script"					   => Script,
	"Lyrics"					   => Lyrics
);

gen_map! (
	#[cfg(feature = "id3v2")]
	ID3V2_MAP;

	"TALB" 			=> AlbumTitle,
	"TSST" 			=> SetSubtitle,
	"TIT1" | "GRP1" => ContentGroup,
	"TIT2"			=> TrackTitle,
	"TIT3" 			=> TrackSubtitle,
	"TOAL" 			=> OriginalAlbumTitle,
	"TOPE" 			=> OriginalArtist,
	"TOLY" 			=> OriginalLyricist,
	"TSOA" 			=> AlbumTitleSortOrder,
	"TSO2" 			=> AlbumArtistSortOrder,
	"TSOT" 			=> TrackTitleSortOrder,
	"TSOP" 			=> TrackArtistSortOrder,
	"TSOC" 			=> ComposerSortOrder,
	"TPE2" 			=> AlbumArtist,
	"TPE1" 			=> TrackArtist,
	"TEXT" 			=> Writer,
	"TCOM" 			=> Composer,
	"TPE3" 			=> Conductor,
	"TIPL" 			=> InvolvedPeople,
	"TEXT" 			=> Lyricist,
	"TMCL" 			=> MusicianCredits,
	"IPRO" 			=> Producer,
	"TPUB" 			=> Publisher,
	"TPUB" 			=> Label,
	"TRSN" 			=> InternetRadioStationName,
	"TRSO" 			=> InternetRadioStationOwner,
	"TPE4" 			=> Remixer,
	"TPOS" 			=> DiscNumber,
	"TPOS" 			=> DiscTotal,
	"TRCK" 			=> TrackNumber,
	"TRCK" 			=> TrackTotal,
	"POPM" 			=> Popularimeter,
	"TDRC" 			=> RecordingDate,
	"TDOR" 			=> OriginalReleaseDate,
	"TSRC" 			=> ISRC,
	"MVNM" 			=> Movement,
	"MVIN" 			=> MovementIndex,
	"TCMP" 			=> FlagCompilation,
	"PCST" 			=> FlagPodcast,
	"TFLT" 			=> FileType,
	"TOWN" 			=> FileOwner,
	"TDTG" 			=> TaggingTime,
	"TLEN" 			=> Length,
	"TOFN" 			=> OriginalFileName,
	"TMED" 			=> OriginalMediaType,
	"TENC" 			=> EncodedBy,
	"TSSE" 			=> EncoderSoftware,
	"TSSE" 			=> EncoderSettings,
	"TDEN" 			=> EncodingTime,
	"WOAF" 			=> AudioFileURL,
	"WOAS" 			=> AudioSourceURL,
	"WCOM" 			=> CommercialInformationURL,
	"WCOP" 			=> CopyrightURL,
	"WOAR" 			=> TrackArtistURL,
	"WORS" 			=> RadioStationURL,
	"WPAY" 			=> PaymentURL,
	"WPUB" 			=> PublisherURL,
	"TCON" 			=> Genre,
	"TLEY" 			=> InitialKey,
	"TMOO" 			=> Mood,
	"TBPM" 			=> BPM,
	"TCOP" 			=> CopyrightMessage,
	"TDES" 			=> PodcastDescription,
	"TCAT" 			=> PodcastSeriesCategory,
	"WFED" 			=> PodcastURL,
	"TDRL" 			=> PodcastReleaseDate,
	"TGID" 			=> PodcastGlobalUniqueID,
	"TKWD" 			=> PodcastKeywords,
	"COMM" 			=> Comment,
	"TLAN" 			=> Language,
	"USLT" 			=> Lyrics
);

gen_map! (
	#[cfg(feature = "mp4_ilst")]
	ILST_MAP;

	"\u{a9}alb" 						  => AlbumTitle,
	"----:com.apple.iTunes:DISCSUBTITLE"  => SetSubtitle,
	"tvsh" 								  => ShowName,
	"\u{a9}grp"						      => ContentGroup,
	"\u{a9}nam"							  => TrackTitle,
	"----:com.apple.iTunes:SUBTITLE"	  => TrackSubtitle,
	"soal"								  => AlbumTitleSortOrder,
	"soaa"								  => AlbumArtistSortOrder,
	"sonm"								  => TrackTitleSortOrder,
	"soar"								  => TrackArtistSortOrder,
	"sosn"								  => ShowNameSortOrder,
	"soco"								  => ComposerSortOrder,
	"aART"								  => AlbumArtist,
	"\u{a9}ART"							  => TrackArtist,
	"\u{a9}wrt"							  => Composer,
	"----:com.apple.iTunes:CONDUCTOR"	  => Conductor,
	"----:com.apple.iTunes:ENGINEER"	  => Engineer,
	"----:com.apple.iTunes:LYRICIST"	  => Lyricist,
	"----:com.apple.iTunes:DJMIXER"		  => MixDj,
	"----:com.apple.iTunes:MIXER"		  => MixEngineer,
	"----:com.apple.iTunes:PRODUCER"	  => Producer,
	"----:com.apple.iTunes:LABEL"		  => Label,
	"----:com.apple.iTunes:REMIXER"		  => Remixer,
	"disk"								  => DiscNumber,
	"disk"								  => DiscTotal,
	"trkn"								  => TrackNumber,
	"trkn"								  => TrackTotal,
	"rate"								  => LawRating,
	"\u{a9}day"							  => RecordingDate,
	"----:com.apple.iTunes:ISRC"		  => ISRC,
	"----:com.apple.iTunes:BARCODE"		  => Barcode,
	"----:com.apple.iTunes:CATALOGNUMBER" => CatalogNumber,
	"cpil"								  => FlagCompilation,
	"pcst"								  => FlagPodcast,
	"----:com.apple.iTunes:MEDIA"		  => OriginalMediaType,
	"\u{a9}too"							  => EncoderSoftware,
	"\u{a9}gen"							  => Genre,
	"----:com.apple.iTunes:MOOD"		  => Mood,
	"tmpo"								  => BPM,
	"cprt"								  => CopyrightMessage,
	"----:com.apple.iTunes:LICENSE"		  => License,
	"ldes"								  => PodcastDescription,
	"catg"								  => PodcastSeriesCategory,
	"purl"								  => PodcastURL,
	"egid"								  => PodcastGlobalUniqueID,
	"keyw"								  => PodcastKeywords,
	"\u{a9}cmt"							  => Comment,
	"desc"								  => Description,
	"----:com.apple.iTunes:LANGUAGE"	  => Language,
	"----:com.apple.iTunes:SCRIPT"		  => Script,
	"\u{a9}lyr"							  => Lyrics
);

gen_map! (
	#[cfg(feature = "riff_info_list")]
	RIFF_INFO_MAP;

	"IPRD" 			=> AlbumTitle,
	"INAM" 			=> TrackTitle,
	"IART" 			=> TrackArtist,
	"IWRI" 			=> Writer,
	"IMUS" 			=> Composer,
	"IPRO" 			=> Producer,
	"IPRT" | "ITRK" => TrackNumber,
	"IFRM" 			=> TrackTotal,
	"IRTD" 			=> LawRating,
	"ICRD" 			=> RecordingDate,
	"ISRF" 			=> OriginalMediaType,
	"ITCH" 			=> EncodedBy,
	"ISFT" 			=> EncoderSoftware,
	"IGNR" 			=> Genre,
	"ICOP" 			=> CopyrightMessage,
	"ICMT" 			=> Comment,
	"ILNG" 			=> Language
);

gen_map!(
	#[cfg(feature = "vorbis_comments")]
	VORBIS_MAP;

	"ALBUM" 	      		   	   => AlbumTitle,
	"DISCSUBTITLE"    		   	   => SetSubtitle,
	"GROUPING"	   	  		   	   => ContentGroup,
	"TITLE"		   	  		   	   => TrackTitle,
	"SUBTITLE"	   	  		   	   => TrackSubtitle,
	"ALBUMSORT"	   	  		   	   => AlbumTitleSortOrder,
	"ALBUMARTISTSORT" 		   	   => AlbumArtistSortOrder,
	"TITLESORT" 	  		   	   => TrackTitleSortOrder,
	"ARTISTSORT"	  		   	   => TrackArtistSortOrder,
	"ALBUMARTIST"	  		   	   => AlbumArtist,
	"ARTIST"		  		   	   => TrackArtist,
	"ARRANGER"		  		   	   => Arranger,
	"AUTHOR" | "WRITER" 	   	   => Writer,
	"COMPOSER"				   	   => Composer,
	"CONDUCTOR"				   	   => Conductor,
	"ENGINEER"				   	   => Engineer,
	"LYRICIST"				   	   => Lyricist,
	"DJMIXER"				   	   => MixDj,
	"MIXER"					   	   => MixEngineer,
	"PERFORMER"				   	   => Performer,
	"PRODUCER"				   	   => Producer,
	"PUBLISHER"				   	   => Publisher,
	"LABEL"					   	   => Label,
	"REMIXER"				   	   => Remixer,
	"DISCNUMBER"			   	   => DiscNumber,
	"DISCTOTAL" | "TOTALDISCS" 	   => DiscTotal,
	"TRACKNUMBER"			   	   => TrackNumber,
	"TRACKTOTAL" | "TOTALTRACKS"   => TrackTotal,
	"DATE"						   => RecordingDate,
	"YEAR" 			   			   => Year,
	"ORIGINALDATE" 				   => OriginalReleaseDate,
	"ISRC" 						   => ISRC,
	"CATALOGNUMBER" 			   => CatalogNumber,
	"COMPILATION" 				   => FlagCompilation,
	"MEDIA" 					   => OriginalMediaType,
	"ENCODED-BY" 				   => EncodedBy,
	"ENCODER" 					   => EncoderSoftware,
	"ENCODING" | "ENCODERSETTINGS" => EncoderSettings,
	"GENRE" 					   => Genre,
	"MOOD" 					 	   => Mood,
	"BPM" 					 	   => BPM,
	"COPYRIGHT" 				   => CopyrightMessage,
	"LICENSE" 					   => License,
	"COMMENT" 					   => Comment,
	"LANGUAGE" 					   => Language,
	"SCRIPT" 					   => Script,
	"LYRICS" 					   => Lyrics
);

macro_rules! gen_item_keys {
	(
		MAPS => [
			$(
				$(#[$feat:meta])?
				[$tag_type:pat, $MAP:ident]
			),+
		];
		KEYS => [
			$($variant:ident),+ $(,)?
		]
	) => {
		#[derive(PartialEq, Clone, Debug, Eq, Hash)]
		#[allow(missing_docs)]
		#[non_exhaustive]
		/// A generic representation of a tag's key
		pub enum ItemKey {
			$(
				$variant,
			)+
			/// When a key couldn't be mapped to another variant
			///
			/// This **will not** allow writing keys that are out of spec (Eg. ID3v2.4 frame IDs **must** be 4 characters)
			Unknown(String),
		}

		impl ItemKey {
			/// Map a format specific key to an ItemKey
			///
			/// NOTE: If used with ID3v2, this will only check against the ID3v2.4 keys.
			/// If you wish to use a V2 or V3 key, see [`upgrade_v2`](crate::id3::v2::upgrade_v2) and [`upgrade_v3`](crate::id3::v2::upgrade_v3)
			pub fn from_key(tag_type: TagType, key: &str) -> Self {
				match tag_type {
					$(
						$(#[$feat])?
						$tag_type => $MAP.get_item_key(key).unwrap_or_else(|| Self::Unknown(key.to_string())),
					)+
					_ => Self::Unknown(key.to_string())
				}
			}
			/// Maps the variant to a format-specific key
			///
			/// Use `allow_unknown` to include [`ItemKey::Unknown`]. It is up to the caller
			/// to determine if the unknown key actually fits the format's specifications.
			pub fn map_key(&self, tag_type: TagType, allow_unknown: bool) -> Option<&str> {
				match tag_type {
					$(
						$(#[$feat])?
						$tag_type => if let Some(key) = $MAP.get_key(self) {
							return Some(key)
						},
					)+
					_ => {}
				}

				if let ItemKey::Unknown(ref unknown) = self {
					if allow_unknown {
						return Some(unknown)
					}
				}

				None
			}
		}
	}
}

gen_item_keys!(
	MAPS => [
		#[cfg(feature = "aiff_text_chunks")]
		[TagType::AiffText, AIFF_TEXT_MAP],

		#[cfg(feature = "ape")]
		[TagType::Ape, APE_MAP],

		#[cfg(feature = "id3v2")]
		[TagType::Id3v2, ID3V2_MAP],

		#[cfg(feature = "mp4_ilst")]
		[TagType::Mp4Ilst, ILST_MAP],

		#[cfg(feature = "riff_info_list")]
		[TagType::RiffInfo, RIFF_INFO_MAP],

		#[cfg(feature = "vorbis_comments")]
		[TagType::VorbisComments, VORBIS_MAP]
	];

	KEYS => [
		// Titles
		AlbumTitle,
		SetSubtitle,
		ShowName,
		ContentGroup,
		TrackTitle,
		TrackSubtitle,

		// Original names
		OriginalAlbumTitle,
		OriginalArtist,
		OriginalLyricist,

		// Sorting
		AlbumTitleSortOrder,
		AlbumArtistSortOrder,
		TrackTitleSortOrder,
		TrackArtistSortOrder,
		ShowNameSortOrder,
		ComposerSortOrder,

		// People & Organizations
		AlbumArtist,
		TrackArtist,
		Arranger,
		Writer,
		Composer,
		Conductor,
		Engineer,
		InvolvedPeople,
		Lyricist,
		MixDj,
		MixEngineer,
		MusicianCredits,
		Performer,
		Producer,
		Publisher,
		Label,
		InternetRadioStationName,
		InternetRadioStationOwner,
		Remixer,

		// Counts & Indexes
		DiscNumber,
		DiscTotal,
		TrackNumber,
		TrackTotal,
		Popularimeter,
		LawRating,

		// Dates
		RecordingDate,
		Year,
		OriginalReleaseDate,

		// Identifiers
		ISRC,
		Barcode,
		CatalogNumber,
		Movement,
		MovementIndex,

		// Flags
		FlagCompilation,
		FlagPodcast,

		// File Information
		FileType,
		FileOwner,
		TaggingTime,
		Length,
		OriginalFileName,
		OriginalMediaType,

		// Encoder information
		EncodedBy,
		EncoderSoftware,
		EncoderSettings,
		EncodingTime,

		// URLs
		AudioFileURL,
		AudioSourceURL,
		CommercialInformationURL,
		CopyrightURL,
		TrackArtistURL,
		RadioStationURL,
		PaymentURL,
		PublisherURL,

		// Style
		Genre,
		InitialKey,
		Mood,
		BPM,

		// Legal
		CopyrightMessage,
		License,

		// Podcast
		PodcastDescription,
		PodcastSeriesCategory,
		PodcastURL,
		PodcastReleaseDate,
		PodcastGlobalUniqueID,
		PodcastKeywords,

		// Miscellaneous
		Comment,
		Description,
		Language,
		Script,
		Lyrics,
	]
);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Represents a tag item's value
pub enum ItemValue {
	/// Any UTF-8 encoded text
	Text(String),
	/// Any UTF-8 encoded locator of external information
	///
	/// This is only gets special treatment in `ID3v2` and `APE` tags, being written
	/// as a normal string in other tags
	Locator(String),
	/// Binary information
	Binary(Vec<u8>),
}

pub(crate) enum ItemValueRef<'a> {
	Text(&'a str),
	Locator(&'a str),
	Binary(&'a [u8]),
}

impl<'a> Into<ItemValueRef<'a>> for &'a ItemValue {
	fn into(self) -> ItemValueRef<'a> {
		match self {
			ItemValue::Text(text) => ItemValueRef::Text(text),
			ItemValue::Locator(locator) => ItemValueRef::Locator(locator),
			ItemValue::Binary(binary) => ItemValueRef::Binary(binary),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Represents a tag item (key/value)
pub struct TagItem {
	pub(crate) item_key: ItemKey,
	pub(crate) item_value: ItemValue,
}

impl TagItem {
	/// Create a new [`TagItem`]
	///
	/// NOTES:
	///
	/// * This will check for validity based on the [`TagType`].
	/// * If the [`ItemKey`] does not map to a key in the target format, `None` will be returned.
	/// * It is pointless to do this if you plan on using [`Tag::insert_item`](crate::Tag::insert_item), as it does validity checks itself.
	pub fn new_checked(
		tag_type: TagType,
		item_key: ItemKey,
		item_value: ItemValue,
	) -> Option<Self> {
		item_key.map_key(tag_type, false).is_some().then(|| Self {
			item_key,
			item_value,
		})
	}

	/// Create a new [`TagItem`]
	pub fn new(item_key: ItemKey, item_value: ItemValue) -> Self {
		Self {
			item_key,
			item_value,
		}
	}

	/// Returns a reference to the [`ItemKey`]
	pub fn key(&self) -> &ItemKey {
		&self.item_key
	}

	/// Returns a reference to the [`ItemValue`]
	pub fn value(&self) -> &ItemValue {
		&self.item_value
	}

	pub(crate) fn re_map(&self, tag_type: TagType) -> Option<()> {
		#[cfg(feature = "id3v1")]
		if tag_type == TagType::Id3v1 {
			use crate::id3::v1::constants::VALID_ITEMKEYS;

			return VALID_ITEMKEYS.contains(&self.item_key).then(|| ());
		}

		self.item_key.map_key(tag_type, false).is_some().then(|| ())
	}
}
