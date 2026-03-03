pub mod auth;
pub mod news;
pub mod system;
pub mod users;

use axum::Router;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(system::router())
        .merge(auth::router())
        .merge(news::router())
        .merge(users::router())
}
