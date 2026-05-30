mod health;
mod projects;
mod setup;
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
        .merge(setup::router())
        .merge(projects::router())
        .merge(teams::router())
        .merge(tasks::router())
        .with_state(state)
}

pub fn build_app(state: AppState) -> Router {
    let api = api_router(state.clone());
    let ws = crate::ws::router(state.clone());

    let mut app = Router::new().nest("/api", api).merge(ws);

    if state.config.serves_static_ui() {
        let dist = state.config.static_dir.clone().unwrap();
        app = app.merge(crate::static_files::spa_router(dist));
    } else if let Some(dist) = &state.config.static_dir {
        tracing::warn!(path = %dist.display(), "static dir missing index.html");
    }

    if state.config.profile == Profile::Dev && !state.config.serves_static_ui() {
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
