CREATE TYPE article_status AS ENUM (
    'draft',
    'review',
    'published',
    'archived'
    );

CREATE OR REPLACE FUNCTION set_updated_at()
    RETURNS TRIGGER
AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE articles (
                          id TEXT PRIMARY KEY,
                          author_user_id TEXT NOT NULL,
                          slug TEXT NOT NULL UNIQUE,
                          title TEXT NOT NULL,
                          summary TEXT NOT NULL,
                          body_markdown TEXT NOT NULL,
                          body_html TEXT,
                          cover_image_url TEXT,
                          status article_status NOT NULL DEFAULT 'draft',
                          published_at TIMESTAMPTZ,
                          created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                          updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                          deleted_at TIMESTAMPTZ,

                          CONSTRAINT articles_author_user_id_not_blank CHECK (length(btrim(author_user_id)) > 0),
                          CONSTRAINT articles_slug_format CHECK (slug ~ '^[a-z0-9]+(?:-[a-z0-9]+)*$'),
                          CONSTRAINT articles_title_not_blank CHECK (length(btrim(title)) > 0),
                          CONSTRAINT articles_summary_not_blank CHECK (length(btrim(summary)) > 0),
                          CONSTRAINT articles_body_markdown_not_blank CHECK (length(btrim(body_markdown)) > 0),
                          CONSTRAINT articles_published_requires_timestamp CHECK (
                              status <> 'published' OR published_at IS NOT NULL
                              )
);

CREATE TABLE tags (
                      id TEXT PRIMARY KEY,
                      slug TEXT NOT NULL UNIQUE,
                      name TEXT NOT NULL,
                      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

                      CONSTRAINT tags_slug_format CHECK (slug ~ '^[a-z0-9]+(?:-[a-z0-9]+)*$'),
                      CONSTRAINT tags_name_not_blank CHECK (length(btrim(name)) > 0)
);

CREATE TABLE article_tags (
                              article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                              tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                              created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                              PRIMARY KEY (article_id, tag_id)
);

CREATE INDEX idx_articles_public_listing
    ON articles (published_at DESC, id DESC)
    WHERE status = 'published' AND deleted_at IS NULL;

CREATE INDEX idx_articles_author_user_id
    ON articles (author_user_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_articles_status
    ON articles (status);

CREATE INDEX idx_article_tags_tag_id
    ON article_tags (tag_id);

CREATE TRIGGER trg_articles_set_updated_at
    BEFORE UPDATE ON articles
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
