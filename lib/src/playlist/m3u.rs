//! Extract urls from M3U playlist files

pub struct PlaylistItem {
    pub url: String,
}

pub fn decode(content: &str) -> Vec<PlaylistItem> {
    let lines = content.lines();
    let mut list = vec![];
    for line in lines {
        if line.starts_with('#') {
            continue;
        }

        list.push(PlaylistItem {
            url: String::from(line),
        });
    }
    list
}
