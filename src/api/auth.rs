use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    async_trait,
};

// Placeholder auth user struct
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
}

// Simple placeholder authentication (no JWT for now)
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // For now, just accept any request as an authenticated admin user
        // TODO: Implement proper JWT validation
        Ok(AuthUser {
            user_id: "admin".to_string(),
            username: "admin".to_string(),
        })
    }
}
