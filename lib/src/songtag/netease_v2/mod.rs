use std::{collections::HashMap, time::Duration};

use anyhow::anyhow;
use base64::{engine::general_purpose, Engine};
use bytes::Buf as _;
use libaes::Cipher;
use lofty::picture::Picture;
use model::{to_lyric, to_song_info, to_song_url};
use num_bigint::BigUint;
use rand::{rngs::OsRng, RngCore};
use reqwest::{Client, ClientBuilder, RequestBuilder};

use crate::songtag::ServiceProvider;

use super::service::{SongTagService, SongTagServiceError};

mod model;

const URL_SEARCH_NETEASE: &str = "https://music.163.com/weapi/search/get";
const URL_LYRIC_NETEASE: &str = "https://music.163.com/weapi/song/lyric";
const URL_DOWNLOAD_NETEASE: &str = "https://music.163.com/weapi/song/enhance/player/url/v1";
const URL_PICTURE_SERVICE: &str = "https://p3.music.126.net/";

const REFERER: &str = "https://music.163.com";
const HOST: &str = "music.163.com";

pub struct Api {
    client: Client,
}

impl Api {
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client.");

        Self { client }
    }

    fn common_request(builder: RequestBuilder, params: &[(&str, &str)]) -> RequestBuilder {
        let body = {
            // using hashmap as the API expects a object, a Vec would create a array
            let params: HashMap<&str, &str> = params.iter().map(|v| (v.0, v.1)).collect();
            // there does not seem to be any CSRF-Token need to be set from the responses
            // params.insert("csrf_token", "");

            let text = serde_json::to_string(&params).unwrap();

            Self::encrypt_for_weapi(&text)
        };

        builder
            .header("Cookie", "os=pc; appver=2.7.1.198277")
            .header("Accept", "*/*")
            .header("Accept-Encoding", "gzip,deflate")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Connection", "keep-alive")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Host", HOST)
            .header("Referer", REFERER)
            .body(body)
    }

    /// Encode the picture id with some magic
    fn encode_pic_id(id: &str) -> String {
        const MAGIC: &[u8] = b"3go8&$8*3*3h0k(2)2";

        let magic_id: Vec<u8> = id
            .as_bytes()
            .iter()
            .enumerate()
            .map(|(idx, sid)| *sid ^ MAGIC[idx % MAGIC.len()])
            .collect();

        general_purpose::URL_SAFE
            .encode(md5::compute(&magic_id).as_ref())
            .replace('/', "_")
            .replace('+', "-")
    }

    /// Encrypt the given `text` for netease.
    /// Returns the text encrypted in url-encoding.
    fn encrypt_for_weapi(text: &str) -> String {
        const BASE62_LIKE: &[u8] =
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        // truncation is expected and within limits of u8
        #[allow(clippy::cast_possible_truncation)]
        const BASE62_LIKE_LEN: u8 = BASE62_LIKE.len() as u8;
        const IV: &[u8] = b"0102030405060708";
        const PRESET_KEY: &[u8; 16] = b"0CoJUm6Qyw8W8jud";

        let mut random_bytes = [0u8; 16];
        OsRng.fill_bytes(&mut random_bytes);

        let mut key = [0u8; 16];
        random_bytes.iter().enumerate().for_each(|(idx, value)| {
            key[idx] = BASE62_LIKE[(value % BASE62_LIKE_LEN) as usize];
        });

        let round1 = Self::aes_encrypt(text.as_bytes(), PRESET_KEY, IV);
        let round2 = Self::aes_encrypt(round1.as_bytes(), &key, IV);

        let key_string: String = key.iter().map(|v| char::from(*v)).collect();
        let enc_sec_key = Self::rsa_encrypt(&key_string);

        let escaped_round2 = urlencoding::encode(&round2);
        let escaped_enc_sec_key = urlencoding::encode(&enc_sec_key);

        format!("params={escaped_round2}&encSecKey={escaped_enc_sec_key}")
    }

    /// Run the aes encryption with the given data. Also encode result in base64
    fn aes_encrypt(data: &[u8], key: &[u8; 16], iv: &[u8]) -> String {
        let cipher = Cipher::new_128(key);

        let encrypted = cipher.cbc_encrypt(iv, data);

        general_purpose::URL_SAFE.encode(encrypted)
    }

    /// Encrypt `text` with the RSA public key from netease
    fn rsa_encrypt(text: &str) -> String {
        const MODULUS: &[u8] = b"e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7";
        const PUBKEY: &[u8] = b"010001";

        let rev: String = text.chars().rev().collect();

        let pubkey = BigUint::parse_bytes(PUBKEY, 16).unwrap();
        let modulus = BigUint::parse_bytes(MODULUS, 16).unwrap();

        let as_bigint = BigUint::parse_bytes(hex::encode(rev).as_bytes(), 16).unwrap();

        let pow = as_bigint.modpow(&pubkey, &modulus);

        pow.to_str_radix(16)
    }
}

