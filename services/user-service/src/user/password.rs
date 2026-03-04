use anyhow::Result;
use argon2::Argon2;
use password_hash::{PasswordHash, PasswordVerifier, PasswordHasher, SaltString};
use rand_core::OsRng;

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|error| anyhow::anyhow!("failed to parse password hash: {error:#}"))?;

    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}



