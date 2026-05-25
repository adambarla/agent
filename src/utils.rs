use anyhow::{Context, Result};
use chrono::Local;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::{File, create_dir_all};
use std::io;
use std::io::Write;

pub fn init_logging() -> Result<()> {
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

pub fn get_input() -> Result<String> {
    print!("You: ");
    io::stdout().flush().context("failed to flush stdout")?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read user input")?;

    while input.ends_with('\n') || input.ends_with('\r') {
        input.pop();
    }

    Ok(input)
}
