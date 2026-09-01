use crate::database::repositories::{SessionRepository, UserRepository};
use crate::lastfm::auth::{AuthError, AuthService};
use crate::lastfm::client::LastFmClient;
use crate::state::AppState;
use serde::Serialize;
use std::sync::OnceLock;
use tauri::State;
use tokio::sync::Mutex;

async fn load_saved_session_from_db(state: &AppState) -> Result<Option<(String, String)>, String> {
    let repo = SessionRepository::new(state.db.pool().clone());
    repo.get_session()
        .await
        .map_err(|e| format!("Database error: {}", e))
}

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

#[derive(Serialize)]
pub struct AuthUrlResponse {
    url: String,
}

#[derive(Serialize)]
pub struct SessionResponse {
    username: String,
    has_session: bool,
}

static AUTH_FLOW_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[tauri::command]
pub async fn get_auth_url() -> Result<AuthUrlResponse, String> {
    let (api_key, api_secret) = get_api_credentials()?;
    let auth_service = AuthService::new(api_key, api_secret);
    let url = auth_service.get_auth_url();
    Ok(AuthUrlResponse { url })
}

#[tauri::command]
pub async fn start_lastfm_auth(state: State<'_, AppState>) -> Result<SessionResponse, String> {
    let auth_guard = AUTH_FLOW_LOCK.get_or_init(|| Mutex::new(())).lock().await;

    if let Some((username, _)) = load_saved_session_from_db(&state).await? {
        drop(auth_guard);
        return Ok(SessionResponse {
            username,
            has_session: true,
        });
    }

    let (api_key, api_secret) = get_api_credentials()?;
    let auth_service = AuthService::new(api_key, api_secret);

    eprintln!("[tauri] starting Last.fm auth flow with in-flight lock");

    let session = match auth_service.start_auth_flow().await {
        Ok(session) => session,
        Err(err) => {
            drop(auth_guard);
            eprintln!("[tauri] Last.fm authentication failed: {:?}", err);
            eprintln!(
                "[tauri] backtrace: {:?}",
                std::backtrace::Backtrace::capture()
            );
            return Err(format!(
                "Last.fm authentication failed: {}. Backtrace: {:?}",
                err,
                std::backtrace::Backtrace::capture()
            ));
        }
    };

    let repo = SessionRepository::new(state.db.pool().clone());
    match repo.save_session(&session.name, &session.key).await {
        Ok(_) => {}
        Err(e) => {
            drop(auth_guard);
            return Err(format!("Failed to persist session: {}", e));
        }
    }

    let response = SessionResponse {
        username: session.name,
        has_session: true,
    };

    drop(auth_guard);
    Ok(response)
}

#[tauri::command]
pub async fn complete_auth(
    token: String,
    state: State<'_, AppState>,
) -> Result<SessionResponse, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("Last.fm auth token is empty.".to_string());
    }

    let (api_key, api_secret) = get_api_credentials()?;
    let auth_service = AuthService::new(api_key.clone(), api_secret.clone());

    let session = auth_service
        .create_session(token)
        .await
        .map_err(|e| format!("Failed to create session: {}", e))?;

    auth_service
        .save_session(&session.name, &session.key)
        .map_err(|e| format!("Failed to save session: {}", e))?;

    let db_repo = SessionRepository::new(state.db.pool().clone());
    db_repo
        .save_session(&session.name, &session.key)
        .await
        .map_err(|e| format!("Failed to persist session to database: {}", e))?;

    let client = LastFmClient::new(api_key, api_secret);
    let user = client
        .get_user_info(&session.name)
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    let user_repo = UserRepository::new(state.db.pool().clone());
    user_repo
        .upsert_user(&user)
        .await
        .map_err(|e| format!("Failed to cache user: {}", e))?;

    Ok(SessionResponse {
        username: session.name,
        has_session: true,
    })
}

#[tauri::command]
pub async fn handle_auth_callback(
    token: String,
    state: State<'_, AppState>,
) -> Result<SessionResponse, String> {
    complete_auth(token, state).await
}

#[tauri::command]
pub async fn get_session(state: State<'_, AppState>) -> Result<SessionResponse, String> {
    if let Some((username, _)) = load_saved_session_from_db(&state).await? {
        return Ok(SessionResponse {
            username,
            has_session: true,
        });
    }

    let (api_key, api_secret) = match get_api_credentials() {
        Ok(credentials) => credentials,
        Err(_) => {
            return Ok(SessionResponse {
                username: String::new(),
                has_session: false,
            });
        }
    };

    let auth_service = AuthService::new(api_key, api_secret);

    match auth_service.retrieve_session() {
        Ok((username, _)) => Ok(SessionResponse {
            username,
            has_session: true,
        }),
        Err(AuthError::NoSession) => Ok(SessionResponse {
            username: String::new(),
            has_session: false,
        }),
        Err(e) => Err(format!("Failed to get session: {}", e)),
    }
}
