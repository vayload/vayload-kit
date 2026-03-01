use anyhow::{Context, Result};
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response, multipart};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::logger;
use crate::types::{ErrorResponse, JsonResponse};

#[allow(unused)]
#[derive(Debug)]
pub enum ClientError {
    Network(reqwest::Error),

    Serialization {
        source: serde_json::Error,
        body: String,
    },

    Api {
        status: StatusCode,
        message: String,
        payload: Box<ErrorResponse>,
    },

    UnexpectedResponse {
        status: StatusCode,
        body: String,
        source: serde_json::Error,
    },
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Network(e) => write!(f, "Network error: {}", e),

            ClientError::Serialization { source, .. } => {
                write!(f, "Serialization error: {}", source)
            },

            ClientError::Api { status, message, .. } => {
                write!(f, "API error ({}): {}", status, message)
            },

            ClientError::UnexpectedResponse { status, .. } => {
                write!(f, "Unexpected response with status {}", status)
            },
        }
    }
}

impl std::error::Error for ClientError {}

type AuthFn = Arc<dyn Fn() -> Option<String> + Send + Sync>;

#[derive(Clone)]
pub struct HttpClient {
    base_url: String,
    client: Client,
    auth_fn: Option<AuthFn>,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let client =
            Client::builder().timeout(Duration::from_secs(240)).build().context("Failed to build HTTP client")?;

        Ok(Self { base_url: base_url.into(), client, auth_fn: None })
    }

    #[allow(dead_code)]
    pub fn new_with_token(base_url: impl Into<String>, token: String) -> Result<Self> {
        let client =
            Client::builder().timeout(Duration::from_secs(240)).build().context("Failed to build HTTP client")?;

        let token = Arc::new(token);
        let token_clone = token.clone();
        let auth_fn: AuthFn = Arc::new(move || Some(token_clone.to_string()));

        Ok(Self { base_url: base_url.into(), client, auth_fn: Some(auth_fn) })
    }

    pub fn set_auth_fn<F>(&mut self, f: F)
    where
        F: Fn() -> Option<String> + Send + Sync + 'static,
    {
        self.auth_fn = Some(Arc::new(f));
    }

    fn with_auth(&self, rb: reqwest::blocking::RequestBuilder) -> reqwest::blocking::RequestBuilder {
        if let Some(auth_fn) = &self.auth_fn
            && let Some(token) = auth_fn()
        {
            return rb.bearer_auth(token);
        }
        rb
    }

    pub fn get_raw(&self, path: &str) -> Result<Response, ClientError> {
        let request = self.client.get(self.url(path));
        let request = self.with_auth(request);

        let response = request.send().map_err(ClientError::Network)?;
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let body = response.text().map_err(ClientError::Network)?;

            let parsed: ErrorResponse = serde_json::from_str(&body)
                .map_err(|e| ClientError::Serialization { source: e, body: body.clone() })?;

            Err(ClientError::Api {
                status,
                message: parsed.error.message.clone(),
                payload: Box::new(parsed),
            })
        }
    }

    pub fn get<T>(&self, path: &str) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let request = self.client.get(self.url(path));
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    pub fn post<T, B>(&self, path: &str, body: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.post(self.url(path)).json(body);
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    #[allow(dead_code)]
    pub fn post_form<T, B>(&self, path: &str, form: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.post(self.url(path)).form(form);
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    pub fn post_multipart<T>(&self, path: &str, form: multipart::Form) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let request = self.client.post(self.url(path)).multipart(form);
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    #[allow(dead_code)]
    pub fn put<T, B>(&self, path: &str, body: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.put(self.url(path)).json(body);
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    #[allow(dead_code)]
    pub fn put_form<T, B>(&self, path: &str, form: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.put(self.url(path)).form(form);
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    #[allow(dead_code)]
    pub fn patch<T, B>(&self, path: &str, body: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.patch(self.url(path)).json(body);
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    #[allow(dead_code)]
    pub fn patch_form<T, B>(&self, path: &str, form: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.patch(self.url(path)).form(form);
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    #[allow(dead_code)]
    pub fn delete<T>(&self, path: &str) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let request = self.client.delete(self.url(path));
        let request = self.with_auth(request);
        let response = request.send().map_err(ClientError::Network)?;

        self.parse_json(response)
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub fn parse_json<T>(&self, response: Response) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();

        let body = match response.text() {
            Ok(text) => text,
            Err(err) => {
                logger::error(&format!("Network error while reading response body: {}", err));
                return Err(ClientError::Network(err));
            },
        };

        if status.is_success() {
            match self.try_parse_success::<T>(&body) {
                Ok(val) => Ok(val),
                Err(err) => {
                    logger::error(&format!(
                        "Serialization error parsing successful response: {} | body: {}",
                        err, body
                    ));

                    Err(ClientError::Serialization { source: err, body })
                },
            }
        } else {
            match serde_json::from_str::<ErrorResponse>(&body) {
                Ok(parsed) => {
                    logger::error(&format!("API error ({}): {}", status, parsed.error.message));
                    Err(ClientError::Api {
                        status,
                        message: parsed.error.message.clone(),
                        payload: Box::new(parsed),
                    })
                },
                Err(err) => {
                    logger::error(&format!(
                        "Unexpected response parsing error: {} | status: {} | body: {}",
                        err, status, body
                    ));
                    Err(ClientError::UnexpectedResponse { status, body, source: err })
                },
            }
        }
    }

    fn try_parse_success<T>(&self, body: &str) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str::<JsonResponse<T>>(body)
            .map(|wrapped| wrapped.data)
            .or_else(|_| serde_json::from_str::<T>(body))
    }
}
