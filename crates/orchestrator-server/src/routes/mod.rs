mod health;
mod projects;
mod tasks;
mod teams;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::app_state::AppState;
use crate::config::Profile;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .merge(projects::router())
        .merge(teams::router())
        .merge(tasks::router())
        .with_state(state)
}

pub fn build_app(state: AppState) -> Router {
    let api = api_router(state.clone());
    let ws = crate::ws::router(state.clone());

    let mut app = Router::new().nest("/api", api).merge(ws);

    if state.config.profile == Profile::Prod {
        if let Some(dist) = &state.config.static_dir {
            if dist.join("index.html").exists() {
                app = app.merge(crate::static_files::spa_router(dist.clone()));
            } else {
                tracing::warn!(path = %dist.display(), "static dir missing index.html");
            }
        }
    }

    if state.config.profile == Profile::Dev {
        let cors = CorsLayer::new()
            .allow_origin([
                "http://localhost:5173".parse().unwrap(),
                "http://127.0.0.1:5173".parse().unwrap(),
            ])
            .allow_methods(Any)
            .allow_headers(Any);
        app = app.layer(cors);
    }

    app.layer(TraceLayer::new_for_http())
}
