use anyhow::{Context as _, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hmac::{Hmac, Mac};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

// ── Key hashing ───────────────────────────────────────────────────────────────

pub fn hash_api_key(key: &str, pepper: &str) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(pepper.as_bytes()).context("invalid pepper")?;
    mac.update(key.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn verify_api_key(key: &str, pepper: &str, stored_hash: &str) -> Result<bool> {
    let computed = hash_api_key(key, pepper)?;
    Ok(computed == stored_hash)
}

// ── Key generation ────────────────────────────────────────────────────────────

pub fn generate_api_key(env: &str) -> String {
    use rand::Rng as _;
    let random: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    format!("la_{env}_{random}")
}

// ── Password hashing ──────────────────────────────────────────────────────────

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("password hash failed: {e}"))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("invalid password hash: {e}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

// ── JWT ───────────────────────────────────────────────────────────────────────

const TOKEN_EXPIRY_SECS: u64 = 60 * 60 * 24 * 7; // 7 days

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub exp: u64,
}

pub fn issue_token(user_id: &str, email: &str, secret: &str) -> Result<String> {
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("system time error")?
        .as_secs()
        + TOKEN_EXPIRY_SECS;
    let claims = Claims {
        sub: user_id.to_owned(),
        email: email.to_owned(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("token encoding failed")
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .context("invalid token")?;
    Ok(data.claims)
}
