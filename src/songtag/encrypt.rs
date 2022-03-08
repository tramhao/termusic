/**
 * encrypt.rs
 * Copyright (C) 2019 gmg137 <gmg137@live.com>
 * Distributed under terms of the GPLv3 license.
 */

use lazy_static::lazy_static;
use libaes::Cipher;
use md5::compute;
use num_bigint::BigUint;
use rand::{rngs::OsRng, Rng, RngCore};
use std::convert::TryFrom;

// below imports are left for debug
// use std::io::Write;
// use openssl::rsa;

lazy_static! {
    static ref IV: Vec<u8> = b"0102030405060708".to_vec();
    static ref PRESET_KEY: Vec<u8> = b"0CoJUm6Qyw8W8jud".to_vec();
    static ref LINUX_API_KEY: Vec<u8> = b"rFgB&h#%2?^eDg:Q".to_vec();
    static ref BASE62: Vec<u8> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes().to_vec();
    static ref RSA_PUBLIC_KEY: Vec<u8> = b"-----BEGIN PUBLIC KEY-----\nMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgtQn2JZ34ZC28NWYpAUd98iZ37BUrX/aKzmFbt7clFSs6sXqHauqKWqdtLkF2KexO40H1YTX8z2lSgBBOAxLsvaklV8k4cBFK9snQXE9/DDaFt6Rr7iVZMldczhC0JNgTz+SHXT6CBHuX3e9SdB1Ua44oncaTWz7OBGLbCiK45wIDAQAB\n-----END PUBLIC KEY-----".to_vec();
    static ref EAPIKEY: Vec<u8> = b"e82ckenh8dichen8".to_vec();
}
static MODULUS:&str = "e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7";
// static MODULUS:&str =
// "00e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7"
// ;
static PUBKEY: &str = "010001";

#[allow(non_snake_case)]
pub struct Crypto;

#[allow(dead_code)]
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
        let hash = compute(message.as_bytes());
        let digest = hex::encode(hash.as_ref());

        let data = format!("{}-36cd479b6b5-{}-36cd479b6b5-{}", url, text, digest);
        let params = Self::aes_encrypt(&data, &*EAPIKEY, Some(&*IV), hex::encode_upper);

        let p_value = Self::escape(&params);
        let result = format!("params={}&", p_value);
        result
    }

    pub fn weapi(text: &str) -> String {
        let mut secret_key = [0_u8; 16];
        OsRng.fill_bytes(&mut secret_key);
        let key: Vec<u8> = secret_key
            .iter()
            .map(|i| BASE62[(i % 62) as usize])
            .collect();

        let params1 = Self::aes_encrypt(text, &*PRESET_KEY, Some(&*IV), base64::encode);

        let params = Self::aes_encrypt(&params1, &key, Some(&*IV), base64::encode);

        let key_string = key.iter().map(|&c| c as char).collect::<String>();
        let enc_sec_key = Self::rsa(&key_string);

        // code here is left for debug, as probably the public key could be banned from netease
        // server, thus we should replace the exp and modulus.

        // if let Ok(key) = rsa::Rsa::public_key_from_pem(&RSA_PUBLIC_KEY) {
        //     let modulus = key.n();
        //     let exp = key.e();
        //     let mut file = std::fs::File::create("data.txt").expect("create failed");
        //     let _drop = writeln!(
        //         &mut file,
        //         "modulus is:{} and exp is:{}",
        //         modulus.to_hex_str().unwrap().to_lowercase(),
        //         exp.to_hex_str().unwrap(),
        //     );
        // }

        let p_value = Self::escape(&params);
        let enc_value = Self::escape(&enc_sec_key);

        format!("params={}&encSecKey={}&", p_value, enc_value)
    }

    pub fn linuxapi(text: &str) -> String {
        let params = Self::aes_encrypt(text, &*LINUX_API_KEY, None, hex::encode).to_uppercase();
        let e_value = Self::escape(&params);
        format!("eparams={}&", e_value)
    }

    pub fn aes_encrypt(
        data: &str,
        key: &[u8],
        iv: Option<&[u8]>,
        encode: fn(Vec<u8>) -> String,
    ) -> String {
        let mut iv_real: Vec<u8> = vec![0_u8; 16];
        if let Some(i) = iv {
            iv_real = i.to_vec();
        }
        // Create a new 128-bit cipher
        let key_16 = <&[u8; 16]>::try_from(key).unwrap();
        let cipher = Cipher::new_128(key_16);

        // Encryption
        let encrypted = cipher.cbc_encrypt(&iv_real, data.as_bytes());

        encode(encrypted)
    }

    fn rsa(text: &str) -> String {
        let text = text.chars().rev().collect::<String>();
        let text = BigUint::parse_bytes(hex::encode(text).as_bytes(), 16).unwrap();
        let pubkey = BigUint::parse_bytes(PUBKEY.as_bytes(), 16).unwrap();
        let modulus = BigUint::parse_bytes(MODULUS.as_bytes(), 16).unwrap();
        let pow = text.modpow(&pubkey, &modulus);
        pow.to_str_radix(16)
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
        base64::encode_config(compute(&song_id).as_ref(), base64::URL_SAFE)
            .replace('/', "_")
            .replace('+', "-")
    }

    fn escape(str: &str) -> String {
        let mut enc = Vec::<u8>::new();
        for ch in str.as_bytes() {
            if Self::keep_as(*ch) {
                enc.push(*ch);
            } else {
                enc.push(0x25);
                let n1 = (*ch >> 4) & 0xf;
                let n2 = *ch & 0xf;
                enc.push(Self::to_dec_ascii(n1));
                enc.push(Self::to_dec_ascii(n2));
            }
        }
        String::from_utf8(enc).unwrap()
    }

    const fn keep_as(n: u8) -> bool {
        n.is_ascii_alphanumeric()
            || n == b'*'
            || n == b'-'
            || n == b'.'
            || n == b'_'
            || n == b'\''
            || n == b'~'
            || n == b'!'
            || n == b'('
            || n == b')'
    }

    const fn to_dec_ascii(n: u8) -> u8 {
        match n {
            0 => 48,
            1 => 49,
            2 => 50,
            3 => 51,
            4 => 52,
            5 => 53,
            6 => 54,
            7 => 55,
            8 => 56,
            9 => 57,
            10 => b'A',
            11 => b'B',
            12 => b'C',
            13 => b'D',
            14 => b'E',
            15 => b'F',
            _ => 127,
        }
    }
}
