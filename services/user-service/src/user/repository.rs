use anyhow::Result;
use contracts::users::list_users::{Response as ListUsersResponse, User};
use sqlx::{PgPool, Row};

pub async fn list_users(pool: &PgPool, limit: i64, offset: i64) -> Result<ListUsersResponse> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            email,
            display_name,
            status::text AS status,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at,
            to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS updated_at
        FROM users
        ORDER BY created_at DESC, id DESC
        LIMIT $1
        OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(ListUsersResponse {
        users,
        limit,
        offset,
    })
}
