use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, Key, KeyInit};
use pbkdf2::{
    password_hash::{rand_core::OsRng, SaltString},
    pbkdf2_hmac,
};
// use aes_gcm::
use sha2::Sha256;

fn derive_key(password: &[u8], salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    // iterations
    let n = 500_000;
    pbkdf2_hmac::<Sha256>(password, salt, n, &mut key);
    key
}

pub fn encrypt(data: &[u8], password: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut res = vec![];

    // generate salt
    let salt_string = SaltString::generate(&mut OsRng);
    let mut buffer = vec![0u8; salt_string.len()];
    let salt = salt_string.as_salt().decode_b64(&mut buffer).unwrap();

    let key = derive_key(password, salt);

    let key: &Key<Aes256Gcm> = &key.into();

    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let Ok(ciphertext) = cipher.encrypt(&nonce, data) else {
        return Err("encryption failed".into());
    };

    res.extend_from_slice(salt);
    res.extend_from_slice(&nonce);
    res.extend_from_slice(&ciphertext);

    Ok(res)
}

pub fn decrypt(data: &[u8], password: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let salt = &data[..16];
    let nonce = &data[16..28];

    let key = derive_key(password, salt);

    let key: &Key<Aes256Gcm> = &key.into();

    let cipher = Aes256Gcm::new(key);
    let Ok(decrypted) = cipher.decrypt(nonce.into(), &data[28..]) else {
        return Err("decryption failed".into());
    };

    Ok(decrypted)
}

/// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt() {
        let data = b"hello world";
        let password = b"password";
        let encrypted = encrypt(data, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }
}
