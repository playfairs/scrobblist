use crate::database::DatabaseError;
use crate::database::models::{CachedTrack, CachedUser};
use crate::lastfm::models::{Track, User};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

pub struct SessionRepository {
    pool: SqlitePool,
}

impl SessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn save_session(
        &self,
        username: &str,
        session_key: &str,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO sessions (username, session_key, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(username) DO UPDATE SET
                session_key = excluded.session_key,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(username)
        .bind(session_key)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_session(&self) -> Result<Option<(String, String)>, DatabaseError> {
        let row = sqlx::query(
            "SELECT username, session_key FROM sessions ORDER BY updated_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(record) => {
                let username: String = record.try_get("username")?;
                let session_key: String = record.try_get("session_key")?;
                Ok(Some((username, session_key)))
            }
            None => Ok(None),
        }
    }

    pub async fn clear_session(&self) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM sessions")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert_user(&self, user: &User) -> Result<CachedUser, DatabaseError> {
        let image_url = user
            .image
            .iter()
            .find(|img| img.size == "extralarge")
            .map(|img| img.text.clone())
            .or_else(|| user.image.first().map(|img| img.text.clone()));

        let result = sqlx::query_as::<_, CachedUser>(
            r#"
            INSERT INTO users (
                username, realname, url, image_url, country, age, gender,
                subscriber, playcount, playlists, registered_unixtime, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(username) DO UPDATE SET
                realname = excluded.realname,
                url = excluded.url,
                image_url = excluded.image_url,
                country = excluded.country,
                age = excluded.age,
                gender = excluded.gender,
                subscriber = excluded.subscriber,
                playcount = excluded.playcount,
                playlists = excluded.playlists,
                registered_unixtime = excluded.registered_unixtime,
                updated_at = excluded.updated_at
            RETURNING *
            "#,
        )
        .bind(&user.name)
        .bind(&user.realname)
        .bind(&user.url)
        .bind(&image_url)
        .bind(&user.country)
        .bind(user.age.map(|a| a as i32))
        .bind(&user.gender)
        .bind(user.subscriber as i32)
        .bind(user.playcount as i64)
        .bind(user.playlists as i32)
        .bind(user.registered.unixtime)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_user(&self, username: &str) -> Result<Option<CachedUser>, DatabaseError> {
        let user = sqlx::query_as::<_, CachedUser>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }
}

pub struct TrackRepository {
    pool: SqlitePool,
}

impl TrackRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert_tracks(
        &self,
        username: &str,
        tracks: &[Track],
    ) -> Result<(), DatabaseError> {
        for track in tracks {
            let image_url = track
                .image
                .iter()
                .find(|img| img.size == "extralarge")
                .map(|img| img.text.clone())
                .or_else(|| track.image.first().map(|img| img.text.clone()));

            let now_playing = track
                .attr
                .as_ref()
                .and_then(|attr| attr.nowplaying.as_ref())
                .map(|np| np == "true")
                .unwrap_or(false);

            let scrobbled_at = track
                .date
                .as_ref()
                .map(|d| chrono::DateTime::from_timestamp(d.uts, 0).unwrap());

            sqlx::query(
                r#"
                INSERT INTO tracks (
                    username, track_name, artist_name, artist_url, album_name,
                    track_url, image_url, now_playing, scrobbled_at, cached_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(username)
            .bind(&track.name)
            .bind(&track.artist.name)
            .bind(&track.artist.url)
            .bind(track.album.as_ref().map(|a| a.text.clone()))
            .bind(&track.url)
            .bind(&image_url)
            .bind(now_playing as i32)
            .bind(scrobbled_at)
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_recent_tracks(
        &self,
        username: &str,
        limit: i64,
    ) -> Result<Vec<CachedTrack>, DatabaseError> {
        let tracks = sqlx::query_as::<_, CachedTrack>(
            r#"
            SELECT * FROM tracks 
            WHERE username = ? 
            ORDER BY scrobbled_at DESC, cached_at DESC
            LIMIT ?
            "#,
        )
        .bind(username)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(tracks)
    }

    pub async fn get_now_playing(
        &self,
        username: &str,
    ) -> Result<Option<CachedTrack>, DatabaseError> {
        let track = sqlx::query_as::<_, CachedTrack>(
            "SELECT * FROM tracks WHERE username = ? AND now_playing = 1 LIMIT 1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(track)
    }
}
