pub mod auth;
pub mod news;
pub mod system;

use axum::Router;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(system::router())
        .merge(auth::router())
        .merge(news::router())
}
