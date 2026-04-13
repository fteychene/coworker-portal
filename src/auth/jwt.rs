use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,         // user id
    pub username: String,
    pub exp: u64,
    pub iat: u64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_hours: u64,
}

impl JwtService {
    pub fn new(secret: &str, expiry_hours: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiry_hours,
        }
    }

    pub fn generate(&self, user_id: i32, username: &str) -> Result<String> {
        let now = jsonwebtoken::get_current_timestamp();
        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            iat: now,
            exp: now + self.expiry_hours * 3600,
        };
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn verify(&self, token: &str) -> Result<Claims> {
        let data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(data.claims)
    }
}
