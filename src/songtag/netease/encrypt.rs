//
// encrypt.rs
// Copyright (C) 2019 gmg137 <gmg137@live.com>
// Distributed under terms of the GPLv3 license.
//
use lazy_static::lazy_static;
use openssl::hash::{hash, DigestBytes, MessageDigest};
use openssl::rsa::{Padding, Rsa};
use openssl::symm::{encrypt, Cipher};
use rand::rngs::OsRng;
use rand::Rng;
use rand::RngCore;
use urlqstring::QueryParams;
use AesMode::{cbc, ecb};

lazy_static! {
    static ref IV: Vec<u8> = "0102030405060708".as_bytes().to_vec();
    static ref PRESET_KEY: Vec<u8> = "0CoJUm6Qyw8W8jud".as_bytes().to_vec();
    static ref LINUX_API_KEY: Vec<u8> = "rFgB&h#%2?^eDg:Q".as_bytes().to_vec();
    static ref BASE62: Vec<u8> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes().to_vec();
    static ref RSA_PUBLIC_KEY: Vec<u8> = "-----BEGIN PUBLIC KEY-----\nMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgtQn2JZ34ZC28NWYpAUd98iZ37BUrX/aKzmFbt7clFSs6sXqHauqKWqdtLkF2KexO40H1YTX8z2lSgBBOAxLsvaklV8k4cBFK9snQXE9/DDaFt6Rr7iVZMldczhC0JNgTz+SHXT6CBHuX3e9SdB1Ua44oncaTWz7OBGLbCiK45wIDAQAB\n-----END PUBLIC KEY-----".as_bytes().to_vec();
    static ref EAPIKEY: Vec<u8> = "e82ckenh8dichen8".as_bytes().to_vec();
}

#[allow(non_snake_case)]
pub struct Crypto;

#[allow(dead_code, non_camel_case_types)]
pub enum HashType {
    md5,
}

#[allow(non_camel_case_types)]
pub enum AesMode {
    cbc,
    ecb,
}

#[allow(dead_code, clippy::redundant_closure)]
impl Crypto {
    pub fn hex_random_bytes(n: usize) -> String {
        let mut data: Vec<u8> = Vec::with_capacity(n);
        OsRng.fill_bytes(&mut data);
        hex::encode(data)
    }

    pub fn alpha_lowercase_random_bytes(n: usize) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        let rand_string: String = (0..n)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        rand_string
    }

    pub fn eapi(url: &str, text: &str) -> String {
        let message = format!("nobody{}use{}md5forencrypt", url, text);
        let mut digest = String::new();
        if let Ok(hash) = hash(MessageDigest::md5(), message.as_bytes()) {
            digest = hex::encode(hash);
        }
        let data = format!("{}-36cd479b6b5-{}-36cd479b6b5-{}", url, text, digest);
        let params = Crypto::aes_encrypt(&data, &*EAPIKEY, ecb, Some(&*IV), |t: &Vec<u8>| {
            hex::encode_upper(t)
        });
        QueryParams::from(vec![("params", params.as_str())]).stringify()
    }

    pub fn weapi(text: &str) -> String {
        let mut secret_key = [0_u8; 16];
        OsRng.fill_bytes(&mut secret_key);
        let key: Vec<u8> = secret_key
            .iter()
            .map(|i| BASE62[(i % 62) as usize])
            .collect();

        let params1 = Crypto::aes_encrypt(text, &*PRESET_KEY, cbc, Some(&*IV), |t: &Vec<u8>| {
            base64::encode(t)
        });

        let params = Crypto::aes_encrypt(&params1, &key, cbc, Some(&*IV), |t: &Vec<u8>| {
            base64::encode(t)
        });

        let mut enc_sec_key = String::new();

        if let Ok(key_vec) = std::str::from_utf8(&key.iter().rev().copied().collect::<Vec<u8>>()) {
            enc_sec_key = Crypto::rsa_encrypt(key_vec, &*RSA_PUBLIC_KEY)
        };

        // let enc_sec_key = Crypto::rsa_encrypt(
        //     std::str::from_utf8(&key.iter().rev().copied().collect::<Vec<u8>>()),
        //     &*RSA_PUBLIC_KEY,
        // );

        QueryParams::from(vec![
            ("params", params.as_str()),
            ("encSecKey", enc_sec_key.as_str()),
        ])
        .stringify()
    }

    pub fn linuxapi(text: &str) -> String {
        let params = Crypto::aes_encrypt(text, &*LINUX_API_KEY, ecb, None, |t: &Vec<u8>| {
            hex::encode(t)
        })
        .to_uppercase();
        QueryParams::from(vec![("eparams", params.as_str())]).stringify()
    }

    pub fn aes_encrypt(
        data: &str,
        key: &[u8],
        mode: AesMode,
        iv: Option<&[u8]>,
        encode: fn(&Vec<u8>) -> String,
    ) -> String {
        let cipher = match mode {
            cbc => Cipher::aes_128_cbc(),
            ecb => Cipher::aes_128_ecb(),
        };

        let mut cipher_text: Vec<u8> = Vec::new();
        if let Ok(c) = encrypt(cipher, key, iv, data.as_bytes()) {
            cipher_text = c;
        };

        encode(&cipher_text)
    }

    pub fn rsa_encrypt(data: &str, key: &[u8]) -> String {
        match Rsa::public_key_from_pem(key) {
            Ok(rsa) => {
                let prefix = vec![0_u8; 128 - data.len()];

                let data = [&prefix[..], data.as_bytes()].concat();

                let mut buf = vec![0; rsa.size() as usize];

                let _ = rsa.public_encrypt(&data, &mut buf, Padding::NONE);

                hex::encode(buf)
            }
            Err(_) => "".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn hash_encrypt(
        data: &str,
        algorithm: HashType,
        encode: fn(DigestBytes) -> String,
    ) -> String {
        match algorithm {
            HashType::md5 => match hash(MessageDigest::md5(), data.as_bytes()) {
                Ok(result) => encode(result),
                Err(_) => "".to_string(),
            },
        }
    }

    // This is for getting url of picture.
    pub fn encrypt_id(id: &str) -> String {
        let magic = b"3go8&$8*3*3h0k(2)2";
        let magic_len = magic.len();
        let id = id;
        let mut song_id = id.to_string().into_bytes();
        id.as_bytes().iter().enumerate().for_each(|(i, sid)| {
            song_id[i] = *sid ^ magic[i % magic_len];
        });
        match hash(MessageDigest::md5(), &song_id) {
            Ok(result) => base64::encode_config(result, base64::URL_SAFE)
                .replace("/", "_")
                .replace("+", "-"),
            Err(_) => "".to_string(),
        }
    }
}
