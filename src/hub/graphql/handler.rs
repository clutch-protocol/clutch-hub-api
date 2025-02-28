use actix_web::{web, HttpRequest};
use async_graphql::{Schema, EmptySubscription};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use crate::hub::graphql::{Query, Mutation};
use crate::hub::graphql::types::AuthUser;
use crate::hub::configuration::AppConfig;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use tracing::error;

// Define JWT claim structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub pk: String,      // public key
    pub exp: usize,      // expiration time
}

// Extract JWT token from Authorization header and validate it
fn extract_auth_user(req: &HttpRequest, config: &AppConfig) -> Option<AuthUser> {
    // Get the Authorization header
    let auth_header = req.headers().get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    
    // Check if it's a Bearer token
    if !auth_str.starts_with("Bearer ") {
        return None;
    }
    
    let token = &auth_str["Bearer ".len()..];
    
    // Use JWT secret from configuration
    let secret = config.jwt_secret.as_bytes();
    
    // Validate the JWT token and extract claims
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256)
    ) {
        Ok(token_data) => {
            // Extract user data from validated claims
            let claims = token_data.claims;
            
            // Create and return AuthUser from JWT claims
            Some(AuthUser {
                public_key: claims.pk.clone(),
            })
        },
        Err(err) => {
            // Log the error and return None
            error!("JWT validation failed: {}", err);
            None
        }
    }
}

pub async fn graphql_handler(
    schema: web::Data<Schema<Query, Mutation, EmptySubscription>>,
    config: web::Data<AppConfig>,
    req: GraphQLRequest,
    http_req: HttpRequest,
) -> GraphQLResponse {
    // Extract auth user from JWT token
    let auth_user = extract_auth_user(&http_req, &config);
    
    // Build GraphQL request with auth data
    let mut request = req.into_inner();
    if let Some(user) = auth_user {
        request = request.data(user);
    }
    
    schema.execute(request).await.into()
}
