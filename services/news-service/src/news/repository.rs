use anyhow::Result;
use contracts::news::list_articles::{ArticleSummary, Response as ListArticlesResponse};
use sqlx::{PgPool, Row};

pub async fn list_articles(pool: &PgPool) -> Result<ListArticlesResponse> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, slug
        FROM articles
        ORDER BY published_at DESC, slug ASC
        "#,
    )
        .fetch_all(pool)
        .await?;

    let articles = rows
        .into_iter()
        .map(|row| ArticleSummary {
            id: row.get("id"),
            title: row.get("title"),
            slug: row.get("slug"),
        })
        .collect();

    Ok(ListArticlesResponse { articles })
}
