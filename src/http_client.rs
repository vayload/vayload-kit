use anyhow::{Context, Result};
use reqwest::blocking::{multipart, Client, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use std::{io, sync::Arc};
use thiserror::Error;

use crate::types::{ErrorResponse, JsonResponse};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("{message}")]
    Api { message: String, payload: ErrorResponse },
}

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
        if let Some(auth_fn) = &self.auth_fn {
            if let Some(token) = auth_fn() {
                return rb.bearer_auth(token);
            }
        }
        rb
    }

    pub fn get_raw(&self, path: &str) -> Result<Response, ClientError> {
        let request = self.client.get(self.url(path));
        let request = self.with_auth(request);

        let response = request.send()?;
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let body = response.text()?;

            let parsed: ErrorResponse = serde_json::from_str(&body).map_err(|e| ClientError::Serialization(e))?; // Manejo explícito si falla el parseo

            Err(ClientError::Api { message: parsed.error.message.clone(), payload: parsed })
        }
    }

    pub fn get<T>(&self, path: &str) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let request = self.client.get(self.url(path));
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    pub fn post<T, B>(&self, path: &str, body: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.post(self.url(path)).json(body);
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    #[allow(dead_code)]
    pub fn post_form<T, B>(&self, path: &str, form: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.post(self.url(path)).form(form);
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    pub fn post_multipart<T>(&self, path: &str, form: multipart::Form) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let request = self.client.post(self.url(path)).multipart(form);
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    #[allow(dead_code)]
    pub fn put<T, B>(&self, path: &str, body: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.put(self.url(path)).json(body);
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    #[allow(dead_code)]
    pub fn put_form<T, B>(&self, path: &str, form: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.put(self.url(path)).form(form);
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    #[allow(dead_code)]
    pub fn patch<T, B>(&self, path: &str, body: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.patch(self.url(path)).json(body);
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    #[allow(dead_code)]
    pub fn patch_form<T, B>(&self, path: &str, form: &B) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.client.patch(self.url(path)).form(form);
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    #[allow(dead_code)]
    pub fn delete<T>(&self, path: &str) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let request = self.client.delete(self.url(path));
        let request = self.with_auth(request);
        let response = request.send()?;

        Self::parse_json(response)
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn parse_json<T>(response: Response) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let body = response.text()?;

        if status.is_success() {
            if let Ok(wrapped) = serde_json::from_str::<JsonResponse<T>>(&body) {
                return Ok(wrapped.data);
            }

            if let Ok(direct) = serde_json::from_str::<T>(&body) {
                return Ok(direct);
            }

            let data = serde_json::from_str::<T>(&body).map_err(ClientError::Serialization)?;

            Ok(data)
        } else {
            let parsed: ErrorResponse = serde_json::from_str(&body)?;
            Err(ClientError::Api { message: parsed.error.message.clone(), payload: parsed })
        }
    }
}
