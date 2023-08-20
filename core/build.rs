use ring::rand::{SecureRandom, SystemRandom};
use std::{fs, path::PathBuf};

fn main() {
    let rng = SystemRandom::new();
    let mut buf = [0u8; 10];
    rng.fill(&mut buf).unwrap();
    let key_path = PathBuf::from("_gen/key");
    let need_to_gen = !key_path.exists();
    if need_to_gen {
        fs::create_dir_all("_gen").unwrap();
        fs::write(key_path.as_path(), buf).unwrap();
    }
}
