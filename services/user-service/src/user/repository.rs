use anyhow::Result;
use contracts::users::list_users::{Response as ListUsersResponse, User};
use sqlx::{PgPool, Row};

pub struct UserRow {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub status: String,
}

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

pub async fn create_user(
    pool: &PgPool,
    id: uuid::Uuid,
    email: &str,
    display_name: &str,
    password_hash: &str,
) -> Result<UserRow> {
    let row = sqlx::query(
        r#"
        INSERT INTO users (id, email, display_name, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id::text AS id,
            email,
            display_name,
            password_hash,
            status::text AS status
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(display_name)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(UserRow {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        password_hash: row.get("password_hash"),
        status: row.get("status"),
    })
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>> {
    let row = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            email,
            display_name,
            password_hash,
            status::text AS status
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| UserRow {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        password_hash: row.get("password_hash"),
        status: row.get("status"),
    }))
}
