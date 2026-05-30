use std::path::PathBuf;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

/// Serves the built Svelte app with SPA fallback.
pub fn spa_router(dist: PathBuf) -> Router {
    let index = dist.join("index.html");
    Router::new().fallback_service(ServeDir::new(dist).not_found_service(ServeFile::new(index)))
}
