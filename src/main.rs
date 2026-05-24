use anyhow::{Context, Result, bail};
use chrono::Local;
use json::JsonValue;
use json::object;
use reqwest::header::CONTENT_TYPE;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::fs::{File, create_dir_all};
use std::process::ExitCode;

fn init_logging() -> Result<()> {
    let now = Local::now();
    let log_dir = format!("logs/{}", now.format("%Y%m%d"));
    let log_path = format!("{log_dir}/chat_{}.log", now.format("%H%M%S"));

    create_dir_all(&log_dir).context("failed to create logs directory")?;

    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create(&log_path).context("failed to create log file")?,
    )
    .context("failed to initialize logger")?;

    Ok(())
}

fn call(body: &JsonValue) -> Result<JsonValue> {
    const CHAT_COMPLETIONS_URL: &str = "https://api.deepseek.com/chat/completions";

    log::debug!("request url: {CHAT_COMPLETIONS_URL}");
    log::debug!("request body: {}", body.dump());

    let api_key = env::var("DEEPSEEK_API_KEY").context("DEEPSEEK_API_KEY not set")?;
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(CHAT_COMPLETIONS_URL)
        .bearer_auth(api_key)
        .header(CONTENT_TYPE, "application/json")
        .body(body.dump())
        .send()
        .with_context(|| format!("failed to send request to {CHAT_COMPLETIONS_URL}"))?;

    let status = response.status();
    let text = response.text().context("failed to read response body")?;

    if status.is_success() {
        log::debug!("response status: {status}");
        log::debug!("response body: {text}");
    } else {
        log::error!("request url: {CHAT_COMPLETIONS_URL}");
        log::error!("response status: {status}");
        log::error!("response body: {text}");
        bail!("DeepSeek returned non-success status {status} for {CHAT_COMPLETIONS_URL}");
    }

    json::parse(&text).context("failed to parse response JSON")
}

fn run() -> Result<()> {
    dotenv::dotenv().ok();
    init_logging()?;
    log::info!("agent started");

    let body = object! {
        model: "deepseek-v4-flash",
        messages: [
          {
            role: "system",
            content: "You are a helpful assistant name Gugu. Introduce yourself.",
          },
          {
            role: "user",
            content: "hi!",
          }
        ]
    };
    let response = call(&body)?;

    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .context("response did not contain choices[0].message.content")?;

    println!("{content}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error:?}");
            eprintln!("The chat request failed. Details were written to the log file.");
            ExitCode::FAILURE
        }
    }
}
