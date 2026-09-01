use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CachedUser {
    pub id: i64,
    pub username: String,
    pub realname: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub country: Option<String>,
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub subscriber: i32,
    pub playcount: i64,
    pub playlists: i32,
    pub registered_unixtime: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CachedTrack {
    pub id: i64,
    pub username: String,
    pub track_name: String,
    pub artist_name: String,
    pub artist_url: String,
    pub album_name: Option<String>,
    pub track_url: String,
    pub image_url: Option<String>,
    pub now_playing: bool,
    pub scrobbled_at: Option<DateTime<Utc>>,
    pub cached_at: DateTime<Utc>,
}
