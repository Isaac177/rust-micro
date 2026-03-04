use anyhow::Result;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub display_name: String,
    pub token_type: TokenType,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

const ACCESS_TOKEN_DURATION_SECS: i64 = 15 * 60;
const REFRESH_TOKEN_DURATION_SECS: i64 = 7 * 24 * 60 * 60;

pub fn mint_token_pair(
    user_id: &str,
    email: &str,
    display_name: &str,
    secret: &str,
    issuer: &str,
) -> Result<TokenPair> {
    let access_token = mint_token(user_id, email, display_name, TokenType::Access, secret, issuer, ACCESS_TOKEN_DURATION_SECS)?;
    let refresh_token = mint_token(user_id, email, display_name, TokenType::Refresh, secret, issuer, REFRESH_TOKEN_DURATION_SECS)?;
    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

fn mint_token(
    user_id: &str,
    email: &str,
    display_name: &str,
    token_type: TokenType,
    secret: &str,
    issuer: &str,
    duration_secs: i64,
) -> Result<String> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        display_name: display_name.to_string(),
        token_type,
        iss: issuer.to_string(),
        iat: now,
        exp: now + duration_secs,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn validate_token(token: &str, secret: &str, issuer: &str) -> Result<Claims> {
    let mut validation = Validation::default();
    validation.set_issuer(&[issuer]);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok(token_data.claims)
}
