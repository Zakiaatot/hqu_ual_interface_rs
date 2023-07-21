use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};

use random_string::generate;

type Aes128Cbc = cbc::Encryptor<aes::Aes128>;
pub fn encoded_password(data: String, key: String) -> Result<String> {
    let data = gen_random_string(64) + &data;
    let mut buf = data.as_bytes().to_owned();
    let iv = gen_random_string(16);
    let encryptor = {
        match Aes128Cbc::new_from_slices(key.as_bytes(), iv.as_bytes()) {
            Ok(res) => res,
            Err(_) => return Err(anyhow!("Encryptor error!")),
        }
    };
    let res = general_purpose::STANDARD.encode(encryptor.encrypt_padded_vec_mut::<Pkcs7>(&mut buf));
    Ok(res)
}
const CHARSET: &str = "ABCDEFGHJKMNPQRSTWXYZabcdefhijkmnprstwxyz2345678";
fn gen_random_string(len: usize) -> String {
    generate(len, CHARSET)
}

#[test]
fn crypto_test() {
    let key = "2C6rKsudIrhANGbU".to_string();
    let data = "abcdefg".to_string();
    println!("{}", encoded_password(data, key).unwrap());
}
