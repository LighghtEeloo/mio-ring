use aes_gcm::{Aes256Gcm, aead::OsRng, KeyInit, AeadCore};
use std::{fs, path::PathBuf};

fn main() {
    let key = Aes256Gcm::generate_key(OsRng);
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let gen_path = PathBuf::from("_gen");
    let key_path = gen_path.join("key");
    let nonce_path = gen_path.join("nonce");
    fs::create_dir_all(gen_path.as_path()).unwrap();
    if !key_path.exists() {
        fs::write(key_path.as_path(), key).unwrap();
    }
    if !nonce_path.exists() {
        fs::write(nonce_path.as_path(), nonce).unwrap();
    }
}
