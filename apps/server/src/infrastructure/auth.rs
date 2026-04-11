// Auth module - JWT bootstrap
use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // household_id
    pub exp: i64,
}

pub fn create_token(household_id: Uuid, secret: &str) -> Result<String> {
    let expiry = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(365))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: household_id.to_string(),
        exp: expiry,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
