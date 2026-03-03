


pub mod news {
    pub mod list_articles {
        use serde::{Deserialize, Serialize};

        pub const SUBJECT: &str = "news.articles.list";

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Request {}

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Response {
            pub articles: Vec<ArticleSummary>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ArticleSummary {
            pub id: String,
            pub title: String,
            pub slug: String,
        }
    }
}