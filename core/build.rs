use ring::rand::{SecureRandom, SystemRandom};

fn main() {
    let rng = SystemRandom::new();
    let mut buf = [0u8; 10];
    rng.fill(&mut buf).unwrap();
    std::fs::create_dir_all("_gen").unwrap();
    std::fs::write("_gen/key", buf).unwrap();
}
