use axum::{extract::State, routing::get, Json, Router};

use crate::{app::AppState, error::AppError};

use super::{dto::ListArticlesResponse, nats};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/news/articles", get(list_articles))
}

async fn list_articles(State(state): State<AppState>) -> Result<Json<ListArticlesResponse>, AppError> {
    let response = nats::request_list_articles(&state).await?;

    Ok(Json(response))
}