// use crate::ui::activity::Loop;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use tuirealm::props::Color;

lazy_static! {

    /**
     * Regex matches:
     * - group 1: Red
     * - group 2: Green
     * - group 3: Blue
     */
    static ref COLOR_HEX_REGEX: Regex = Regex::new(r"#(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})").unwrap();
    /**
     * Regex matches:
     * - group 2: Red
     * - group 4: Green
     * - group 6: blue
     */
    static ref COLOR_RGB_REGEX: Regex = Regex::new(r"^(rgb)?\(?([01]?\d\d?|2[0-4]\d|25[0-5])(\W+)([01]?\d\d?|2[0-4]\d|25[0-5])\W+(([01]?\d\d?|2[0-4]\d|25[0-5])\)?)").unwrap();

}

#[derive(Clone, Deserialize, Serialize)]
pub struct Colors {
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub library_foreground: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub library_border: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub library_highlight: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            library_foreground: Color::Reset,
            library_border: Color::LightYellow,
            library_highlight: Color::LightYellow,
        }
    }
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    // Parse color
    match parse_color(s) {
        None => Err(DeError::custom("Invalid color")),
        Some(color) => Ok(color),
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_color<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert color to string
    let s: String = fmt_color(color);
    serializer.serialize_str(s.as_str())
}
/// ### `parse_color`
///
/// Parse color from string into a `Color` enum.
///
/// Color may be in different format:
///
/// 1. color name:
///     - Black,
///     - Blue,
///     - Cyan,
///     - `DarkGray`,
///     - Gray,
///     - Green,
///     - `LightBlue`,
///     - `LightCyan`,
///     - `LightGreen`,
///     - `LightMagenta`,
///     - `LightRed`,
///     - `LightYellow`,
///     - Magenta,
///     - Red,
///     - Reset,
///     - White,
///     - Yellow,
/// 2. Hex format:
///     - #f0ab05
///     - #AA33BC
/// 3. Rgb format:
///     - rgb(255, 64, 32)
///     - rgb(255,64,32)
///     - 255, 64, 32
#[allow(clippy::too_many_lines)]
fn parse_color(color: &str) -> Option<Color> {
    match color.to_lowercase().as_str() {
        // -- lib colors
        "black" => Some(Color::Black),
        "blue" => Some(Color::Blue),
        "cyan" => Some(Color::Cyan),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "default" => Some(Color::Reset),
        "gray" => Some(Color::Gray),
        "green" => Some(Color::Green),
        "lightblue" => Some(Color::LightBlue),
        "lightcyan" => Some(Color::LightCyan),
        "lightgreen" => Some(Color::LightGreen),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightred" => Some(Color::LightRed),
        "lightyellow" => Some(Color::LightYellow),
        "magenta" => Some(Color::Magenta),
        "red" => Some(Color::Red),
        "white" => Some(Color::White),
        "yellow" => Some(Color::Yellow),
        // -- css colors
        "aliceblue" => Some(Color::Rgb(240, 248, 255)),
        "antiquewhite" => Some(Color::Rgb(250, 235, 215)),
        "aqua" => Some(Color::Rgb(0, 255, 255)),
        "aquamarine" => Some(Color::Rgb(127, 255, 212)),
        "azure" => Some(Color::Rgb(240, 255, 255)),
        "beige" => Some(Color::Rgb(245, 245, 220)),
        "bisque" => Some(Color::Rgb(255, 228, 196)),
        "blanchedalmond" => Some(Color::Rgb(255, 235, 205)),
        "blueviolet" => Some(Color::Rgb(138, 43, 226)),
        "brown" => Some(Color::Rgb(165, 42, 42)),
        "burlywood" => Some(Color::Rgb(222, 184, 135)),
        "cadetblue" => Some(Color::Rgb(95, 158, 160)),
        "chartreuse" => Some(Color::Rgb(127, 255, 0)),
        "chocolate" => Some(Color::Rgb(210, 105, 30)),
        "coral" => Some(Color::Rgb(255, 127, 80)),
        "cornflowerblue" => Some(Color::Rgb(100, 149, 237)),
        "cornsilk" => Some(Color::Rgb(255, 248, 220)),
        "crimson" => Some(Color::Rgb(220, 20, 60)),
        "darkblue" => Some(Color::Rgb(0, 0, 139)),
        "darkcyan" => Some(Color::Rgb(0, 139, 139)),
        "darkgoldenrod" => Some(Color::Rgb(184, 134, 11)),
        "darkgreen" => Some(Color::Rgb(0, 100, 0)),
        "darkkhaki" => Some(Color::Rgb(189, 183, 107)),
        "darkmagenta" => Some(Color::Rgb(139, 0, 139)),
        "darkolivegreen" => Some(Color::Rgb(85, 107, 47)),
        "darkorange" => Some(Color::Rgb(255, 140, 0)),
        "darkorchid" => Some(Color::Rgb(153, 50, 204)),
        "darkred" => Some(Color::Rgb(139, 0, 0)),
        "darksalmon" => Some(Color::Rgb(233, 150, 122)),
        "darkseagreen" => Some(Color::Rgb(143, 188, 143)),
        "darkslateblue" => Some(Color::Rgb(72, 61, 139)),
        "darkslategray" | "darkslategrey" => Some(Color::Rgb(47, 79, 79)),
        "darkturquoise" => Some(Color::Rgb(0, 206, 209)),
        "darkviolet" => Some(Color::Rgb(148, 0, 211)),
        "deeppink" => Some(Color::Rgb(255, 20, 147)),
        "deepskyblue" => Some(Color::Rgb(0, 191, 255)),
        "dimgray" | "dimgrey" => Some(Color::Rgb(105, 105, 105)),
        "dodgerblue" => Some(Color::Rgb(30, 144, 255)),
        "firebrick" => Some(Color::Rgb(178, 34, 34)),
        "floralwhite" => Some(Color::Rgb(255, 250, 240)),
        "forestgreen" => Some(Color::Rgb(34, 139, 34)),
        "fuchsia" => Some(Color::Rgb(255, 0, 255)),
        "gainsboro" => Some(Color::Rgb(220, 220, 220)),
        "ghostwhite" => Some(Color::Rgb(248, 248, 255)),
        "gold" => Some(Color::Rgb(255, 215, 0)),
        "goldenrod" => Some(Color::Rgb(218, 165, 32)),
        "greenyellow" => Some(Color::Rgb(173, 255, 47)),
        "grey" => Some(Color::Rgb(128, 128, 128)),
        "honeydew" => Some(Color::Rgb(240, 255, 240)),
        "hotpink" => Some(Color::Rgb(255, 105, 180)),
        "indianred" => Some(Color::Rgb(205, 92, 92)),
        "indigo" => Some(Color::Rgb(75, 0, 130)),
        "ivory" => Some(Color::Rgb(255, 255, 240)),
        "khaki" => Some(Color::Rgb(240, 230, 140)),
        "lavender" => Some(Color::Rgb(230, 230, 250)),
        "lavenderblush" => Some(Color::Rgb(255, 240, 245)),
        "lawngreen" => Some(Color::Rgb(124, 252, 0)),
        "lemonchiffon" => Some(Color::Rgb(255, 250, 205)),
        "lightcoral" => Some(Color::Rgb(240, 128, 128)),
        "lightgoldenrodyellow" => Some(Color::Rgb(250, 250, 210)),
        "lightgray" | "lightgrey" => Some(Color::Rgb(211, 211, 211)),
        "lightpink" => Some(Color::Rgb(255, 182, 193)),
        "lightsalmon" => Some(Color::Rgb(255, 160, 122)),
        "lightseagreen" => Some(Color::Rgb(32, 178, 170)),
        "lightskyblue" => Some(Color::Rgb(135, 206, 250)),
        "lightslategray" | "lightslategrey" => Some(Color::Rgb(119, 136, 153)),
        "lightsteelblue" => Some(Color::Rgb(176, 196, 222)),
        "lime" => Some(Color::Rgb(0, 255, 0)),
        "limegreen" => Some(Color::Rgb(50, 205, 50)),
        "linen" => Some(Color::Rgb(250, 240, 230)),
        "maroon" => Some(Color::Rgb(128, 0, 0)),
        "mediumaquamarine" => Some(Color::Rgb(102, 205, 170)),
        "mediumblue" => Some(Color::Rgb(0, 0, 205)),
        "mediumorchid" => Some(Color::Rgb(186, 85, 211)),
        "mediumpurple" => Some(Color::Rgb(147, 112, 219)),
        "mediumseagreen" => Some(Color::Rgb(60, 179, 113)),
        "mediumslateblue" => Some(Color::Rgb(123, 104, 238)),
        "mediumspringgreen" => Some(Color::Rgb(0, 250, 154)),
        "mediumturquoise" => Some(Color::Rgb(72, 209, 204)),
        "mediumvioletred" => Some(Color::Rgb(199, 21, 133)),
        "midnightblue" => Some(Color::Rgb(25, 25, 112)),
        "mintcream" => Some(Color::Rgb(245, 255, 250)),
        "mistyrose" => Some(Color::Rgb(255, 228, 225)),
        "moccasin" => Some(Color::Rgb(255, 228, 181)),
        "navajowhite" => Some(Color::Rgb(255, 222, 173)),
        "navy" => Some(Color::Rgb(0, 0, 128)),
        "oldlace" => Some(Color::Rgb(253, 245, 230)),
        "olive" => Some(Color::Rgb(128, 128, 0)),
        "olivedrab" => Some(Color::Rgb(107, 142, 35)),
        "orange" => Some(Color::Rgb(255, 165, 0)),
        "orangered" => Some(Color::Rgb(255, 69, 0)),
        "orchid" => Some(Color::Rgb(218, 112, 214)),
        "palegoldenrod" => Some(Color::Rgb(238, 232, 170)),
        "palegreen" => Some(Color::Rgb(152, 251, 152)),
        "paleturquoise" => Some(Color::Rgb(175, 238, 238)),
        "palevioletred" => Some(Color::Rgb(219, 112, 147)),
        "papayawhip" => Some(Color::Rgb(255, 239, 213)),
        "peachpuff" => Some(Color::Rgb(255, 218, 185)),
        "peru" => Some(Color::Rgb(205, 133, 63)),
        "pink" => Some(Color::Rgb(255, 192, 203)),
        "plum" => Some(Color::Rgb(221, 160, 221)),
        "powderblue" => Some(Color::Rgb(176, 224, 230)),
        "purple" => Some(Color::Rgb(128, 0, 128)),
        "rebeccapurple" => Some(Color::Rgb(102, 51, 153)),
        "rosybrown" => Some(Color::Rgb(188, 143, 143)),
        "royalblue" => Some(Color::Rgb(65, 105, 225)),
        "saddlebrown" => Some(Color::Rgb(139, 69, 19)),
        "salmon" => Some(Color::Rgb(250, 128, 114)),
        "sandybrown" => Some(Color::Rgb(244, 164, 96)),
        "seagreen" => Some(Color::Rgb(46, 139, 87)),
        "seashell" => Some(Color::Rgb(255, 245, 238)),
        "sienna" => Some(Color::Rgb(160, 82, 45)),
        "silver" => Some(Color::Rgb(192, 192, 192)),
        "skyblue" => Some(Color::Rgb(135, 206, 235)),
        "slateblue" => Some(Color::Rgb(106, 90, 205)),
        "slategray" | "slategrey" => Some(Color::Rgb(112, 128, 144)),
        "snow" => Some(Color::Rgb(255, 250, 250)),
        "springgreen" => Some(Color::Rgb(0, 255, 127)),
        "steelblue" => Some(Color::Rgb(70, 130, 180)),
        "tan" => Some(Color::Rgb(210, 180, 140)),
        "teal" => Some(Color::Rgb(0, 128, 128)),
        "thistle" => Some(Color::Rgb(216, 191, 216)),
        "tomato" => Some(Color::Rgb(255, 99, 71)),
        "turquoise" => Some(Color::Rgb(64, 224, 208)),
        "violet" => Some(Color::Rgb(238, 130, 238)),
        "wheat" => Some(Color::Rgb(245, 222, 179)),
        "whitesmoke" => Some(Color::Rgb(245, 245, 245)),
        "yellowgreen" => Some(Color::Rgb(154, 205, 50)),
        // -- hex and rgb
        other => {
            // Try as hex
            parse_hex_color(other).map_or_else(|| parse_rgb_color(other), Some)
        }
    }
}
/// ### `parse_hex_color`
///
/// Try to parse a color in hex format, such as:
///
///     - #f0ab05
///     - #AA33BC
fn parse_hex_color(color: &str) -> Option<Color> {
    COLOR_HEX_REGEX.captures(color).map(|groups| {
        Color::Rgb(
            u8::from_str_radix(groups.get(1).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(2).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(3).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
        )
    })
}

/// ### `parse_rgb_color`
///
/// Try to parse a color in rgb format, such as:
///
///     - rgb(255, 64, 32)
///     - rgb(255,64,32)
///     - 255, 64, 32
fn parse_rgb_color(color: &str) -> Option<Color> {
    COLOR_RGB_REGEX.captures(color).map(|groups| {
        Color::Rgb(
            u8::from_str(groups.get(2).unwrap().as_str()).ok().unwrap(),
            u8::from_str(groups.get(4).unwrap().as_str()).ok().unwrap(),
            u8::from_str(groups.get(6).unwrap().as_str()).ok().unwrap(),
        )
    })
}

/// ### `fmt_color`
///
/// Format color
#[allow(clippy::too_many_lines)]
#[allow(clippy::trivially_copy_pass_by_ref)]
fn fmt_color(color: &Color) -> String {
    match color {
        Color::Black => "Black".to_string(),
        Color::Blue => "Blue".to_string(),
        Color::Cyan => "Cyan".to_string(),
        Color::DarkGray => "DarkGray".to_string(),
        Color::Gray => "Gray".to_string(),
        Color::Green => "Green".to_string(),
        Color::LightBlue => "LightBlue".to_string(),
        Color::LightCyan => "LightCyan".to_string(),
        Color::LightGreen => "LightGreen".to_string(),
        Color::LightMagenta => "LightMagenta".to_string(),
        Color::LightRed => "LightRed".to_string(),
        Color::LightYellow => "LightYellow".to_string(),
        Color::Magenta => "Magenta".to_string(),
        Color::Red => "Red".to_string(),
        Color::Reset | Color::Indexed(_) => "Default".to_string(),
        Color::White => "White".to_string(),
        Color::Yellow => "Yellow".to_string(),
        // -- css colors
        Color::Rgb(240, 248, 255) => "aliceblue".to_string(),
        Color::Rgb(250, 235, 215) => "antiquewhite".to_string(),
        Color::Rgb(0, 255, 255) => "aqua".to_string(),
        Color::Rgb(127, 255, 212) => "aquamarine".to_string(),
        Color::Rgb(240, 255, 255) => "azure".to_string(),
        Color::Rgb(245, 245, 220) => "beige".to_string(),
        Color::Rgb(255, 228, 196) => "bisque".to_string(),
        Color::Rgb(0, 0, 0) => "black".to_string(),
        Color::Rgb(255, 235, 205) => "blanchedalmond".to_string(),
        Color::Rgb(0, 0, 255) => "blue".to_string(),
        Color::Rgb(138, 43, 226) => "blueviolet".to_string(),
        Color::Rgb(165, 42, 42) => "brown".to_string(),
        Color::Rgb(222, 184, 135) => "burlywood".to_string(),
        Color::Rgb(95, 158, 160) => "cadetblue".to_string(),
        Color::Rgb(127, 255, 0) => "chartreuse".to_string(),
        Color::Rgb(210, 105, 30) => "chocolate".to_string(),
        Color::Rgb(255, 127, 80) => "coral".to_string(),
        Color::Rgb(100, 149, 237) => "cornflowerblue".to_string(),
        Color::Rgb(255, 248, 220) => "cornsilk".to_string(),
        Color::Rgb(220, 20, 60) => "crimson".to_string(),
        Color::Rgb(0, 0, 139) => "darkblue".to_string(),
        Color::Rgb(0, 139, 139) => "darkcyan".to_string(),
        Color::Rgb(184, 134, 11) => "darkgoldenrod".to_string(),
        Color::Rgb(169, 169, 169) => "darkgray".to_string(),
        Color::Rgb(0, 100, 0) => "darkgreen".to_string(),
        Color::Rgb(189, 183, 107) => "darkkhaki".to_string(),
        Color::Rgb(139, 0, 139) => "darkmagenta".to_string(),
        Color::Rgb(85, 107, 47) => "darkolivegreen".to_string(),
        Color::Rgb(255, 140, 0) => "darkorange".to_string(),
        Color::Rgb(153, 50, 204) => "darkorchid".to_string(),
        Color::Rgb(139, 0, 0) => "darkred".to_string(),
        Color::Rgb(233, 150, 122) => "darksalmon".to_string(),
        Color::Rgb(143, 188, 143) => "darkseagreen".to_string(),
        Color::Rgb(72, 61, 139) => "darkslateblue".to_string(),
        Color::Rgb(47, 79, 79) => "darkslategray".to_string(),
        Color::Rgb(0, 206, 209) => "darkturquoise".to_string(),
        Color::Rgb(148, 0, 211) => "darkviolet".to_string(),
        Color::Rgb(255, 20, 147) => "deeppink".to_string(),
        Color::Rgb(0, 191, 255) => "deepskyblue".to_string(),
        Color::Rgb(105, 105, 105) => "dimgray".to_string(),
        Color::Rgb(30, 144, 255) => "dodgerblue".to_string(),
        Color::Rgb(178, 34, 34) => "firebrick".to_string(),
        Color::Rgb(255, 250, 240) => "floralwhite".to_string(),
        Color::Rgb(34, 139, 34) => "forestgreen".to_string(),
        Color::Rgb(255, 0, 255) => "fuchsia".to_string(),
        Color::Rgb(220, 220, 220) => "gainsboro".to_string(),
        Color::Rgb(248, 248, 255) => "ghostwhite".to_string(),
        Color::Rgb(255, 215, 0) => "gold".to_string(),
        Color::Rgb(218, 165, 32) => "goldenrod".to_string(),
        Color::Rgb(128, 128, 128) => "gray".to_string(),
        Color::Rgb(0, 128, 0) => "green".to_string(),
        Color::Rgb(173, 255, 47) => "greenyellow".to_string(),
        Color::Rgb(240, 255, 240) => "honeydew".to_string(),
        Color::Rgb(255, 105, 180) => "hotpink".to_string(),
        Color::Rgb(205, 92, 92) => "indianred".to_string(),
        Color::Rgb(75, 0, 130) => "indigo".to_string(),
        Color::Rgb(255, 255, 240) => "ivory".to_string(),
        Color::Rgb(240, 230, 140) => "khaki".to_string(),
        Color::Rgb(230, 230, 250) => "lavender".to_string(),
        Color::Rgb(255, 240, 245) => "lavenderblush".to_string(),
        Color::Rgb(124, 252, 0) => "lawngreen".to_string(),
        Color::Rgb(255, 250, 205) => "lemonchiffon".to_string(),
        Color::Rgb(173, 216, 230) => "lightblue".to_string(),
        Color::Rgb(240, 128, 128) => "lightcoral".to_string(),
        Color::Rgb(224, 255, 255) => "lightcyan".to_string(),
        Color::Rgb(250, 250, 210) => "lightgoldenrodyellow".to_string(),
        Color::Rgb(211, 211, 211) => "lightgray".to_string(),
        Color::Rgb(144, 238, 144) => "lightgreen".to_string(),
        Color::Rgb(255, 182, 193) => "lightpink".to_string(),
        Color::Rgb(255, 160, 122) => "lightsalmon".to_string(),
        Color::Rgb(32, 178, 170) => "lightseagreen".to_string(),
        Color::Rgb(135, 206, 250) => "lightskyblue".to_string(),
        Color::Rgb(119, 136, 153) => "lightslategray".to_string(),
        Color::Rgb(176, 196, 222) => "lightsteelblue".to_string(),
        Color::Rgb(255, 255, 224) => "lightyellow".to_string(),
        Color::Rgb(0, 255, 0) => "lime".to_string(),
        Color::Rgb(50, 205, 50) => "limegreen".to_string(),
        Color::Rgb(250, 240, 230) => "linen".to_string(),
        Color::Rgb(128, 0, 0) => "maroon".to_string(),
        Color::Rgb(102, 205, 170) => "mediumaquamarine".to_string(),
        Color::Rgb(0, 0, 205) => "mediumblue".to_string(),
        Color::Rgb(186, 85, 211) => "mediumorchid".to_string(),
        Color::Rgb(147, 112, 219) => "mediumpurple".to_string(),
        Color::Rgb(60, 179, 113) => "mediumseagreen".to_string(),
        Color::Rgb(123, 104, 238) => "mediumslateblue".to_string(),
        Color::Rgb(0, 250, 154) => "mediumspringgreen".to_string(),
        Color::Rgb(72, 209, 204) => "mediumturquoise".to_string(),
        Color::Rgb(199, 21, 133) => "mediumvioletred".to_string(),
        Color::Rgb(25, 25, 112) => "midnightblue".to_string(),
        Color::Rgb(245, 255, 250) => "mintcream".to_string(),
        Color::Rgb(255, 228, 225) => "mistyrose".to_string(),
        Color::Rgb(255, 228, 181) => "moccasin".to_string(),
        Color::Rgb(255, 222, 173) => "navajowhite".to_string(),
        Color::Rgb(0, 0, 128) => "navy".to_string(),
        Color::Rgb(253, 245, 230) => "oldlace".to_string(),
        Color::Rgb(128, 128, 0) => "olive".to_string(),
        Color::Rgb(107, 142, 35) => "olivedrab".to_string(),
        Color::Rgb(255, 165, 0) => "orange".to_string(),
        Color::Rgb(255, 69, 0) => "orangered".to_string(),
        Color::Rgb(218, 112, 214) => "orchid".to_string(),
        Color::Rgb(238, 232, 170) => "palegoldenrod".to_string(),
        Color::Rgb(152, 251, 152) => "palegreen".to_string(),
        Color::Rgb(175, 238, 238) => "paleturquoise".to_string(),
        Color::Rgb(219, 112, 147) => "palevioletred".to_string(),
        Color::Rgb(255, 239, 213) => "papayawhip".to_string(),
        Color::Rgb(255, 218, 185) => "peachpuff".to_string(),
        Color::Rgb(205, 133, 63) => "peru".to_string(),
        Color::Rgb(255, 192, 203) => "pink".to_string(),
        Color::Rgb(221, 160, 221) => "plum".to_string(),
        Color::Rgb(176, 224, 230) => "powderblue".to_string(),
        Color::Rgb(128, 0, 128) => "purple".to_string(),
        Color::Rgb(102, 51, 153) => "rebeccapurple".to_string(),
        Color::Rgb(255, 0, 0) => "red".to_string(),
        Color::Rgb(188, 143, 143) => "rosybrown".to_string(),
        Color::Rgb(65, 105, 225) => "royalblue".to_string(),
        Color::Rgb(139, 69, 19) => "saddlebrown".to_string(),
        Color::Rgb(250, 128, 114) => "salmon".to_string(),
        Color::Rgb(244, 164, 96) => "sandybrown".to_string(),
        Color::Rgb(46, 139, 87) => "seagreen".to_string(),
        Color::Rgb(255, 245, 238) => "seashell".to_string(),
        Color::Rgb(160, 82, 45) => "sienna".to_string(),
        Color::Rgb(192, 192, 192) => "silver".to_string(),
        Color::Rgb(135, 206, 235) => "skyblue".to_string(),
        Color::Rgb(106, 90, 205) => "slateblue".to_string(),
        Color::Rgb(112, 128, 144) => "slategray".to_string(),
        Color::Rgb(255, 250, 250) => "snow".to_string(),
        Color::Rgb(0, 255, 127) => "springgreen".to_string(),
        Color::Rgb(70, 130, 180) => "steelblue".to_string(),
        Color::Rgb(210, 180, 140) => "tan".to_string(),
        Color::Rgb(0, 128, 128) => "teal".to_string(),
        Color::Rgb(216, 191, 216) => "thistle".to_string(),
        Color::Rgb(255, 99, 71) => "tomato".to_string(),
        Color::Rgb(64, 224, 208) => "turquoise".to_string(),
        Color::Rgb(238, 130, 238) => "violet".to_string(),
        Color::Rgb(245, 222, 179) => "wheat".to_string(),
        Color::Rgb(255, 255, 255) => "white".to_string(),
        Color::Rgb(245, 245, 245) => "whitesmoke".to_string(),
        Color::Rgb(255, 255, 0) => "yellow".to_string(),
        Color::Rgb(154, 205, 50) => "yellowgreen".to_string(),
        // -- others
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
    }
}
