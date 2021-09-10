use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, Meta};

#[proc_macro_derive(LoftyTag, attributes(expected))]
#[allow(clippy::too_many_lines)]
pub fn impl_tag(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let data = match input.data {
		Data::Struct(data) => data,
		_ => {
			return Error::new(name.span(), "LoftyTag is only applicable to structs")
				.into_compile_error()
				.into()
		},
	};

	let fields = match data.fields {
		Fields::Named(fields) => fields.named,
		_ => {
			return Error::new(name.span(), format!("`{}` has no named fields", name))
				.into_compile_error()
				.into()
		},
	};

	let inner_ident = "inner".to_string();
	let format_ident = "_format".to_string();

	let mut inner_ty = None;
	let mut tag_type = None;

	for field in fields.iter() {
		let ident = field.ident.as_ref().unwrap().to_string();

		if ident == inner_ident {
			inner_ty = Some(field.ty.clone())
		}

		if ident == format_ident {
			let expected_attr = field
				.attrs
				.iter()
				.find(|a| a.path.is_ident("expected"))
				.expect(&*format!(
					"`{}`'s `_format` field has no `expected` attribute",
					name
				));

			if let Ok(Meta::List(list)) = expected_attr.parse_meta() {
				tag_type = Some(list.nested)
			}
		}
	}

	let inner = inner_ty.expect(&*format!("`{}` has no `inner` field", name));
	let tag_type = tag_type.expect(&*format!("`{}` has no `_format` field", name));

	TokenStream::from(quote! {
		impl #name {
			/// Creates a new default tag
			pub fn new() -> Self {
				Self {
					inner: #inner::default(),
					properties: FileProperties::default(),
					_format: #tag_type
				}
			}
		}

		use std::any::Any;

		impl ToAnyTag for #name {
			fn to_anytag(&self) -> AnyTag<'_> {
				self.into()
			}
		}

		impl ToAny for #name {
			fn to_any(&self) -> &dyn Any {
				self
			}
			fn to_any_mut(&mut self) -> &mut dyn Any {
				self
			}
		}

		impl AudioTag for #name {}

		// From wrapper to inner (same type)
		impl From<#name> for #inner {
			fn from(inp: #name) -> Self {
				inp.inner
			}
		}

		// From inner to wrapper (same type)
		impl From<#inner> for #name {
			fn from(inp: #inner) -> Self {
				Self {
					inner: inp,
					properties: FileProperties::default(),
					_format: #tag_type
				}
			}
		}

		impl<'a> From<&'a #name> for AnyTag<'a> {
			fn from(inp: &'a #name) -> Self {
				Self {
					title: inp.title(),
					artist: inp.artist(),
					year: inp.year().map(|y| y as i32),
					album: Album::new(
						inp.album_title(),
						inp.album_artist(),
						inp.album_covers(),
					),
					track_number: inp.track_number(),
					total_tracks: inp.total_tracks(),
					disc_number: inp.disc_number(),
					total_discs: inp.total_discs(),
					comments: None, // TODO
					date: inp.date(),
				}
			}
		}

		impl<'a> From<AnyTag<'a>> for #name {
			fn from(inp: AnyTag<'a>) -> Self {
				let mut tag = #name::new();

				if let Some(v) = inp.title() {
					tag.set_title(v)
				}
				if let Some(v) = inp.artist() {
					tag.set_artist(&v)
				}
				if let Some(v) = inp.year {
					tag.set_year(v)
				}
				if let Some(v) = inp.track_number() {
					tag.set_track_number(v)
				}
				if let Some(v) = inp.total_tracks() {
					tag.set_total_tracks(v)
				}
				if let Some(v) = inp.disc_number() {
					tag.set_disc_number(v)
				}
				if let Some(v) = inp.total_discs() {
					tag.set_total_discs(v)
				}

				let album = inp.album();

				if let Some(v) = album.title {
					tag.set_album_title(v)
				}
				if let Some(v) = album.artist {
					tag.set_album_artist(v)
				}
				if let Some(v) = album.covers.0 {
					tag.set_front_cover(v)
				}
				if let Some(v) = album.covers.1 {
					tag.set_back_cover(v)
				}

				tag
			}
		}

		// From dyn AudioTag to wrapper (any type)
		impl From<Box<dyn AudioTag>> for #name {
			fn from(inp: Box<dyn AudioTag>) -> Self {
				let mut inp = inp;
				if let Some(t_refmut) = inp.to_any_mut().downcast_mut::<#name>() {
					let t = std::mem::replace(t_refmut, #name::new()); // TODO: can we avoid creating the dummy tag?
					t
				} else {
					let mut t = inp.to_dyn_tag(#tag_type);
					let t_refmut = t.to_any_mut().downcast_mut::<#name>().unwrap();
					let t = std::mem::replace(t_refmut, #name::new());
					t
				}
			}
		}

		// From dyn AudioTag to inner (any type)
		impl From<Box<dyn AudioTag>> for #inner {
			fn from(inp: Box<dyn AudioTag>) -> Self {
				let t: #name = inp.into();
				t.into()
			}
		}
	})
}

