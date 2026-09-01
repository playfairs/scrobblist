use crate::database::repositories::{SessionRepository, TrackRepository, UserRepository};
use crate::lastfm::auth::AuthService;
use crate::lastfm::client::LastFmClient;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

fn get_api_credentials() -> Result<(String, String), String> {
    let api_key = std::env::var("LASTFM_API_KEY").map_err(|_| {
        "LASTFM_API_KEY is not configured. Add it to your environment or .env file.".to_string()
    })?;
    let api_secret = std::env::var("LASTFM_API_SECRET").map_err(|_| {
        "LASTFM_API_SECRET is not configured. Add it to your environment or .env file.".to_string()
    })?;

    if api_key.trim().is_empty() || api_secret.trim().is_empty() {
        return Err("LASTFM_API_KEY and LASTFM_API_SECRET must not be empty.".to_string());
    }

    Ok((api_key, api_secret))
}

fn normalize_image_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    Some(value.replacen("http://", "https://", 1))
}

fn encode_lastfm_path(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[derive(Serialize)]
pub struct ProfileResponse {
    username: String,
    realname: Option<String>,
    url: String,
    image_url: Option<String>,
    country: Option<String>,
    playcount: u64,
    loved_songs: u64,
    age: Option<u32>,
    gender: Option<String>,
    subscriber: bool,
    registered_at: String,
    weekly_scrobbles: u64,
}

#[derive(Serialize, Clone)]
pub struct TrackResponse {
    name: String,
    artist: String,
    artist_url: String,
    album: Option<String>,
    album_url: Option<String>,
    url: String,
    image_url: Option<String>,
    now_playing: bool,
    scrobbled_at: Option<String>,
}

#[derive(Serialize)]
pub struct RecentTracksResponse {
    tracks: Vec<TrackResponse>,
    now_playing: Option<TrackResponse>,
}

#[derive(Serialize)]
pub struct TopItemResponse {
    pub name: String,
    pub artist: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub playcount: u64,
}

#[tauri::command]
pub async fn get_top_items(
    state: State<'_, AppState>,
    kind: String,
    period: String,
    limit: Option<u32>,
) -> Result<Vec<TopItemResponse>, String> {
    let valid_periods = ["7day", "1month", "3month", "6month", "12month", "overall"];
    if !valid_periods.contains(&period.as_str()) {
        return Err("Invalid chart period".to_string());
    }
    let (api_key, api_secret) = get_api_credentials()?;
    let auth_service = AuthService::new(api_key.clone(), api_secret.clone());
    let username = {
        let repo = SessionRepository::new(state.db.pool().clone());
        match repo
            .get_session()
            .await
            .map_err(|e| format!("Database error: {}", e))?
        {
            Some((saved_username, _)) => saved_username,
            None => {
                auth_service
                    .retrieve_session()
                    .map_err(|e| format!("No session: {}", e))?
                    .0
            }
        }
    };
    let client = LastFmClient::new(api_key, api_secret);
    let limit = limit.unwrap_or(50);
    let image_url = |images: &[crate::lastfm::models::Image]| {
        images
            .iter()
            .rev()
            .find_map(|image| normalize_image_url(&image.text))
    };

    match kind.as_str() {
        "artists" => {
            let mut items = Vec::new();
            for item in client
                .get_top_artists(&username, &period, limit)
                .await
                .map_err(|e| format!("Failed to fetch top artists: {}", e))?
            {
                let info = if image_url(&item.image).is_none() {
                    client.get_artist_info(&item.name).await.ok()
                } else {
                    None
                };
                let artwork = image_url(&item.image)
                    .or_else(|| info.as_ref().and_then(|artist| image_url(&artist.image)));
                let url = if item.url.is_empty() {
                    info.as_ref()
                        .map(|artist| artist.url.clone())
                        .unwrap_or_else(|| {
                            format!(
                                "https://www.last.fm/music/{}",
                                encode_lastfm_path(&item.name)
                            )
                        })
                } else {
                    item.url
                };
                items.push(TopItemResponse {
                    name: item.name,
                    artist: None,
                    url,
                    image_url: artwork,
                    playcount: item.playcount.parse().unwrap_or(0),
                });
            }
            Ok(items)
        }
        "albums" => Ok(client
            .get_top_albums(&username, &period, limit)
            .await
            .map_err(|e| format!("Failed to fetch top albums: {}", e))?
            .into_iter()
            .map(|item| TopItemResponse {
                name: item.name,
                artist: Some(item.artist.name),
                url: item.url,
                image_url: image_url(&item.image),
                playcount: item.playcount.parse().unwrap_or(0),
            })
            .collect()),
        "tracks" => Ok(client
            .get_top_tracks(&username, &period, limit)
            .await
            .map_err(|e| format!("Failed to fetch top tracks: {}", e))?
            .into_iter()
            .map(|item| TopItemResponse {
                name: item.name,
                artist: Some(item.artist.name),
                url: item.url,
                image_url: image_url(&item.image),
                playcount: item.playcount.parse().unwrap_or(0),
            })
            .collect()),
        _ => Err("Invalid chart type".to_string()),
    }
}

#[tauri::command]
pub async fn get_profile(state: State<'_, AppState>) -> Result<ProfileResponse, String> {
    let (api_key, api_secret) = get_api_credentials()?;
    let auth_service = AuthService::new(api_key.clone(), api_secret.clone());
    let (username, session_key) = {
        let repo = SessionRepository::new(state.db.pool().clone());
        match repo
            .get_session()
            .await
            .map_err(|e| format!("Database error: {}", e))?
        {
            Some(session) => session,
            None => auth_service
                .retrieve_session()
                .map_err(|e| format!("No session: {}", e))?,
        }
    };

    let user_repo = UserRepository::new(state.db.pool().clone());
    eprintln!("[user] fetching live Last.fm profile for {}", username);
    let client = LastFmClient::new(api_key, api_secret);
    let user = client
        .get_user_info(&username)
        .await
        .map_err(|e| format!("Failed to fetch user: {}", e))?;
    let loved_songs = client
        .get_loved_tracks_count(&username)
        .await
        .map_err(|e| format!("Failed to fetch loved songs: {}", e))?;
    let weekly_scrobbles = client
        .get_scrobbles_count_since(
            &session_key,
            (chrono::Utc::now() - chrono::Duration::days(7)).timestamp(),
        )
        .await
        .map_err(|e| format!("Failed to fetch weekly scrobbles: {}", e))?;

    let image_url = user
        .image
        .iter()
        .find(|img| img.size == "extralarge" && !img.text.trim().is_empty())
        .and_then(|img| normalize_image_url(&img.text))
        .or_else(|| {
            user.image
                .iter()
                .find_map(|img| normalize_image_url(&img.text))
        });

    user_repo
        .upsert_user(&user)
        .await
        .map_err(|e| format!("Failed to cache user: {}", e))?;

    Ok(ProfileResponse {
        username: user.name,
        realname: user.realname,
        url: user.url,
        image_url,
        country: user.country,
        playcount: user.playcount,
        loved_songs,
        age: user.age,
        gender: user.gender,
        subscriber: user.subscriber != 0,
        registered_at: chrono::DateTime::from_timestamp(user.registered.unixtime, 0)
            .map(|date| date.to_rfc3339())
            .unwrap_or_default(),
        weekly_scrobbles,
    })
}

#[tauri::command]
pub async fn get_recent_tracks(
    state: State<'_, AppState>,
    limit: Option<u32>,
    page: Option<u32>,
    refresh: Option<bool>,
) -> Result<RecentTracksResponse, String> {
    let (api_key, api_secret) = get_api_credentials()?;
    let auth_service = AuthService::new(api_key.clone(), api_secret.clone());
    let (username, session_key) = {
        let repo = SessionRepository::new(state.db.pool().clone());
        match repo
            .get_session()
            .await
            .map_err(|e| format!("Database error: {}", e))?
        {
            Some((saved_username, saved_session_key)) => (saved_username, saved_session_key),
            None => auth_service
                .retrieve_session()
                .map_err(|e| format!("No session: {}", e))?,
        }
    };

    let limit = limit.unwrap_or(20);
    let page = page.unwrap_or(1);
    let refresh = refresh.unwrap_or(false);

    let track_repo = TrackRepository::new(state.db.pool().clone());
    let cached_tracks = track_repo
        .get_recent_tracks(&username, limit as i64)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let now_playing = track_repo
        .get_now_playing(&username)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if !refresh && page == 1 && !cached_tracks.is_empty() {
        let track_responses: Vec<TrackResponse> = cached_tracks
            .into_iter()
            .map(|t| TrackResponse {
                name: t.track_name,
                artist: t.artist_name,
                artist_url: t.artist_url,
                album: t.album_name,
                album_url: None,
                url: t.track_url,
                image_url: t.image_url,
                now_playing: t.now_playing,
                scrobbled_at: t.scrobbled_at.map(|dt| dt.to_rfc3339()),
            })
            .collect();

        let now_playing_response = now_playing.map(|t| TrackResponse {
            name: t.track_name,
            artist: t.artist_name,
            artist_url: t.artist_url,
            album: t.album_name,
            album_url: None,
            url: t.track_url,
            image_url: t.image_url,
            now_playing: t.now_playing,
            scrobbled_at: t.scrobbled_at.map(|dt| dt.to_rfc3339()),
        });

        return Ok(RecentTracksResponse {
            tracks: track_responses,
            now_playing: now_playing_response,
        });
    }

    let client = LastFmClient::new(api_key, api_secret);
    let recent_tracks = client
        .get_recent_tracks_with_session(&session_key, Some(limit), Some(page))
        .await
        .map_err(|e| format!("Failed to fetch recent tracks: {}", e))?;

    track_repo
        .upsert_tracks(&username, &recent_tracks.track)
        .await
        .map_err(|e| format!("Failed to cache tracks: {}", e))?;

    let track_responses: Vec<TrackResponse> = recent_tracks
        .track
        .into_iter()
        .map(|t| {
            let image_url = t
                .image
                .iter()
                .find(|img| img.size == "extralarge" && !img.text.trim().is_empty())
                .and_then(|img| normalize_image_url(&img.text))
                .or_else(|| {
                    t.image
                        .iter()
                        .find_map(|img| normalize_image_url(&img.text))
                });

            let now_playing = t
                .attr
                .as_ref()
                .and_then(|attr| attr.nowplaying.as_ref())
                .map(|np| np == "true")
                .unwrap_or(false);

            let scrobbled_at = t.date.as_ref().map(|d| {
                chrono::DateTime::from_timestamp(d.uts, 0)
                    .unwrap()
                    .to_rfc3339()
            });

            TrackResponse {
                name: t.name,
                artist: t.artist.name,
                artist_url: t.artist.url,
                album: t.album.as_ref().map(|a| a.text.clone()),
                album_url: t.album.as_ref().map(|a| a.url.clone()),
                url: t.url,
                image_url,
                now_playing,
                scrobbled_at,
            }
        })
        .collect();

    let now_playing_response = track_responses.iter().find(|t| t.now_playing).cloned();

    Ok(RecentTracksResponse {
        tracks: track_responses,
        now_playing: now_playing_response,
    })
}
