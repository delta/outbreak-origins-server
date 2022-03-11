use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::env;

pub fn decrypt_data(payload: &str) -> Result<String, String> {
    let enc_key = env::var("ENC_KEY").expect("ENC_KEY must be present");
    let mc = new_magic_crypt!(enc_key, 256);
    match mc.decrypt_base64_to_string(payload) {
        Ok(x) => Ok(x),
        Err(_) => Err("Couldn't decrypt data".to_string()),
    }
}
