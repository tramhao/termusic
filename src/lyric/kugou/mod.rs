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
static BASE_URL_LYRIC: &str =
    "http://www.kugou.com/yy/index.php?r=play/getdata&hash=CB7EE97F4CC11C4EA7A1FA4B516A5D97";

pub struct KugouApi {
    client: Client,
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
        let offset = offset.to_string();
        let limit = limit.to_string();
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

    // 查询歌词
    // music_id: 歌曲id
    #[allow(unused)]
    pub fn song_lyric(&mut self, music_id: String) -> NCMResult<String> {
        let csrf_token = self.csrf.to_owned();
        let path = "/weapi/song/lyric";
        let mut params = HashMap::new();
        params.insert("id", music_id.as_str());
        params.insert("lv", "-1");
        params.insert("tv", "-1");
        params.insert("csrf_token", &csrf_token);
        let result = self.request(params)?;
        to_lyric(result)
    }

    // 歌曲 URL
    // ids: 歌曲列表
    #[allow(unused)]
    pub fn songs_url(&mut self, ids: &[u64]) -> NCMResult<Vec<SongUrl>> {
        let csrf_token = self.csrf.to_owned();
        let path = "/weapi/song/enhance/player/url/v1";
        let mut params = HashMap::new();
        let ids = serde_json::to_string(ids)?;
        params.insert("ids", ids.as_str());
        params.insert("level", "standard");
        params.insert("encodeType", "aac");
        params.insert("csrf_token", &csrf_token);
        let result = self.request(params)?;
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
