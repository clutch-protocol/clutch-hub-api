use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::hub::{configuration::AppConfig, signature_keys::SignatureKeys};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub pk: String,      // public key
    pub exp: usize,      // expiration time
}

pub fn generate_jwt_token(public_key: &str, config: &AppConfig) -> Result<String, String> {
    // Validate the public key first
    SignatureKeys::validate_public_key(public_key)?;

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + (config.jwt_expiration_hours * 3600) as usize;

    let claims = Claims {
        pk: public_key.to_string(),
        exp: expiration,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes())
    ).map_err(|e| e.to_string())
}