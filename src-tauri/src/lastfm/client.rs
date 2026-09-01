use crate::lastfm::models::*;
use crate::lastfm::signing::sign;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use thiserror::Error;

const API_URL: &str = "https://ws.audioscrobbler.com/2.0/";

#[derive(Error, Debug)]
pub enum LastFmClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Last.fm API error: {0}")]
    ApiError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Missing API key")]
    MissingApiKey,
}

pub struct LastFmClient {
    client: Client,
    api_key: String,
    api_secret: String,
}

impl LastFmClient {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .expect("failed to create Last.fm HTTP client"),
            api_key,
            api_secret,
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        params: HashMap<String, String>,
    ) -> Result<T, LastFmClientError> {
        let mut params = params;
        params.insert("api_key".to_string(), self.api_key.clone());
        params.insert("format".to_string(), "json".to_string());

        let response = self.client.get(API_URL).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(LastFmClientError::ApiError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let text = response.text().await?;

        if let Ok(error) = serde_json::from_str::<LastFmError>(&text) {
            return Err(LastFmClientError::ApiError(error.message));
        }

        serde_json::from_str(&text).map_err(LastFmClientError::SerializationError)
    }

    pub async fn post_signed<T: DeserializeOwned>(
        &self,
        mut params: HashMap<String, String>,
    ) -> Result<T, LastFmClientError> {
        params.insert("api_key".to_string(), self.api_key.clone());
        params.insert("format".to_string(), "json".to_string());

        let mut sorted_params = BTreeMap::new();
        for (key, value) in params.iter() {
            sorted_params.insert(key.clone(), value.clone());
        }

        let api_sig = sign(&sorted_params, &self.api_secret);
        params.insert("api_sig".to_string(), api_sig.clone());

        eprintln!(
            "[lastfm] POST {} with params: {}",
            API_URL,
            params
                .keys()
                .map(|key| key.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        eprintln!(
            "[lastfm] api_sig length={} method={}",
            api_sig.len(),
            params.get("method").map(|v| v.as_str()).unwrap_or("<none>")
        );

        let response = self.client.post(API_URL).form(&params).send().await?;
        let status = response.status();
        let text = response.text().await?;

        eprintln!("[lastfm] response status={} body={}", status, text);

        if !status.is_success() {
            return Err(LastFmClientError::ApiError(format!(
                "HTTP error: {}",
                status
            )));
        }

        if let Ok(error) = serde_json::from_str::<LastFmError>(&text) {
            return Err(LastFmClientError::ApiError(error.message));
        }

        serde_json::from_str(&text).map_err(LastFmClientError::SerializationError)
    }

    fn create_api_sig(&self, params: &HashMap<String, String>) -> String {
        let mut sorted = BTreeMap::new();
        for (key, value) in params {
            sorted.insert(key.clone(), value.clone());
        }
        sign(&sorted, &self.api_secret)
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    pub async fn get_user_info(&self, username: &str) -> Result<User, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "user.getInfo".to_string());
        params.insert("user".to_string(), username.to_string());

        let response: UserInfoResponse = self.get(params).await?;
        Ok(response.user)
    }

    pub async fn get_recent_tracks(
        &self,
        username: &str,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> Result<RecentTracks, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "user.getRecentTracks".to_string());
        params.insert("user".to_string(), username.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        if let Some(page) = page {
            params.insert("page".to_string(), page.to_string());
        }

        let response: RecentTracksResponse = self.get(params).await?;
        Ok(response.recenttracks)
    }

    pub async fn get_recent_tracks_with_session(
        &self,
        session_key: &str,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> Result<RecentTracks, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "user.getRecentTracks".to_string());
        params.insert("sk".to_string(), session_key.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        if let Some(page) = page {
            params.insert("page".to_string(), page.to_string());
        }

        let response: RecentTracksResponse = self.get(params).await?;
        Ok(response.recenttracks)
    }

    pub async fn get_scrobbles_count_since(
        &self,
        session_key: &str,
        from: i64,
    ) -> Result<u64, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "user.getRecentTracks".to_string());
        params.insert("sk".to_string(), session_key.to_string());
        params.insert("from".to_string(), from.to_string());
        params.insert("limit".to_string(), "1".to_string());
        let response: RecentTracksResponse = self.get(params).await?;
        Ok(response.recenttracks.attr.total.parse().unwrap_or(0))
    }

    pub async fn get_top_artists(
        &self,
        username: &str,
        period: &str,
        limit: u32,
    ) -> Result<Vec<TopArtist>, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "user.getTopArtists".to_string());
        params.insert("user".to_string(), username.to_string());
        params.insert("period".to_string(), period.to_string());
        params.insert("limit".to_string(), limit.to_string());
        Ok(self
            .get::<TopArtistsResponse>(params)
            .await?
            .topartists
            .artist)
    }

    pub async fn get_artist_info(&self, artist: &str) -> Result<ArtistInfo, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "artist.getInfo".to_string());
        params.insert("artist".to_string(), artist.to_string());
        Ok(self.get::<ArtistInfoResponse>(params).await?.artist)
    }

    pub async fn get_top_albums(
        &self,
        username: &str,
        period: &str,
        limit: u32,
    ) -> Result<Vec<TopAlbum>, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "user.getTopAlbums".to_string());
        params.insert("user".to_string(), username.to_string());
        params.insert("period".to_string(), period.to_string());
        params.insert("limit".to_string(), limit.to_string());
        Ok(self.get::<TopAlbumsResponse>(params).await?.topalbums.album)
    }

    pub async fn get_top_tracks(
        &self,
        username: &str,
        period: &str,
        limit: u32,
    ) -> Result<Vec<TopTrack>, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "user.getTopTracks".to_string());
        params.insert("user".to_string(), username.to_string());
        params.insert("period".to_string(), period.to_string());
        params.insert("limit".to_string(), limit.to_string());
        Ok(self.get::<TopTracksResponse>(params).await?.toptracks.track)
    }

    pub async fn get_loved_tracks_count(&self, username: &str) -> Result<u64, LastFmClientError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "user.getLovedTracks".to_string());
        params.insert("user".to_string(), username.to_string());
        params.insert("limit".to_string(), "1".to_string());
        let response = self.get::<LovedTracksResponse>(params).await?;
        Ok(response.lovedtracks.attr.total.parse().unwrap_or(0))
    }
}
