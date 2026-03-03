use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Debug, Clone, Copy)]
struct SeedConfig {
    articles: i64,
    tags: i64,
    tags_per_article: i64,
    truncate: bool,
}

impl Default for SeedConfig {
    fn default() -> Self {
        Self {
            articles: 1_000,
            tags: 50,
            tags_per_article: 3,
            truncate: false,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_local_env();

    let config = parse_args()?;
    let database_url = resolve_database_url()?;
    let pool = connect_pool(&database_url).await?;

    run_seed(&pool, config).await?;

    println!(
        "seed completed: articles={}, tags={}, tags_per_article={}, truncate={}",
        config.articles, config.tags, config.tags_per_article, config.truncate
    );

    Ok(())
}

fn parse_args() -> anyhow::Result<SeedConfig> {
    let mut cfg = SeedConfig::default();
    let args: Vec<String> = env::args().collect();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--articles" => {
                cfg.articles = parse_i64_arg(&args, i, "--articles")?;
                i += 2;
            }
            "--tags" => {
                cfg.tags = parse_i64_arg(&args, i, "--tags")?;
                i += 2;
            }
            "--tags-per-article" => {
                cfg.tags_per_article = parse_i64_arg(&args, i, "--tags-per-article")?;
                i += 2;
            }
            "--truncate" => {
                cfg.truncate = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            arg => anyhow::bail!("unknown argument: {arg}"),
        }
    }

    if cfg.articles <= 0 || cfg.tags <= 0 || cfg.tags_per_article <= 0 {
        anyhow::bail!("all numeric options must be > 0");
    }

    Ok(cfg)
}

fn parse_i64_arg(args: &[String], index: usize, flag: &str) -> anyhow::Result<i64> {
    let Some(raw) = args.get(index + 1) else {
        anyhow::bail!("missing value for {flag}");
    };

    raw.parse::<i64>()
        .map_err(|e| anyhow::anyhow!("invalid value for {flag}: {raw} ({e})"))
}

fn print_help() {
    println!("Usage: cargo run -p news-service --bin seed -- [options]");
    println!("Options:");
    println!("  --articles <N>             Number of articles to create (default: 1000)");
    println!("  --tags <N>                 Number of tags to create (default: 50)");
    println!("  --tags-per-article <N>     Tag links per article (default: 3)");
    println!("  --truncate                 Truncate article tables before seeding");
}

fn load_local_env() {
    let candidates = ["services/news-service/.env.dev", ".env.dev"];

    for path in candidates {
        if dotenvy::from_filename(path).is_ok() {
            break;
        }
    }
}

fn resolve_database_url() -> anyhow::Result<String> {
    if let Ok(url) = env::var("DATABASE_URL") {
        return Ok(url);
    }

    anyhow::bail!("DATABASE_URL is required");
}

async fn connect_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}

async fn run_seed(pool: &PgPool, cfg: SeedConfig) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    if cfg.truncate {
        sqlx::query("TRUNCATE TABLE article_tags, articles, tags CASCADE")
            .execute(&mut *tx)
            .await?;
    }

    let batch = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    sqlx::query(
        r#"
        INSERT INTO tags (id, slug, name)
        SELECT
            format('tag_%s_%s', $1, gs),
            format('seed-batch-%s-tag-%s', $1, gs),
            format('Seed Tag %s-%s', $1, gs)
        FROM generate_series(1, $2) AS gs
        "#,
    )
        .bind(batch)
        .bind(cfg.tags)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO articles (
            id,
            author_user_id,
            slug,
            title,
            summary,
            body_markdown,
            body_html,
            cover_image_url,
            status,
            published_at
        )
        SELECT
            format('article_%s_%s', $1, gs),
            format('user_%s', ((gs - 1) % 100) + 1),
            format('seed-batch-%s-article-%s', $1, gs),
            format('Seed Article %s-%s', $1, gs),
            format('Summary for seed article %s-%s', $1, gs),
            format(
                '# Seed Article %1$s-%2$s

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.

Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Integer nec odio. Praesent libero. Sed cursus ante dapibus diam. Sed nisi. Nulla quis sem at nibh elementum imperdiet.

## Operational Context

Curabitur sodales ligula in libero. Sed dignissim lacinia nunc. Curabitur tortor. Pellentesque nibh. Aenean quam. In scelerisque sem at dolor. Maecenas mattis.

Vestibulum lacinia arcu eget nulla. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Nam nec ante.

- batch: %1$s
- article: %2$s
- author_user_id: user_%3$s',
                $1,
                gs,
                ((gs - 1) % 100) + 1
            ),
            format(
                '<h1>Seed Article %1$s-%2$s</h1><p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p><p>Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.</p><p>Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Integer nec odio. Praesent libero. Sed cursus ante dapibus diam. Sed nisi. Nulla quis sem at nibh elementum imperdiet.</p><h2>Operational Context</h2><p>Curabitur sodales ligula in libero. Sed dignissim lacinia nunc. Curabitur tortor. Pellentesque nibh. Aenean quam. In scelerisque sem at dolor. Maecenas mattis.</p><p>Vestibulum lacinia arcu eget nulla. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Nam nec ante.</p><ul><li>batch: %1$s</li><li>article: %2$s</li><li>author_user_id: user_%3$s</li></ul>',
                $1,
                gs,
                ((gs - 1) % 100) + 1
            ),
            format('https://picsum.photos/seed/%s-%s/1200/630', $1, gs),
            CASE
                WHEN gs % 7 = 0 THEN 'draft'::article_status
                WHEN gs % 5 = 0 THEN 'review'::article_status
                WHEN gs % 11 = 0 THEN 'archived'::article_status
                ELSE 'published'::article_status
            END,
            CASE
                WHEN gs % 7 = 0 OR gs % 5 = 0 OR gs % 11 = 0 THEN NULL
                ELSE NOW() - ((gs % 30) * INTERVAL '1 day') - ((gs % 24) * INTERVAL '1 hour')
            END
        FROM generate_series(1, $2) AS gs
        "#,
    )
        .bind(batch)
        .bind(cfg.articles)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO article_tags (article_id, tag_id)
        SELECT
            format('article_%s_%s', $1, article_idx),
            format('tag_%s_%s', $1, (((article_idx + tag_offset - 2) % $2) + 1))
        FROM generate_series(1, $3) AS article_idx
        CROSS JOIN generate_series(1, $4) AS tag_offset
        ON CONFLICT DO NOTHING
        "#,
    )
        .bind(batch)
        .bind(cfg.tags)
        .bind(cfg.articles)
        .bind(cfg.tags_per_article)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
