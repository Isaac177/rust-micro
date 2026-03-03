pub mod system;

use axum::Router;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new().merge(system::router())
}