impl SongTagService for Api {
    type Error = anyhow::Error;

    fn display_name() -> &'static str
    where
        Self: Sized,
    {
        "netease"
    }

    async fn search_recording(
        &self,
        keywords: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<super::SongTag>, super::service::SongTagServiceError<Self::Error>> {
        let offset = offset.to_string();
        let limit = limit.to_string();

        let query_params = &[
            ("s", keywords),
            // types: single (1), singer (100), album (10), playlist (1000), user (1002) *(type)*
            ("type", "1"),
            ("offset", &offset),
            ("limit", &limit),
        ];

        let response = Self::common_request(self.client.post(URL_SEARCH_NETEASE), query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        let result = response.text().await.map_err(anyhow::Error::from)?;

        to_song_info(&result).map_err(|err| {
            SongTagServiceError::Other(err.context("convert result to songtag array"))
        })
    }

    async fn get_lyrics(
        &self,
        song: &super::SongTag,
    ) -> Result<String, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Netease {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        let Some(lyric_id) = &song.lyric_id else {
            return Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a lyric_id!"
            )));
        };

        let query_params = &[
            ("id", lyric_id.as_str()),
            // what do they mean?
            ("lv", "-1"),
            ("tv", "-1"),
        ];

        let response = Self::common_request(self.client.post(URL_LYRIC_NETEASE), query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        let result = response.text().await.map_err(anyhow::Error::from)?;

        to_lyric(&result)
            .map_err(|err| SongTagServiceError::Other(err.context("get lyric text from response")))
    }

    async fn get_picture(
        &self,
        song: &super::SongTag,
    ) -> Result<lofty::picture::Picture, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Netease {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        let Some(pic_id) = &song.pic_id else {
            return Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a pic_id!"
            )));
        };

        let id_encrypted = Self::encode_pic_id(pic_id);
        let pic_url = format!("{URL_PICTURE_SERVICE}{id_encrypted}/{pic_id}.jpg?param=300y300");

        let result = self
            .client
            .get(pic_url)
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        let mut reader = result.bytes().await.map_err(anyhow::Error::from)?.reader();
        let picture = Picture::from_reader(&mut reader).map_err(anyhow::Error::from)?;

        Ok(picture)
    }

    async fn download_recording(
        &self,
        song: &super::SongTag,
    ) -> Result<String, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Netease {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        // netease wants the "ids" as a string array, but we only have 1
        let ids = serde_json::to_string(&[&song.song_id]).unwrap();

        let query_params = &[
            ("ids", ids.as_str()),
            // what does this control?
            ("level", "standard"),
            ("encodeType", "aac"),
        ];

        let response = Self::common_request(self.client.post(URL_DOWNLOAD_NETEASE), query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        let result = response.text().await.map_err(anyhow::Error::from)?;

        to_song_url(&result).map_err(|err| {
            SongTagServiceError::Other(err.context("get download url from response"))
        })
    }
}
