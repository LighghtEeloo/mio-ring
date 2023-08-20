use aes_gcm::{Aes256Gcm, aead::OsRng, KeyInit};
use std::{fs, path::PathBuf};

fn main() {
    let key = Aes256Gcm::generate_key(OsRng);
    let gen_path = PathBuf::from("_gen");
    let key_path = gen_path.join("key");
    let need_to_gen = !key_path.exists();
    if need_to_gen {
        fs::create_dir_all(gen_path.as_path()).unwrap();
        fs::write(key_path.as_path(), key).unwrap();
    }
}
