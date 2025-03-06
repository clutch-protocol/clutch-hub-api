use async_graphql::{SimpleObject, Guard, Context, Result, Error};
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct RideRequest {
    pub pickup_location: String,
    pub dropoff_location: String,
}

#[derive(SimpleObject)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: usize,
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub public_key: String,
    // Add additional user fields as needed (name, email, etc.)
}

/// Authentication guard for GraphQL operations
pub struct AuthGuard;

impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        // Try to get the authenticated user from the context
        if ctx.data::<AuthUser>().is_ok() {
            // User is authenticated
            Ok(())
        } else {
            // User is not authenticated
            Err(Error::new("Unauthorized: Authentication required"))
        }
    }
}

/// Helper function to get authenticated user from context
pub fn get_auth_user<'a>(ctx: &'a Context<'_>) -> Option<&'a AuthUser> {
    ctx.data::<AuthUser>().ok()
}
