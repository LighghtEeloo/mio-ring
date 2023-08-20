use super::*;

pub(crate) struct Cipher;
impl Cipher {
    pub(crate) fn main_key() -> Key<Aes256Gcm> {
        let key_file = include_crypt::include_crypt!(AES, "_gen/key").decrypt();
        let key = Key::<Aes256Gcm>::from_slice(key_file.as_slice());
        *key
    }
    pub(crate) fn index_nonce() -> Nonce<Aes256Gcm> {
        let nonce_file = include_crypt::include_crypt!(AES, "_gen/nonce").decrypt();
        let nonce = Nonce::<Aes256Gcm>::from_slice(nonce_file.as_slice());
        *nonce
    }
    pub(crate) fn decrypt(
        ciphertext: impl AsRef<[u8]>,
        nonce: &Nonce<Aes256Gcm>,
    ) -> anyhow::Result<Vec<u8>> {
        let key = Self::main_key();
        let cipher = Aes256Gcm::new(&key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("decryption failed"))?;
        Ok(plaintext)
    }
    pub(crate) fn encrypt(
        plaintext: impl AsRef<[u8]>,
        nonce: &Nonce<Aes256Gcm>,
    ) -> anyhow::Result<Vec<u8>> {
        let key = Self::main_key();
        let cipher = Aes256Gcm::new(&key);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|_| anyhow::anyhow!("encryption failed"))?;
        Ok(ciphertext)
    }
}
