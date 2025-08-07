use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use anyhow::Result;

pub mod jwt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub roles: Vec<String>,
}

/// Authentication middleware for Axum
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract and validate token
    let token = jwt::extract_token_from_header(auth_header)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let claims = jwt::validate_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check if user has author role
    if !claims.roles.contains(&"author".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Add claims to request extensions for use in handlers
    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Extract claims from request extensions
pub fn extract_claims(request: &Request) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}

/// Check if user has specific role
pub fn has_role(claims: &Claims, role: &str) -> bool {
    claims.roles.contains(&role.to_string())
}
