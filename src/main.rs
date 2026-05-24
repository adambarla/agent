use anyhow::{Context, Result, bail};
use chrono::Local;
use json::JsonValue;
use reqwest::header::CONTENT_TYPE;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::fs::{File, create_dir_all};
use std::io;
use std::io::Write;
use std::process::ExitCode;

mod constants;

use constants::SYSTEM_PROMPT;

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

fn call(body: String) -> Result<JsonValue> {
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

fn get_input() -> Result<String> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read user input")?;

    while input.ends_with('\n') || input.ends_with('\r') {
        input.pop();
    }

    Ok(input)
}

fn build_chat_request(messages: &JsonValue) -> String {
    format!(
        r#"{{"model":"deepseek-v4-flash","messages":{}}}"#,
        messages.dump()
    )
}

fn run() -> Result<()> {
    dotenv::dotenv().ok();
    init_logging()?;
    log::info!("agent started");

    let mut messages = JsonValue::new_array();
    let mut system_message = JsonValue::new_object();
    system_message["role"] = "system".into();
    system_message["content"] = SYSTEM_PROMPT.into();
    messages
        .push(system_message)
        .context("failed to add system message")?;

    loop {
        print!("You: ");
        io::stdout().flush().context("failed to flush stdout")?;

        let user_input = get_input()?;
        if user_input == "exit" {
            break;
        }

        let mut user_message = JsonValue::new_object();
        user_message["role"] = "user".into();
        user_message["content"] = user_input.into();
        messages
            .push(user_message)
            .context("failed to add user message")?;

        let body = build_chat_request(&messages);
        let response = call(body)?;

        let assistant_message = response["choices"][0]["message"].clone();
        let content = assistant_message["content"]
            .as_str()
            .context("response did not contain choices[0].message.content")?;

        println!("Assistant: {content}");
        messages
            .push(assistant_message)
            .context("failed to add assistant message")?;
    }
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
