use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            realname TEXT,
            url TEXT NOT NULL,
            image_url TEXT,
            country TEXT,
            age INTEGER,
            gender TEXT,
            subscriber INTEGER NOT NULL DEFAULT 0,
            playcount INTEGER NOT NULL DEFAULT 0,
            playlists INTEGER NOT NULL DEFAULT 0,
            registered_unixtime INTEGER NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            track_name TEXT NOT NULL,
            artist_name TEXT NOT NULL,
            artist_url TEXT NOT NULL,
            album_name TEXT,
            track_url TEXT NOT NULL,
            image_url TEXT,
            now_playing INTEGER NOT NULL DEFAULT 0,
            scrobbled_at TEXT,
            cached_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            session_key TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tracks_username ON tracks(username)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tracks_scrobbled_at ON tracks(scrobbled_at)")
        .execute(pool)
        .await?;

    Ok(())
}
