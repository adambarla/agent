use std::env;

use anyhow::{Context, Result, bail};
use json::JsonValue;
use reqwest::header::CONTENT_TYPE;

pub fn call(body: String) -> Result<JsonValue> {
    let chat_completions_url =
        env::var("CHAT_COMPLETIONS_URL").context("CHAT_COMPLETIONS_URL not set")?;
    let api_key = env::var("API_KEY").context("API_KEY not set")?;

    log::debug!("request url: {chat_completions_url}");
    log::debug!("request body: {body}");

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&chat_completions_url)
        .bearer_auth(api_key)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .with_context(|| format!("failed to send request to {}", chat_completions_url))?;

    let status = response.status();
    let text = response.text().context("failed to read response body")?;

    if status.is_success() {
        log::debug!("response status: {status}");
        log::debug!("response body: {text}");
    } else {
        log::error!("response status: {status}");
        log::error!("response body: {text}");
        bail!("DeepSeek returned non-success status {status} for {chat_completions_url}");
    }

    json::parse(&text).context("failed to parse response JSON")
}
