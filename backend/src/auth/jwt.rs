use anyhow::{Result, Context};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::Deserialize;
use crate::auth::Claims;

#[derive(Debug, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
    pub kid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JwtPayload {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub exp: u64,
    pub iat: u64,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<ResourceAccess>,
}

#[derive(Debug, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceAccess {
    pub account: Option<ClientAccess>,
    #[serde(rename = "blog-admin")]
    pub blog_admin: Option<ClientAccess>,
}

#[derive(Debug, Deserialize)]
pub struct ClientAccess {
    pub roles: Vec<String>,
}

#[derive(Debug)]
pub struct KeycloakConfig {
    pub realm: String,
    pub client_id: String,
    pub issuer_url: String,
    pub public_key: String,
}

impl Default for KeycloakConfig {
    fn default() -> Self {
        Self {
            realm: "blog-realm".to_string(),
            client_id: "blog-backend".to_string(),
            issuer_url: "http://localhost:8080/auth/realms/blog-realm".to_string(),
            public_key: "".to_string(), // Will be fetched from Keycloak
        }
    }
}

/// Validate JWT token from Keycloak
pub async fn validate_token(token: &str) -> Result<Claims> {
    let config = KeycloakConfig::default();
    
    // Remove "Bearer " prefix if present
    let token = token.trim_start_matches("Bearer ").trim();
    
    // Fetch public key from Keycloak
    let public_key = fetch_keycloak_public_key(&config).await?;
    
    let decoding_key = DecodingKey::from_rsa_pem(public_key.as_bytes())
        .or_else(|_| Ok::<DecodingKey, jsonwebtoken::errors::Error>(DecodingKey::from_secret(public_key.as_ref())))
        .context("Failed to create decoding key")?;
    
    let token_data = decode::<JwtPayload>(
        token,
        &decoding_key,
        &Validation::new(Algorithm::RS256),
    )
    .context("Failed to decode JWT token")?;
    
    let payload = token_data.claims;
    
    // Extract roles from realm_access or resource_access
    let roles = if let Some(realm_access) = payload.realm_access {
        realm_access.roles
    } else if let Some(resource_access) = payload.resource_access {
        if let Some(blog_admin) = resource_access.blog_admin {
            blog_admin.roles
        } else {
            vec![]
        }
    } else {
        vec![]
    };
    
    Ok(Claims {
        sub: payload.sub,
        roles,
    })
}

/// Fetch public key from Keycloak
pub async fn fetch_keycloak_public_key(config: &KeycloakConfig) -> Result<String> {
    // TODO: Implement proper Keycloak public key fetching
    // For now, return a placeholder - this will be implemented later
    // when we add the reqwest dependency back
    Ok("placeholder-public-key".to_string())
}

/// Extract token from Authorization header
pub fn extract_token_from_header(auth_header: &str) -> Result<&str> {
    if auth_header.starts_with("Bearer ") {
        Ok(&auth_header[7..])
    } else {
        Err(anyhow::anyhow!("Invalid authorization header format"))
    }
}
