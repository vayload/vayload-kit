use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use colored::Colorize;
use dialoguer::{Input, Password};
use rand::distr::Alphanumeric;
use rand::{RngExt, rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Arc;
use url::Url;

use crate::credentials_manager::{CredentialManager, RawCredentials};
use crate::http_client::HttpClient;

#[derive(Debug, Clone, Default)]
pub enum ClientType {
    #[default]
    Web,
    Cli,
}

impl ClientType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "web" => Ok(ClientType::Web),
            "cli" => Ok(ClientType::Cli),
            "" => Ok(ClientType::Web),
            _ => anyhow::bail!("Invalid client type"),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct OAuthState {
    pub origin_uri: String,
    pub redirect_uri: Option<String>,
    pub code_challenge: String,
    pub state: String,
    pub client_type: ClientType,
}

impl OAuthState {
    pub fn from_base64(s: &str) -> Result<Self> {
        let bytes = URL_SAFE_NO_PAD.decode(s)?;

        let decoded = String::from_utf8(bytes)?;
        let parts: Vec<&str> = decoded.split('|').collect();

        if parts.len() != 5 {
            return Err(anyhow::anyhow!("Invalid state format"));
        }

        Ok(Self {
            state: parts[0].to_string(),
            origin_uri: parts[1].to_string(),
            redirect_uri: if parts[2].is_empty() {
                None
            } else {
                Some(parts[2].to_string())
            },
            code_challenge: parts[3].to_string(),
            client_type: ClientType::from_str(parts[4])?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct LoginPasswordRequest {
    pub username: String,
    pub password: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct LoginPasswordResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct OAuthAuthorizationResponse {
    pub authorization_uri: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthDataResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,
    pub provider_id: String,
}

const CALLBACK_PATH: &str = "/callback";
const CALLBACK_PORT: u16 = 53682;

pub struct AuthCommands {
    cm: Arc<CredentialManager>,
    http_client: HttpClient,
}

impl AuthCommands {
    pub fn new(credentials_manager: Arc<CredentialManager>, http_client: HttpClient) -> Self {
        AuthCommands { cm: credentials_manager, http_client }
    }

    /// Login with username and password
    pub fn login_with_password(&self, username: Option<String>, password: Option<String>) -> Result<()> {
        let username = match username {
            Some(u) => u,
            None => Input::new().with_prompt("Username").interact_text().context("Failed to read username")?,
        };

        let password = match password {
            Some(p) => p,
            None => Password::new().with_prompt("Password").interact().context("Failed to read password")?,
        };

        println!("{}", "🔐 Authenticating...".cyan());

        let login_response = self
            .http_client
            .post::<LoginPasswordResponse, _>("/auth/login", &LoginPasswordRequest { username, password })?;

        let credentials = RawCredentials::new(
            login_response.access_token.clone(),
            login_response.refresh_token.clone(),
            login_response.expires_in as u64,
        );

        self.cm.store_tokens(credentials)?;

        println!("{}", "✓ Login successful!".green().bold());

        Ok(())
    }

    /// Login with OAuth (Google or GitHub)
    /// The server handles all OAuth logic, we just open the browser and receive the callback
    pub fn login_with_oauth(&self, provider: &str) -> Result<()> {
        println!("{} Starting OAuth login with {}...", "🔐".bold(), provider.cyan());

        // Start the server to listen for the callback
        let listener = match TcpListener::bind(format!("localhost:{CALLBACK_PORT}")) {
            Ok(listener) => listener,
            Err(_) => {
                println!(
                    "Port {} is already in use. Please close the conflicting app or try again.",
                    CALLBACK_PORT
                );
                return Err(anyhow::anyhow!("Port {} is already in use", CALLBACK_PORT));
            },
        };

        let callback_url = format!("http://localhost:{CALLBACK_PORT}{CALLBACK_PATH}");

        let state = self.random_string(16);
        let code_verifier = self.random_string(64);

        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        let request_url = format!("auth/oauth/{provider}");
        let request_body = serde_json::json!({
            "state": state,
            "code_challenge": code_challenge,
            "redirect_uri": callback_url,
            "origin_uri": "http://localhost:8080",
            "client_type": "cli"
        });

        let auth_response = self.http_client.post::<OAuthAuthorizationResponse, _>(&request_url, &request_body)?;

        println!("\n{}", "Opening browser for authentication...".cyan());

        if let Err(e) = open::that(&auth_response.authorization_uri) {
            eprintln!("{} Failed to open browser: {}", "⚠".yellow(), e);
            println!(
                "{}: {}",
                "Please open the URL manually".yellow(),
                auth_response.authorization_uri.bright_blue()
            );
        }

        println!("{}", "Waiting for authorization...".cyan());

        let (code, state) = self.receive_oauth_callback(&listener, &state)?;

        println!("{}", "✓ Authorization received!".green());
        println!("{}", "Exchanging code for tokens...".cyan());

        let oauth_url = format!("auth/oauth/{provider}/exchange");
        let oauth_body = OAuthCallbackRequest { code, state };

        let oauth_response = self.http_client.post::<OAuthDataResponse, _>(&oauth_url, &oauth_body)?;

        self.cm
            .store_tokens(RawCredentials {
                access_token: oauth_response.access_token.clone(),
                access_expires_in: oauth_response.expires_in,
                refresh_token: oauth_response.refresh_token,
                refresh_expires_in: oauth_response.expires_in,
            })
            .context("Failed to store tokens in keyring")?;

        println!("{}", "✓ OAuth login successful!".green().bold());

        Ok(())
    }

    fn random_string(&self, len: usize) -> String {
        rng().sample_iter(&Alphanumeric).take(len).map(char::from).collect()
    }

    fn receive_oauth_callback(&self, listener: &TcpListener, expected_state: &str) -> Result<(String, String)> {
        listener.set_nonblocking(true).context("Failed to set non-blocking mode")?;

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(120);

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("OAuth login timed out after 120 seconds");
            }

            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut reader = BufReader::new(&stream);
                    let mut request_line = String::new();

                    reader.read_line(&mut request_line).context("Failed to read OAuth callback request")?;

                    let path = request_line.split_whitespace().nth(1).context("Invalid HTTP request format")?;

                    if !path.starts_with(CALLBACK_PATH) {
                        self.send_error_response(&mut stream, "Invalid callback path")?;
                        anyhow::bail!("Invalid callback path");
                    }

                    let full_url = format!("http://localhost{}", path);
                    let parsed = Url::parse(&full_url).context("Failed to parse callback URL")?;

                    let mut code = None;
                    let mut state = None;

                    for (key, value) in parsed.query_pairs() {
                        match key.as_ref() {
                            "code" => code = Some(value.to_string()),
                            "state" => state = Some(value.to_string()),
                            "error" => {
                                self.send_error_response(&mut stream, &value)?;
                                anyhow::bail!("OAuth error: {}", value);
                            },
                            _ => {},
                        }
                    }

                    let code = code.context("No authorization code received")?;
                    let state_str = state.context("No state parameter received")?;
                    let state = OAuthState::from_base64(&state_str)?;

                    if expected_state != state.state {
                        self.send_error_response(&mut stream, "State mismatch - possible CSRF attack")?;
                        anyhow::bail!("State mismatch - possible CSRF attack");
                    }

                    self.send_success_response(&mut stream)?;
                    return Ok((code, state_str));
                },

                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                },

                Err(e) => return Err(e.into()),
            }
        }
    }

    /// Send success HTML response to browser
    fn send_success_response(&self, stream: &mut std::net::TcpStream) -> Result<()> {
        let response = "HTTP/1.1 200 OK\r\n\
            Content-Type: text/html; charset=utf-8\r\n\
            \r\n\
            <!DOCTYPE html>\
            <html lang='en'>\
            <head>\
                <meta charset='UTF-8'>\
                <title>Authentication Successful</title>\
                <style>\
                    :root {\
                        --color-vayload-bg: #050505;\
                        --color-vayload-bg-light: #0a0a0a;\
                        --color-vayload-bg-card: #0f0f10;\
                        --color-vayload-bg-elevated: #121214;\
                        --color-accent: #ff9800;\
                        --color-accent-light: #ffc947;\
                        --color-text: #ffffff;\
                        --color-text-muted: #bbbbbb;\
                    }\
                    body {\
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;\
                        background-color: var(--color-vayload-bg);\
                        color: var(--color-text);\
                        text-align: center;\
                        padding: 50px;\
                    }\
                    .container {\
                        background-color: var(--color-vayload-bg-card);\
                        padding: 40px;\
                        border-radius: 12px;\
                        box-shadow: 0 4px 20px rgba(0,0,0,0.5);\
                        max-width: 500px;\
                        margin: 0 auto;\
                    }\
                    h1 {\
                        color: var(--color-accent);\
                        margin: 0 0 20px 0;\
                    }\
                    .icon {\
                        font-size: 64px;\
                        margin-bottom: 20px;\
                        color: var(--color-accent-light);\
                    }\
                    p {\
                        color: var(--color-text-muted);\
                        margin: 10px 0;\
                    }\
                </style>\
            </head>\
            <body>\
                <div class='container'>\
                    <div class='icon'>✓</div>\
                    <h1>Authentication Successful!</h1>\
                    <p>You can close this window and return to the terminal.</p>\
                </div>\
            </body>\
            </html>";

        stream.write_all(response.as_bytes()).context("Failed to send success response")?;
        Ok(())
    }

    /// Send error HTML response to browser
    fn send_error_response(&self, stream: &mut std::net::TcpStream, error: &str) -> Result<()> {
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\n\
            Content-Type: text/html; charset=utf-8\r\n\
            \r\n\
            <!DOCTYPE html>\
            <html lang='en'>\
            <head>\
                <meta charset='UTF-8'>\
                <title>Authentication Failed</title>\
                <style>\
                    :root {{\
                        --color-vayload-bg: #050505;\
                        --color-vayload-bg-light: #0a0a0a;\
                        --color-vayload-bg-card: #0f0f10;\
                        --color-vayload-bg-elevated: #121214;\
                        --color-accent: #ff9800;\
                        --color-accent-light: #ffc947;\
                        --color-error: #ff5722;\
                        --color-text: #ffffff;\
                        --color-text-muted: #bbbbbb;\
                    }}\
                    body {{\
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;\
                        background-color: var(--color-vayload-bg);\
                        color: var(--color-text);\
                        text-align: center;\
                        padding: 50px;\
                    }}\
                    .container {{\
                        background-color: var(--color-vayload-bg-card);\
                        padding: 40px;\
                        border-radius: 12px;\
                        box-shadow: 0 4px 20px rgba(0,0,0,0.5);\
                        max-width: 500px;\
                        margin: 0 auto;\
                    }}\
                    h1 {{\
                        color: var(--color-error);\
                        margin: 0 0 20px 0;\
                    }}\
                    .icon {{\
                        font-size: 64px;\
                        margin-bottom: 20px;\
                        color: var(--color-error);\
                    }}\
                    p {{\
                        color: var(--color-text-muted);\
                        margin: 10px 0;\
                    }}\
                    .error {{\
                        color: var(--color-error);\
                        font-weight: bold;\
                    }}\
                </style>\
            </head>\
            <body>\
                <div class='container'>\
                    <div class='icon'>✗</div>\
                    <h1>Authentication Failed</h1>\
                    <p class='error'>{}</p>\
                    <p>Please close this window and try again.</p>\
                </div>\
            </body>\
            </html>",
            error
        );

        stream.write_all(response.as_bytes()).context("Failed to send error response")?;
        Ok(())
    }

    /// Get current user information
    pub fn whoami(&self) -> Result<()> {
        if !self.cm.is_authenticated() {
            return Err(anyhow::anyhow!(
                "Not authenticated. Please login first with 'vayload-kit auth -u <username> -p <password>' or 'vayload-kit auth -o <provider>'"
            ));
        }

        let whoami_response = self.http_client.get::<User>("/auth/me")?;

        println!("{}", "Current User:".green().bold());
        self.print_user_info(&whoami_response);

        Ok(())
    }

    /// Logout and clear stored tokens
    pub fn logout(&self) -> Result<()> {
        if !self.cm.is_authenticated() {
            println!("{}", "Already logged out".yellow());
            return Ok(());
        }

        self.cm.clear_all().context("Failed to clear tokens from keyring")?;

        println!("{}", "✓ Logged out successfully!".green().bold());
        println!("{}", "All tokens have been removed from keyring.".bright_black());

        Ok(())
    }

    /// Helper to print user information
    fn print_user_info(&self, user: &User) {
        println!("{} {}", "Username:".bright_black(), user.username.cyan());
        println!("{} {}", "Email:".bright_black(), user.email);

        if let Some(name) = &user.name {
            println!("{} {}", "Name:".bright_black(), name);
        }

        if let Some(avatar) = &user.avatar_url {
            println!("{} {}", "Avatar:".bright_black(), avatar.bright_black());
        }

        println!("{} {}", "Provider:".bright_black(), user.provider);
        println!("{} {}", "Provider ID:".bright_black(), user.provider_id);
    }
}
