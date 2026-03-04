use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum NatsResponse<T> {
    #[serde(rename = "ok")]
    Ok { data: T },
    #[serde(rename = "error")]
    Error { code: String, message: String },
}

pub mod news {
    pub mod get_article {
        use serde::{Deserialize, Serialize};

        pub const SUBJECT: &str = "news.articles.get";

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Request {
            pub id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Response {
            pub id: String,
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
            pub author: Author,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Author {
            pub id: String,
            pub email: String,
            pub display_name: String,
        }
    }

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

    pub mod get_user {
        use serde::{Deserialize, Serialize};

        pub const SUBJECT: &str = "users.get";

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Request {
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Response {
            pub id: String,
            pub email: String,
            pub display_name: String,
        }
    }

    pub mod register {
        use serde::{Deserialize, Serialize};

        pub const SUBJECT: &str = "users.register";

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Request {
            pub email: String,
            pub password: String,
            pub display_name: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Response {
            pub id: String,
            pub email: String,
            pub display_name: String,
        }
    }

    pub mod authenticate {
        use serde::{Deserialize, Serialize};

        pub const SUBJECT: &str = "users.authenticate";

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Request {
            pub email: String,
            pub password: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Response {
            pub id: String,
            pub email: String,
            pub display_name: String,
        }
    }
}
