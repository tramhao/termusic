//! Extract urls from M3U playlist files

#[derive(Debug, Clone, PartialEq)]
pub struct M3UItem {
    pub url: String,
}

pub fn decode(content: &str) -> Vec<M3UItem> {
    let lines = content.lines();
    let mut list = vec![];
    for line in lines {
        if line.starts_with('#') {
            continue;
        }

        list.push(M3UItem {
            url: String::from(line),
        });
    }
    list
}
