use axum::{
    routing::{get, post, put},
    http::StatusCode,
    Json, Router, extract::Path,
    response::Html,
    http::Method,
    middleware,
};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use std::env;

mod auth;
mod routes;
mod markdown;
mod utils;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Starting blog backend server with Axum and Keycloak auth...");

    // Get port from environment or use default
    let port = env::var("BLOG_SERVICE_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers(Any)
        .allow_origin(Any);

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/posts/:slug", get(get_post))
        .route("/preview", post(preview_markdown))
        .nest("/admin", Router::new()
            .route("/new", post(create_post))
            .route("/edit/:slug", put(edit_post))
            .layer(middleware::from_fn(auth::auth_middleware))
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🌐 Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "message": "Blog backend is running with Axum and Keycloak auth",
        "port": env::var("BLOG_SERVICE_PORT").unwrap_or_else(|_| "8000".to_string())
    }))
}

async fn get_post(Path(slug): Path<String>) -> Result<Html<String>, StatusCode> {
    println!("🔍 Attempting to get post with slug: {}", slug);
    
    match crate::markdown::reader::read_and_render_markdown(&slug) {
        Ok(html_content) => {
            println!("✅ Successfully rendered post: {}", slug);
            Ok(Html(html_content))
        }
        Err(e) => {
            println!("❌ Failed to render post {}: {:?}", slug, e);
            Ok(Html("<h1>Post not found</h1><p>The requested post could not be found.</p>".to_string()))
        }
    }
}

#[derive(serde::Deserialize)]
struct PreviewRequest {
    content: String,
}

async fn preview_markdown(Json(payload): Json<PreviewRequest>) -> Html<String> {
    let html_content = crate::markdown::reader::markdown_to_html(&payload.content);
    Html(html_content)
}

#[derive(serde::Deserialize)]
struct CreatePostRequest {
    title: String,
    content: String,
}

#[derive(serde::Serialize)]
struct AdminResponse {
    success: bool,
    message: String,
    slug: Option<String>,
}

async fn create_post(Json(payload): Json<CreatePostRequest>) -> Result<Json<AdminResponse>, StatusCode> {
    // Authentication is handled by middleware
    let slug = crate::utils::generate_slug(&payload.title);
    
    // Create the post
    let post = crate::markdown::Post {
        slug: slug.clone(),
        title: payload.title,
        author: "admin".to_string(), // TODO: Get from JWT claims
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        content: payload.content,
    };
    
    // Save the post
    match crate::markdown::writer::create_post(&post) {
        Ok(_) => {
            println!("✅ Post created successfully: {}", slug);
            Ok(Json(AdminResponse {
                success: true,
                message: "Post created successfully".to_string(),
                slug: Some(slug),
            }))
        }
        Err(e) => {
            println!("❌ Failed to create post: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(serde::Deserialize)]
struct UpdatePostRequest {
    title: String,
    content: String,
}

async fn edit_post(
    Path(slug): Path<String>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<AdminResponse>, StatusCode> {
    // Authentication is handled by middleware
    
    // Create the updated post
    let post = crate::markdown::Post {
        slug: slug.clone(),
        title: payload.title,
        author: "admin".to_string(), // TODO: Get from JWT claims
        created_at: chrono::Utc::now(), // TODO: Get from existing post
        updated_at: chrono::Utc::now(),
        content: payload.content,
    };
    
    // Update the post
    match crate::markdown::writer::update_post(&post) {
        Ok(_) => {
            println!("✅ Post updated successfully: {}", slug);
            Ok(Json(AdminResponse {
                success: true,
                message: "Post updated successfully".to_string(),
                slug: Some(slug),
            }))
        }
        Err(e) => {
            println!("❌ Failed to update post: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