#[proc_macro]
pub fn str_accessor(input: TokenStream) -> TokenStream {
	let input_str = input.to_string();
	let name = input_str.replace("_", " ");

	format!(
		"/// Returns the {display}
			fn {ident}(&self) -> Option<&str> {{
				None
			}}
			/// Sets the {display}
			fn set_{ident}(&mut self, _{ident}: &str) {{}}
			/// Removes the {display}
			fn remove_{ident}(&mut self) {{}}
			",
		ident = input_str,
		display = name,
	)
	.parse()
	.expect("Unable to parse str accessor:")
}

#[proc_macro]
pub fn u16_accessor(input: TokenStream) -> TokenStream {
	let input_str = input.to_string();
	let name = input_str.replace("_", " ");

	format!(
		"/// Returns the {display}
			fn {ident}(&self) -> Option<u16> {{
				None
			}}
			/// Sets the {display}
			fn set_{ident}(&mut self, _{ident}: u16) {{}}
			/// Removes the {display}
			fn remove_{ident}(&mut self) {{}}
			",
		ident = input_str,
		display = name,
	)
	.parse()
	.expect("Unable to parse u16 accessor:")
}

#[proc_macro]
pub fn u32_accessor(input: TokenStream) -> TokenStream {
	let input_str = input.to_string();
	let name = input_str.replace("_", " ");

	format!(
		"/// Returns the {display}
			fn {ident}(&self) -> Option<u32> {{
				None
			}}
			/// Sets the {display}
			fn set_{ident}(&mut self, _{ident}: u32) {{}}
			/// Removes the {display}
			fn remove_{ident}(&mut self) {{}}
			",
		ident = input_str,
		display = name,
	)
	.parse()
	.expect("Unable to parse u32 accessor:")
}

#[proc_macro]
pub fn i32_accessor(input: TokenStream) -> TokenStream {
	let input_str = input.to_string();
	let name = input_str.replace("_", " ");

	format!(
		"/// Returns the {display}
			fn {ident}(&self) -> Option<i32> {{
				None
			}}
			/// Sets the {display}
			fn set_{ident}(&mut self, _{ident}: i32) {{}}
			/// Removes the {display}
			fn remove_{ident}(&mut self) {{}}
			",
		ident = input_str,
		display = name,
	)
	.parse()
	.expect("Unable to parse i32 accessor:")
}

/// Used to create simple tag methods for getting/setting/removing based on a key
#[proc_macro]
pub fn get_set_methods(input: TokenStream) -> TokenStream {
	let input = input.to_string();
	let mut input_split = input.split(',');

	let name = input_split.next().expect("No identifier provided");
	let key = input_split.next().expect("No key provided");

	format!(
		"fn {ident}(&self) -> Option<&str> {{
		self.get_value({key})
	}}
	fn set_{ident}(&mut self, {ident}: &str) {{
		self.set_value({key}, {ident})
	}}
	fn remove_{ident}(&mut self) {{
		self.remove_key({key})
	}}",
		ident = name,
		key = key
	)
	.parse()
	.expect("Unable to parse getters/setters:")
}
