use base64::engine::general_purpose::URL_SAFE;
use base64::Engine as _;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const SALT: &[u8] = b"EyeForge_salt_2026";
const SECRET: &[u8] = b"EyeForge_secret";
const ITERATIONS: u32 = 100_000;

fn derived_key() -> String {
    let mut key = [0_u8; 32];
    pbkdf2_hmac::<Sha256>(SECRET, SALT, ITERATIONS, &mut key);
    URL_SAFE.encode(key)
}

pub fn encrypt(plain: &str) -> String {
    if plain.is_empty() {
        return String::new();
    }

    let key = derived_key();
    let fernet = fernet::Fernet::new(&key).expect("invalid Fernet key");
    fernet.encrypt(plain.as_bytes())
}

pub fn decrypt(token: &str) -> String {
    if token.is_empty() {
        return String::new();
    }

    let key = derived_key();
    let fernet = fernet::Fernet::new(&key).expect("invalid Fernet key");

    match fernet.decrypt(token) {
        Ok(bytes) => String::from_utf8(bytes).unwrap_or_else(|_| token.to_string()),
        Err(_) => token.to_string(),
    }
}
