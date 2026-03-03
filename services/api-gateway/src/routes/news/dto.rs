use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ListArticlesRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListArticlesResponse {
    pub articles: Vec<ArticleSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleSummary {
    pub id: String,
    pub title: String,
    pub slug: String,
}