pub mod news {
    pub mod list_articles {
        use serde::{Deserialize, Serialize};

        pub const SUBJECT: &str = "news.articles.list";

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Request {
            pub limit: i64,
            pub offset: i64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Response {
            pub articles: Vec<Article>,
            pub limit: i64,
            pub offset: i64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Article {
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
            pub deleted_at: Option<String>,
        }
    }
}

pub mod users {
    pub mod list_users {
        use serde::{Deserialize, Serialize};

        pub const SUBJECT: &str = "users.list";

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Request {
            pub limit: i64,
            pub offset: i64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Response {
            pub users: Vec<User>,
            pub limit: i64,
            pub offset: i64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct User {
            pub id: String,
            pub email: String,
            pub display_name: String,
            pub status: String,
            pub created_at: String,
            pub updated_at: String,
        }
    }
}
