mod encrypt;
pub(crate) mod model;

pub(crate) type NCMResult<T> = Result<T, Errors>;
use super::SongTag;
use encrypt::Crypto;
use lazy_static::lazy_static;
use model::*;
use regex::Regex;
use reqwest::blocking::Client;
use std::{collections::HashMap, time::Duration};
use urlqstring::QueryParams;

lazy_static! {
    static ref _CSRF: Regex = Regex::new(r"_csrf=(?P<csrf>[^(;|$)]+)").unwrap();
}

static BASE_URL: &str = "https://music.163.com";

const LINUX_USER_AGNET: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36";

const USER_AGENT_LIST: [&str; 14] = [
    "Mozilla/5.0 (iPhone; CPU iPhone OS 9_1 like Mac OS X) AppleWebKit/601.1.46 (KHTML, like Gecko) Version/9.0 Mobile/13B143 Safari/601.1",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 9_1 like Mac OS X) AppleWebKit/601.1.46 (KHTML, like Gecko) Version/9.0 Mobile/13B143 Safari/601.1",
    "Mozilla/5.0 (Linux; Android 5.0; SM-G900P Build/LRX21T) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 5.1.1; Nexus 6 Build/LYZ28E) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Mobile Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 10_3_2 like Mac OS X) AppleWebKit/603.2.4 (KHTML, like Gecko) Mobile/14F89;GameHelper",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 10_0 like Mac OS X) AppleWebKit/602.1.38 (KHTML, like Gecko) Version/10.0 Mobile/14A300 Safari/602.1",
    "Mozilla/5.0 (iPad; CPU OS 10_0 like Mac OS X) AppleWebKit/602.1.38 (KHTML, like Gecko) Version/10.0 Mobile/14A300 Safari/602.1",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.12; rv:46.0) Gecko/20100101 Firefox/46.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_5) AppleWebKit/603.2.4 (KHTML, like Gecko) Version/10.1.1 Safari/603.2.4",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:46.0) Gecko/20100101 Firefox/46.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.103 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/42.0.2311.135 Safari/537.36 Edge/13.1058",
];

pub struct MusicApi {
    client: Client,
    csrf: String,
}

#[allow(unused)]
enum CryptoApi {
    Weapi,
    Linuxapi,
}

impl MusicApi {
    #[allow(unused)]
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            // .cookies()
            .build()
            .expect("初始化网络请求失败!");
        Self {
            client,
            csrf: String::new(),
        }
    }

    // 发送请求
    // method: 请求方法
    // path: 请求路径
    // params: 请求参数
    // cryptoapi: 请求加密方式
    // ua: 要使用的 USER_AGENT_LIST
    fn request(
        &mut self,
        method: Method,
        path: &str,
        params: HashMap<&str, &str>,
        cryptoapi: CryptoApi,
        ua: &str,
    ) -> NCMResult<String> {
        let mut url = format!("{}{}", BASE_URL, path);
        match method {
            Method::POST => {
                let user_agent = match cryptoapi {
                    CryptoApi::Linuxapi => LINUX_USER_AGNET.to_string(),
                    CryptoApi::Weapi => choose_user_agent(ua).to_string(),
                };
                let body = match cryptoapi {
                    CryptoApi::Linuxapi => {
                        let data = format!(
                            r#"{{"method":"linuxapi","url":"{}","params":{}}}"#,
                            url.replace("weapi", "api"),
                            QueryParams::from_map(params).json()
                        );
                        url = "https://music.163.com/api/linux/forward".to_owned();
                        Crypto::linuxapi(&data)
                    }
                    CryptoApi::Weapi => {
                        let mut params = params;
                        params.insert("csrf_token", &self.csrf[..]);
                        Crypto::weapi(&QueryParams::from_map(params).json())
                    }
                };

                let response = self
                    .client
                    .post(&url)
                    .header("Cookie", "os=pc; appver=2.7.1.198277")
                    .header("Accept", "*/*")
                    .header("Accept-Encoding", "gzip,deflate,br")
                    .header("Accept-Language", "en-US,en;q=0.5")
                    .header("Connection", "keep-alive")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("Host", "music.163.com")
                    .header("Referer", "https://music.163.com")
                    .header("User-Agent", user_agent)
                    .body(body)
                    // .build();
                    .send()
                    .map_err(|_| Errors::NoneError)?;
                // .unwrap();
                // let mut response = self.client.request(reqwest::Method::POST,request).map_err(|_| Errors::NoneError)?;
                if self.csrf.is_empty() {
                    for (k, v) in response.headers() {
                        let v = v.to_str().unwrap_or("");
                        if k.eq("set-cookie") && v.contains("__csrf") {
                            let csrf_token = if let Some(caps) = _CSRF.captures(v) {
                                caps.name("csrf").unwrap().as_str()
                            } else {
                                ""
                            };
                            self.csrf = csrf_token.to_owned();
                        }
                    }
                }
                response.text().map_err(|_| Errors::NoneError)
            }
            Method::GET => self
                .client
                .get(&url)
                .send()
                .map_err(|_| Errors::NoneError)?
                .text()
                .map_err(|_| Errors::NoneError),
        }
    }

    // 搜索
    // keywords: 关键词
    // types: 单曲(1)，歌手(100)，专辑(10)，歌单(1000)，用户(1002) *(type)*
    // offset: 起始点
    // limit: 数量
    #[allow(unused)]
    pub fn search(
        &mut self,
        keywords: String,
        types: u32,
        offset: u16,
        limit: u16,
    ) -> NCMResult<String> {
        let path = "/weapi/search/get";
        let mut params = HashMap::new();
        let _types = types.to_string();
        let offset = offset.to_string();
        let limit = limit.to_string();
        params.insert("s", &keywords[..]);
        params.insert("type", &_types[..]);
        params.insert("offset", &offset[..]);
        params.insert("limit", &limit[..]);
        let result = self.request(Method::POST, path, params, CryptoApi::Weapi, "")?;
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
        // let id = music_id.to_string();
        // params.insert("id", &id[..]);
        params.insert("id", music_id.as_str());
        params.insert("lv", "-1");
        params.insert("tv", "-1");
        params.insert("csrf_token", &csrf_token);
        let result = self.request(Method::POST, path, params, CryptoApi::Weapi, "")?;
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
        let result = self.request(Method::POST, path, params, CryptoApi::Weapi, "")?;
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
        let result = self.request(Method::POST, path, params, CryptoApi::Weapi, "")?;
        to_song_info(result, Parse::USL)
    }
}

fn choose_user_agent(ua: &str) -> &str {
    let index = if ua == "mobile" {
        rand::random::<usize>() % 7
    } else if ua == "pc" {
        rand::random::<usize>() % 5 + 8
    } else if !ua.is_empty() {
        return ua;
    } else {
        rand::random::<usize>() % USER_AGENT_LIST.len()
    };
    USER_AGENT_LIST[index]
}
