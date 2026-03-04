use anyhow::Result;
use contracts::news::list_articles::{Article, Response as ListArticlesResponse};
use sqlx::{PgPool, Row};

pub struct ArticleRow {
    pub id: String,
    pub author_user_id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub body_markdown: String,
    pub body_html: Option<String>,
    pub cover_image_url: Option<String>,
    pub status: String,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_article(pool: &PgPool, id: &str) -> Result<Option<ArticleRow>> {
    let row = sqlx::query(
        r#"
        SELECT
            id,
            author_user_id,
            slug,
            title,
            summary,
            body_markdown,
            body_html,
            cover_image_url,
            status::text AS status,
            to_char(published_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS published_at,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at,
            to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS updated_at
        FROM articles
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| ArticleRow {
        id: r.get("id"),
        author_user_id: r.get("author_user_id"),
        slug: r.get("slug"),
        title: r.get("title"),
        summary: r.get("summary"),
        body_markdown: r.get("body_markdown"),
        body_html: r.get("body_html"),
        cover_image_url: r.get("cover_image_url"),
        status: r.get("status"),
        published_at: r.get("published_at"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

pub async fn list_articles(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<ListArticlesResponse> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            author_user_id,
            slug,
            title,
            summary,
            body_markdown,
            body_html,
            cover_image_url,
            status::text AS status,
            to_char(published_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS published_at,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at,
            to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS updated_at,
            to_char(deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS deleted_at
        FROM articles
        WHERE status = 'published'
          AND deleted_at IS NULL
        ORDER BY published_at DESC, id DESC
        LIMIT $1
        OFFSET $2
        "#,
    )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let articles = rows
        .into_iter()
        .map(|row| Article {
            id: row.get("id"),
            author_user_id: row.get("author_user_id"),
            slug: row.get("slug"),
            title: row.get("title"),
            summary: row.get("summary"),
            body_markdown: row.get("body_markdown"),
            body_html: row.get("body_html"),
            cover_image_url: row.get("cover_image_url"),
            status: row.get("status"),
            published_at: row.get("published_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        })
        .collect();

    Ok(ListArticlesResponse {
        articles,
        limit,
        offset,
    })
}
