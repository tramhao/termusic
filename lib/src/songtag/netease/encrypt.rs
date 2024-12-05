/**
 * encrypt.rs
 * Copyright (C) 2019 gmg137 <gmg137@live.com>
 * Distributed under terms of the GPLv3 license.
 */
use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use libaes::Cipher;
use md5::compute;
use num_bigint::BigUint;
use rand::{rngs::OsRng, RngCore};
use std::convert::TryFrom;

// below imports are left for debug
// use std::io::Write;
// use openssl::rsa;

const IV: &[u8] = b"0102030405060708";
const PRESET_KEY: &[u8] = b"0CoJUm6Qyw8W8jud";
const BASE62: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
// const RSA_PUBLIC_KEY: &[u8] = b"-----BEGIN PUBLIC KEY-----\nMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgtQn2JZ34ZC28NWYpAUd98iZ37BUrX/aKzmFbt7clFSs6sXqHauqKWqdtLkF2KexO40H1YTX8z2lSgBBOAxLsvaklV8k4cBFK9snQXE9/DDaFt6Rr7iVZMldczhC0JNgTz+SHXT6CBHuX3e9SdB1Ua44oncaTWz7OBGLbCiK45wIDAQAB\n-----END PUBLIC KEY-----";

const MODULUS: &[u8] = b"e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7";
// static MODULUS: &[u8] =
// b"00e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7"
// ;
const PUBKEY: &[u8] = b"010001";

pub struct Crypto;

impl Crypto {
    pub fn weapi(text: &str) -> Result<String> {
        let mut secret_key = [0_u8; 16];
        OsRng.fill_bytes(&mut secret_key);
        let key: Vec<u8> = secret_key
            .iter()
            .map(|i| BASE62[(i % 62) as usize])
            .collect();

        let params1 = Self::aes_encrypt(text, PRESET_KEY, Some(IV))?;

        let params = Self::aes_encrypt(&params1, &key, Some(IV))?;

        let key_string = key.iter().map(|&c| c as char).collect::<String>();
        let enc_sec_key = Self::rsa(&key_string);

        // code here is left for debug, as probably the public key could be banned from netease
        // server, thus we should replace the exp and modulus.

        // if let Ok(key) = rsa::Rsa::public_key_from_pem(RSA_PUBLIC_KEY) {
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

        Ok(format!("params={p_value}&encSecKey={enc_value}&"))
    }

    pub fn aes_encrypt(data: &str, key: &[u8], iv: Option<&[u8]>) -> Result<String> {
        let mut iv_real: Vec<u8> = vec![0_u8; 16];
        if let Some(i) = iv {
            iv_real = i.to_vec();
        }
        // Create a new 128-bit cipher
        let key_16 = <&[u8; 16]>::try_from(key)?;
        let cipher = Cipher::new_128(key_16);

        // Encryption
        let encrypted = cipher.cbc_encrypt(&iv_real, data.as_bytes());

        Ok(general_purpose::URL_SAFE.encode(encrypted))
        // encode(encrypted)
    }

    fn rsa(text: &str) -> String {
        let text = text.chars().rev().collect::<String>();
        let text = BigUint::parse_bytes(hex::encode(text).as_bytes(), 16).unwrap();
        let pubkey = BigUint::parse_bytes(PUBKEY, 16).unwrap();
        let modulus = BigUint::parse_bytes(MODULUS, 16).unwrap();
        let pow = text.modpow(&pubkey, &modulus);
        pow.to_str_radix(16)
    }

    // This is for getting url of picture.
    pub fn encrypt_id(id: &str) -> String {
        let magic = b"3go8&$8*3*3h0k(2)2";
        let magic_len = magic.len();
        let mut song_id = id.to_string().into_bytes();
        id.as_bytes().iter().enumerate().for_each(|(i, sid)| {
            song_id[i] = *sid ^ magic[i % magic_len];
        });

        general_purpose::URL_SAFE
            .encode(compute(&song_id).as_ref())
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
