// Music API for netease
use super::{SongTag, TagNetease};
use anyhow::{anyhow, Result};
use openssl::rsa::{Padding, Rsa};
use openssl::symm::{encrypt, Cipher};
use rand::rngs::OsRng;
use rand::RngCore;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json;
use std::collections::HashMap;

use base64;
use lazy_static::lazy_static;
// use openssl::hash::{hash, DigestBytes, MessageDigest};
// use urlqstring::QueryParams;

lazy_static! {
    static ref IV: Vec<u8> = "0102030405060708".as_bytes().to_vec();
    static ref PRESET_KEY: Vec<u8> = "0CoJUm6Qyw8W8jud".as_bytes().to_vec();
    static ref LINUX_API_KEY: Vec<u8> = "rFgB&h#%2?^eDg:Q".as_bytes().to_vec();
    static ref BASE62: Vec<u8> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes().to_vec();
    static ref RSA_PUBLIC_KEY: Vec<u8> = "-----BEGIN PUBLIC KEY-----\nMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgtQn2JZ34ZC28NWYpAUd98iZ37BUrX/aKzmFbt7clFSs6sXqHauqKWqdtLkF2KexO40H1YTX8z2lSgBBOAxLsvaklV8k4cBFK9snQXE9/DDaFt6Rr7iVZMldczhC0JNgTz+SHXT6CBHuX3e9SdB1Ua44oncaTWz7OBGLbCiK45wIDAQAB\n-----END PUBLIC KEY-----".as_bytes().to_vec();
    static ref EAPIKEY: Vec<u8> = "e82ckenh8dichen8".as_bytes().to_vec();
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum AesMode {
    cbc,
    ecb,
}

pub struct NetEaseAPI {
    pub modulus: String,
    pub nonce: [u8; 16],
    pub pub_key: String,
    pub url: String,
    pub header: HeaderMap,
    pub sec_key: String,
}

impl NetEaseAPI {
    pub fn new() -> Self {
        let mut header = HeaderMap::new();
        header.insert("Accept", HeaderValue::from_static("*/*"));
        header.insert(
            "Accept-Encoding",
            HeaderValue::from_static("gzip,deflate,sdch"),
        );
        header.insert(
            "Accept-Language",
            HeaderValue::from_static("zh-CN,zh;q=0.8,gl;q=0.6,zh-TW;q=0.4"),
        );
        header.insert("Connection", HeaderValue::from_static("keep-alive"));
        header.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        header.insert("Host", HeaderValue::from_static("music.163.com"));
        header.insert(
            "Referer",
            HeaderValue::from_static("https://music.163.com/search/"),
        );
        header.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_9_2) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.152 Safari/537.36"));

        let sec_key = NetEaseAPI::hex_random_bytes(16);

        Self{
            modulus: "00e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7".to_string(),
            nonce: b"0CoJUm6Qyw8W8jud".to_owned(),
            pub_key: "010001".to_string(),
            url: "https://music.163.com/weapi/cloudsearch/get/web?csrf_token=".to_string(),
            header,
            sec_key,
        }
    }

    pub fn hex_random_bytes(n: usize) -> String {
        let mut data: Vec<u8> = Vec::with_capacity(n);
        OsRng.fill_bytes(&mut data);
        hex::encode(data)
    }

    pub fn aes_encrypt(
        data: &str,
        key: &Vec<u8>,
        mode: AesMode,
        iv: Option<&[u8]>,
        encode: fn(&Vec<u8>) -> String,
    ) -> String {
        let cipher = match mode {
            AesMode::cbc => Cipher::aes_128_cbc(),
            AesMode::ecb => Cipher::aes_128_ecb(),
        };
        let cipher_text = encrypt(cipher, key, iv, data.as_bytes()).unwrap();

        encode(&cipher_text)
    }

    pub fn rsa_encrypt(data: &str, key: &Vec<u8>) -> String {
        let rsa = Rsa::public_key_from_pem(key).unwrap();

        let prefix = vec![0u8; 128 - data.len()];

        let data = [&prefix[..], &data.as_bytes()[..]].concat();

        let mut buf = vec![0; rsa.size() as usize];

        rsa.public_encrypt(&data, &mut buf, Padding::NONE).unwrap();

        hex::encode(buf)
    }

    pub fn search(&self, s: &str) -> Result<Vec<SongTag>> {
        let mut map = HashMap::new();
        map.insert("hlpretag", "<span class=\"s-fc7\">");
        map.insert("hlposttag", "</span>");
        map.insert("#/discover", "");
        map.insert("s", s);
        map.insert("type", "1");
        map.insert("offset", "0");
        map.insert("total", "true");
        map.insert("limit", "2");
        map.insert("csrf_token", "");

        let serialized = serde_json::to_string(&map).unwrap();
        // println!("text={}", serialized);

        let mut secret_key = [0u8; 16];
        OsRng.fill_bytes(&mut secret_key);
        let key: Vec<u8> = secret_key
            .iter()
            .map(|i| BASE62[(i % 62) as usize])
            .collect();

        println!("key={}", String::from_utf8(key.clone()).unwrap());

        let params1 = NetEaseAPI::aes_encrypt(
            serialized.as_str(),
            &*PRESET_KEY,
            AesMode::cbc,
            Some(&*IV),
            |t: &Vec<u8>| base64::encode(t),
        );

        let params =
            NetEaseAPI::aes_encrypt(&params1, &key, AesMode::cbc, Some(&*IV), |t: &Vec<u8>| {
                base64::encode(t)
            });

        let enc_sec_key = NetEaseAPI::rsa_encrypt(
            std::str::from_utf8(&key.iter().rev().map(|n| *n).collect::<Vec<u8>>()).unwrap(),
            &*RSA_PUBLIC_KEY,
        );

        // let mut body = HashMap::new();
        // body.insert("params", params.as_str());
        // body.insert("encSecKey", enc_sec_key.as_str());

        // let enc = QueryParams::from(vec![
        //     ("params", params.as_str()),
        //     ("encSecKey", enc_sec_key.as_str()),
        // ])
        // .stringify();

        let client = reqwest::blocking::Client::builder()
            .default_headers(self.header.clone())
            .build()?;

        // let resp = client.post(self.url.clone()).json(&body).send()?;
        let resp = client
            .post(self.url.clone())
            .query(&[
                ("params", params.as_str()),
                ("encSecKey", enc_sec_key.as_str()),
            ])
            // .json(&body)
            .send()?;

        // println!("{:#?}", resp);
        // let resp = client
        //     .post(self.url.clone())
        //     .body(&body)
        //     .send()?;

        if resp.status() != 200 {
            return Err(anyhow!("Network error?"));
        }

        // println!("{}", self.sec_key);
        // println!("{}", String::from(resp.text()?.as_str()));

        // println!("{:?}", resp.bytes());
        // let tag_netease: Vec<TagNetease> = resp.json::<Vec<TagNetease>>()?;
        // let result = resp.json::<HashMap<String, String>>()?;

        println!("{:?}", resp);
        let mut result_tags: Vec<SongTag> = vec![];
        // for v in tag_netease.iter() {
        //     let song_tag: SongTag = SongTag {
        //         artist: v.artist.clone(),
        //         title: Some(v.name.clone()),
        //         album: Some(v.album.clone()),
        //         lang_ext: Some(String::from("chi")),
        //         service_provider: Some(String::from("kugou")),
        //         song_id: Some(format!("{}", v.id)),
        //         lyric_id: Some(format!("{}", v.lyric_id)),
        //     };
        //     result_tags.push(song_tag);
        // }

        Ok(result_tags)
    }
}
