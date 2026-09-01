use crate::lastfm::client::LastFmClient;
use crate::lastfm::models::Session;
use keyring::Entry;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const SERVICE_NAME: &str = "Scrobblist";
const USERNAME_KEY: &str = "username";
const SESSION_KEY_KEY: &str = "session_key";
const CALLBACK_URL: &str = "http://localhost:8080/callback";

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Keychain error: {0}")]
    KeychainError(#[from] keyring::Error),
    #[error("Last.fm client error: {0}")]
    ClientError(#[from] crate::lastfm::client::LastFmClientError),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("No session found")]
    NoSession,
    #[error("Callback server failed: {0}")]
    CallbackServer(String),
    #[error("Authentication timed out")]
    Timeout,
    #[error("Invalid callback token")]
    InvalidCallback,
}

pub struct AuthService {
    client: LastFmClient,
}

impl AuthService {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            client: LastFmClient::new(api_key, api_secret),
        }
    }

    pub fn get_auth_url_for_token(&self, token: &str) -> String {
        let callback =
            std::env::var("LASTFM_CALLBACK_URL").unwrap_or_else(|_| CALLBACK_URL.to_string());
        self.get_auth_url_for_token_with_callback(token, &callback)
    }

    fn get_auth_url_for_token_with_callback(&self, token: &str, callback: &str) -> String {
        let encoded_token =
            url::form_urlencoded::byte_serialize(token.as_bytes()).collect::<String>();
        let encoded_callback =
            url::form_urlencoded::byte_serialize(callback.as_bytes()).collect::<String>();
        format!(
            "https://www.last.fm/api/auth/?api_key={}&token={}&cb={}",
            self.client.get_api_key(),
            encoded_token,
            encoded_callback
        )
    }

    pub fn get_auth_url(&self) -> String {
        let token = "";
        self.get_auth_url_for_token(token)
    }

    fn callback_url() -> String {
        std::env::var("LASTFM_CALLBACK_URL").unwrap_or_else(|_| CALLBACK_URL.to_string())
    }

    async fn bind_callback_listener() -> Result<(TcpListener, String), AuthError> {
        let requested_callback = Self::callback_url();
        let parsed = url::Url::parse(&requested_callback).map_err(|_| {
            AuthError::CallbackServer(format!("Invalid callback URL: {}", requested_callback))
        })?;

        let preferred_port = parsed.port_or_known_default().unwrap_or(8080);
        let bind_targets = vec![
            format!("127.0.0.1:{}", preferred_port),
            "127.0.0.1:0".to_string(),
            format!("[::1]:{}", preferred_port),
            "[::1]:0".to_string(),
        ];

        for addr in bind_targets {
            match TcpListener::bind(&addr).await {
                Ok(listener) => {
                    let actual_port = listener
                        .local_addr()
                        .map(|addr| addr.port())
                        .unwrap_or_else(|_| preferred_port);
                    let mut callback = parsed.clone();
                    let _ = callback.set_port(Some(actual_port));
                    let callback_url = callback.to_string();
                    std::env::set_var("LASTFM_CALLBACK_URL", &callback_url);
                    eprintln!("[auth] listening on callback URL {}", callback_url);
                    return Ok((listener, callback_url));
                }
                Err(err) => {
                    eprintln!(
                        "[auth] failed to bind callback listener to {}: {:?}",
                        addr, err
                    );
                }
            }
        }

        let message = format!(
            "No free localhost callback port is available. Stop the stale listener or set LASTFM_CALLBACK_URL to a free port and update the Last.fm app callback.",
        );
        Err(AuthError::CallbackServer(message))
    }

    fn open_url_in_browser(&self, url: &str) -> Result<(), AuthError> {
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(url)
                .status()
                .map_err(|e| AuthError::CallbackServer(format!("Failed to open browser: {}", e)))?;
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/C", "start", "", url])
                .status()
                .map_err(|e| AuthError::CallbackServer(format!("Failed to open browser: {}", e)))?;
            return Ok(());
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Command::new("xdg-open")
                .arg(url)
                .status()
                .map_err(|e| AuthError::CallbackServer(format!("Failed to open browser: {}", e)))?;
            return Ok(());
        }
    }

    pub async fn request_token(&self) -> Result<String, AuthError> {
        eprintln!("[auth] requesting Last.fm token");
        let mut params = HashMap::new();
        params.insert("method".to_string(), "auth.getToken".to_string());

        let response = self
            .client
            .post_signed::<serde_json::Value>(params)
            .await
            .map_err(|e| {
                eprintln!("[auth] auth.getToken request failed: {:?}", e);
                AuthError::ClientError(e)
            })?;

        let token = response["token"]
            .as_str()
            .map(|value| value.to_string())
            .ok_or_else(|| {
                eprintln!(
                    "[auth] auth.getToken response missing token field: {}",
                    response
                );
                AuthError::AuthFailed
            })?;

        eprintln!("[auth] received token from Last.fm");
        Ok(token)
    }

    pub async fn start_auth_flow(&self) -> Result<Session, AuthError> {
        eprintln!("[auth] START auth flow");
        let token = match self.request_token().await {
            Ok(token) => token,
            Err(err) => {
                eprintln!("[auth] request_token failed: {:?}", err);
                return Err(err);
            }
        };
        eprintln!("[auth] token acquired: {}", token);

        let callback_url = Self::callback_url();
        eprintln!(
            "[auth] starting local callback listener for {}",
            callback_url
        );
        let (listener, actual_callback_url) = match Self::bind_callback_listener().await {
            Ok(result) => result,
            Err(err) => {
                eprintln!("[auth] callback listener setup failed: {:?}", err);
                return Err(err);
            }
        };
        eprintln!(
            "[auth] bound callback listener successfully; actual callback URL = {}",
            actual_callback_url
        );

        let auth_url = self.get_auth_url_for_token_with_callback(&token, &actual_callback_url);
        eprintln!("[auth] opening browser for Last.fm authorization");
        eprintln!("[auth] auth URL = {}", auth_url);
        if let Err(err) = self.open_url_in_browser(&auth_url) {
            eprintln!("[auth] browser open failed: {:?}", err);
            return Err(err);
        }
        eprintln!(
            "[auth] browser launched; waiting for callback request on {}",
            actual_callback_url
        );

        let session = tokio::time::timeout(Duration::from_secs(300), async {
            let mut poll_interval = tokio::time::interval(Duration::from_secs(2));

            loop {
                tokio::select! {
                    _ = poll_interval.tick() => {
                        eprintln!("[auth] polling auth.getSession while waiting for Last.fm approval");
                        match self.create_session(&token).await {
                            Ok(session) => {
                                eprintln!("[auth] auth.getSession succeeded for user {}", session.name);
                                break Ok::<Session, AuthError>(session);
                            }
                            Err(err) => {
                                eprintln!("[auth] auth.getSession not ready yet: {:?}", err);
                            }
                        }
                    }
                    accepted = listener.accept() => {
                        let (mut stream, remote_addr) = accepted.map_err(|e| {
                            eprintln!("[auth] callback accept failed: {:?}", e);
                            AuthError::CallbackServer(e.to_string())
                        })?;
                        eprintln!("[auth] accepted inbound connection from {:?}", remote_addr);

                        let mut buffer = [0u8; 4096];
                        let bytes_read = stream.read(&mut buffer).await.map_err(|e| {
                            eprintln!("[auth] callback read failed: {:?}", e);
                            AuthError::CallbackServer(e.to_string())
                        })?;
                        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                        eprintln!("[auth] raw callback request bytes read = {}", bytes_read);
                        eprintln!("[auth] raw callback request = {}", request.replace('\r', "\\r").replace('\n', "\\n"));

                        let first_line = request.lines().next().unwrap_or("<empty request>");
                        eprintln!("[auth] callback first line = {}", first_line);

                        if let Some(parsed_token) = self.extract_callback_token(&request) {
                            eprintln!("[auth] callback token extracted = {}", parsed_token);
                            if parsed_token != token {
                                eprintln!("[auth] callback token mismatch: expected current token '{}' but got '{}'", token, parsed_token);
                                return Err(AuthError::InvalidCallback);
                            }

                            let html = "<html><body><h1>Scrobblist</h1><p>Authentication complete. You can return to Scrobblist.</p></body></html>";
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                html.len(),
                                html
                            );
                            eprintln!("[auth] writing success response to callback client");
                            stream.write_all(response.as_bytes()).await.map_err(|e| {
                                eprintln!("[auth] callback response write failed: {:?}", e);
                                AuthError::CallbackServer(e.to_string())
                            })?;
                            stream.shutdown().await.map_err(|e| {
                                eprintln!("[auth] callback shutdown failed: {:?}", e);
                                AuthError::CallbackServer(e.to_string())
                            })?;
                            break self.create_session(&parsed_token).await;
                        }

                        let html = "<html><body><h1>Scrobblist authorization</h1><p>Return to Scrobblist after approving access in Last.fm.</p></body></html>";
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            html.len(),
                            html
                        );
                        eprintln!("[auth] callback had no token; keeping auth flow alive for token polling");
                        stream.write_all(response.as_bytes()).await.map_err(|e| {
                            eprintln!("[auth] callback response write failed: {:?}", e);
                            AuthError::CallbackServer(e.to_string())
                        })?;
                        stream.shutdown().await.map_err(|e| {
                            eprintln!("[auth] callback shutdown failed: {:?}", e);
                            AuthError::CallbackServer(e.to_string())
                        })?;
                    }
                }
            }
        })
        .await
        .map_err(|_| {
            eprintln!("[auth] callback wait timed out after 300s");
            AuthError::Timeout
        })??;

        eprintln!("[auth] auth flow completed; saving Last.fm session");

        if let Err(err) = self.save_session(&session.name, &session.key) {
            eprintln!("[auth] failed to save session to secure storage: {:?}", err);
            return Err(err);
        }

        eprintln!(
            "[auth] session stored successfully for user {}",
            session.name
        );
        Ok(session)
    }

    fn extract_callback_token(&self, request: &str) -> Option<String> {
        let first_line = request.lines().next()?;
        let path = first_line.split_whitespace().nth(1)?;
        let callback_base =
            std::env::var("LASTFM_CALLBACK_URL").unwrap_or_else(|_| CALLBACK_URL.to_string());
        let target = if path.starts_with("http") {
            path.to_string()
        } else {
            let trimmed = callback_base.trim_end_matches('/');
            if path.starts_with('/') {
                format!("{}{}", trimmed, path)
            } else {
                format!("{}/{}", trimmed, path)
            }
        };

        let parsed = url::Url::parse(&target).ok()?;
        parsed
            .query_pairs()
            .find_map(|(key, value)| (key == "token").then_some(value.into_owned()))
    }

    pub async fn create_session(&self, token: &str) -> Result<Session, AuthError> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "auth.getSession".to_string());
        params.insert("token".to_string(), token.to_string());

        let response = self.client.post_signed::<serde_json::Value>(params).await?;
        let session: Session = serde_json::from_value(response["session"].clone())
            .map_err(|_| AuthError::AuthFailed)?;

        Ok(session)
    }

    pub fn save_session(&self, username: &str, session_key: &str) -> Result<(), AuthError> {
        let username_entry = Entry::new(SERVICE_NAME, USERNAME_KEY)?;
        username_entry.set_password(username)?;

        let session_entry = Entry::new(SERVICE_NAME, SESSION_KEY_KEY)?;
        session_entry.set_password(session_key)?;

        Ok(())
    }

    pub fn retrieve_session(&self) -> Result<(String, String), AuthError> {
        let username_entry = Entry::new(SERVICE_NAME, USERNAME_KEY)?;
        let username = username_entry
            .get_password()
            .map_err(|_| AuthError::NoSession)?;

        let session_entry = Entry::new(SERVICE_NAME, SESSION_KEY_KEY)?;
        let session_key = session_entry
            .get_password()
            .map_err(|_| AuthError::NoSession)?;

        Ok((username, session_key))
    }

    pub fn clear_session(&self) -> Result<(), AuthError> {
        let username_entry = Entry::new(SERVICE_NAME, USERNAME_KEY)?;
        username_entry.delete_credential()?;

        let session_entry = Entry::new(SERVICE_NAME, SESSION_KEY_KEY)?;
        session_entry.delete_credential()?;

        Ok(())
    }
}
