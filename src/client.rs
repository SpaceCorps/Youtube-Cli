//! HTTP client for the Apify YouTube Scraper API, and translation from HTTP status to [`ErrorCode`].
//!
//! One blocking agent per process: a CLI makes a handful of requests, so an async runtime
//! would cost more in startup than it could save. Connections are pooled by the agent.

use std::time::Duration;

use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};

const DEFAULT_BASE: &str = "https://api.apify.com/v2/";
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    auth: String,
}

enum Method {
    Get,
    Post,
}

impl Client {
    pub fn new(api_key: &str) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(360)))
            .timeout_connect(Some(Duration::from_secs(30)))
            .http_status_as_error(false)
            .user_agent(concat!("youtube-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        // For pointing the CLI at a mock server in tests.
        let mut base = std::env::var("YOUTUBE_API_URL")
            .or_else(|_| std::env::var("APIFY_API_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, auth: format!("Bearer {api_key}") }
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Post, path, Some(body))
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{}", self.base, path.trim_start_matches('/'));

        macro_rules! headers {
            ($req:expr) => {{ $req.header("Authorization", &self.auth).header("Accept", "application/json") }};
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Post, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                let req = self.agent.post(&url);
                headers!(req).header("Content-Type", "application/json").send(&json[..])
            }
            (Method::Post, None) => {
                let req = self.agent.post(&url);
                headers!(req).send_empty()
            }
        };

        let response = result.map_err(transport_error)?;
        read(response)
    }
}

fn read(mut response: Response<ureq::Body>) -> Result<Value> {
    let status = response.status().as_u16();
    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Array(Vec::new()));
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the Apify API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    // Try extracting Apify error message from {"error": {"message": "..."}} or {"message": "..."}
    let message_from_body = serde_json::from_str::<Value>(body).ok().and_then(|v| {
        v.get("error")
            .and_then(|e| e.get("message"))
            .or_else(|| v.get("message"))
            .and_then(Value::as_str)
            .map(str::to_string)
    });

    let e = match status {
        401 => Error::new(
            ErrorCode::AuthRequired,
            message_from_body.unwrap_or_else(|| "The Apify API token was rejected.".into()),
        )
        .fix("Set APIFY_TOKEN or run: youtube login <name> --api-key <key>"),
        403 => Error::new(
            ErrorCode::AuthRequired,
            message_from_body.unwrap_or_else(|| "The Apify API token is not allowed to perform this action.".into()),
        )
        .fix("Verify token permissions at https://console.apify.com/account/integrations"),
        404 => Error::new(
            ErrorCode::NotFound,
            message_from_body.unwrap_or_else(|| "The requested Apify actor or resource does not exist.".into()),
        ),
        429 => Error::new(
            ErrorCode::RateLimited,
            message_from_body.unwrap_or_else(|| "Rate limited by the Apify API.".into()),
        )
        .fix("Back off before retrying."),
        400 | 422 => Error::new(
            ErrorCode::InvalidInput,
            message_from_body.unwrap_or_else(|| "The Apify API refused the request.".into()),
        ),
        s if s >= 500 => Error::new(
            ErrorCode::Network,
            message_from_body.unwrap_or_else(|| "The Apify API returned a server error.".into()),
        )
        .fix("Retry; if it persists the Apify service may be experiencing issues."),
        _ => Error::new(ErrorCode::Error, "The request failed."),
    };
    e.detail(detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(422, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(503, "").code, ErrorCode::Network);
        assert_eq!(status_error(400, "").code, ErrorCode::InvalidInput);
    }
}
