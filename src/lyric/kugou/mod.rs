pub mod model;

pub(crate) type NCMResult<T> = Result<T, Errors>;
use super::SongTag;
use lazy_static::lazy_static;
use model::*;
use regex::Regex;
use reqwest::blocking::Client;
use std::{collections::HashMap, time::Duration};
// use urlqstring::QueryParams;

lazy_static! {
    static ref _CSRF: Regex = Regex::new(r"_csrf=(?P<csrf>[^(;|$)]+)").unwrap();
}

static BASE_URL_SEARCH: &str =
    "http://mobilecdn.kugou.com/api/v3/search/song?format=json&showtype=1";
static BASE_URL_LYRIC_SEARCH: &str = "http://krcs.kugou.com/search";
static BASE_URL_LYRIC_DOWNLOAD: &str = "http://lyrics.kugou.com/download";

pub struct KugouApi {
    client: Client,
    #[allow(dead_code)]
    csrf: String,
}

impl KugouApi {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            // .cookies()
            .build()
            .expect("Initialize Web Client Failed!");
        Self {
            client,
            csrf: String::new(),
        }
    }

    fn request(&mut self, _params: HashMap<&str, &str>) -> NCMResult<String> {
        let url = BASE_URL_SEARCH.to_string();
        self.client
            .get(&url)
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)
    }

    #[allow(unused)]
    pub fn search(
        &mut self,
        keywords: String,
        types: u32,
        offset: u16,
        limit: u16,
    ) -> NCMResult<String> {
        let result = self
            .client
            .get(BASE_URL_SEARCH)
            .query(&[
                ("keyword", keywords),
                ("page", offset.to_string()),
                ("pagesize", limit.to_string()),
                ("showtype", 1.to_string()),
            ])
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)?;

        match types {
            1 => to_song_info(result, Parse::SEARCH).and_then(|s| Ok(serde_json::to_string(&s)?)),
            100 => to_singer_info(result).and_then(|s| Ok(serde_json::to_string(&s)?)),
            _ => Err(Errors::NoneError),
        }
    }

    // search and download lyrics
    // music_id: 歌曲id
    pub fn song_lyric(&mut self, music_id: String) -> NCMResult<String> {
        let result = self
            .client
            .get(BASE_URL_LYRIC_SEARCH)
            .query(&[
                ("keyword", "%20-%20".to_string()),
                ("ver", 1.to_string()),
                ("hash", music_id),
                ("client", "mobi".to_string()),
                ("man", "yes".to_string()),
            ])
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)?;

        let (accesskey, id) = to_lyric_id_accesskey(result)?;

        let result = self
            .client
            .get(BASE_URL_LYRIC_DOWNLOAD)
            .query(&[
                ("charset", "utf8".to_string()),
                ("accesskey", accesskey),
                ("id", id),
                ("client", "mobi".to_string()),
                ("fmt", "lrc".to_string()),
                ("ver", 1.to_string()),
            ])
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)?;

        to_lyric(result)
    }

    // 歌曲 URL
    // ids: 歌曲列表
    #[allow(unused)]
    pub fn songs_url(&mut self, id: String) -> NCMResult<Vec<SongUrl>> {
        let url = "http://media.store.kugou.com/v1/get_res_privilege";
        let mut params = HashMap::new();
        params.insert("relate", 1.to_string());
        params.insert("userid", "0".to_string());
        params.insert("vip", 0.to_string());
        params.insert("appid", 1000.to_string());
        params.insert("token", "".to_string());
        params.insert("behavior", "download".to_string());
        params.insert("area_code", "1".to_string());
        params.insert("clientver", "8990".to_string());
        let mut params_resource = HashMap::new();
        params_resource.insert("id", 0.to_string());
        params_resource.insert("type", "audio".to_string());
        params_resource.insert("hash", id);
        let params_resource_string = serde_json::to_string_pretty(&params_resource)?;
        params.insert("resource", params_resource_string);
        println!("{}", serde_json::to_string(&params)?);

        let result = self
            .client
            .post(url)
            .json(&params)
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)?;

        println!("{}", result);

        to_song_url(result)
    }

    // 歌曲详情
    // ids: 歌曲 id 列表
    #[allow(unused)]
    pub fn songs_detail(&mut self, ids: &[u64]) -> NCMResult<Vec<SongTag>> {
        let path = "/weapi/v3/song/detail";
        let mut params = HashMap::new();
        let c = format!(
            r#""[{{"id":{}}}]""#,
            ids.iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
        let ids = format!(
            r#""[{}]""#,
            ids.iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
        params.insert("c", &c[..]);
        params.insert("ids", &ids[..]);
        let result = self.request(params)?;
        to_song_info(result, Parse::USL)
    }
}
