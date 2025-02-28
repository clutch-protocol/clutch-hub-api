use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::hub::configuration::AppConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub pk: String,      // public key
    pub exp: usize,      // expiration time
}

pub fn generate_jwt_token(public_key: &str, config: &AppConfig) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + (24 * 3600); // Token expires in 24 hours

    let claims = Claims {
        pk: public_key.to_string(),
        exp: expiration,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes())
    )
}