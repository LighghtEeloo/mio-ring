use super::*;

pub(crate) fn main_key() -> Key<Aes256Gcm> {
    let key_file = include_crypt::include_crypt!(AES, "_gen/key").decrypt();
    let key = Key::<Aes256Gcm>::from_slice(key_file.as_slice());
    *key
}
