//! Decode File and Title parts from simple playlist PLS files

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct PLSItem {
    pub title: String,
    pub url: String,
}

pub fn decode(content: &str) -> Vec<PLSItem> {
    let lines = content.lines();
    let mut list = vec![];
    let mut found_pls = false;
    let mut map_urls = HashMap::new();
    let mut map_title = HashMap::new();
    let mut default_title = "";
    for line in lines {
        if line.starts_with('#') {
            continue;
        }
        if line.trim().to_lowercase() == "[playlist]" {
            found_pls = true;
        } else if found_pls {
            if line.starts_with("File") {
                let idend = line.find('=');
                if let Some(idend) = idend {
                    let (key, value) = line.split_at(idend);
                    let id: Result<u32, _> = key[4..idend].parse();
                    if let Ok(id) = id {
                        let (_, url) = value.split_at(1);
                        map_urls.insert(id, url);
                    }
                }
            } else if line.starts_with("Title") {
                let idend = line.find('=');
                if let Some(idend) = idend {
                    let (key, value) = line.split_at(idend);
                    let id: Result<u32, _> = key[5..idend].parse();
                    let (_, title) = value.split_at(1);
                    if let Ok(id) = id {
                        map_title.insert(id, title);
                    } else {
                        default_title = title;
                    }
                }
            }
        }
    }

    for (key, value) in map_urls {
        let title = map_title.get(&key).unwrap_or(&default_title);
        list.push(PLSItem {
            title: String::from(*title),
            url: String::from(value),
        });
    }

    list
}
