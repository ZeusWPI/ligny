mod config;
mod locator;
mod notify;
mod render;
mod search;
mod serve;
mod templates;

use anyhow::{Result, anyhow};
use config::Config;
use render::{read_files, write_pages_to_files};
use serve::serve;

mod reader;
use std::env;

static BUILD_COMMAND: &str = "build";

#[tokio::main]
async fn main() -> Result<()> {
    Config::initialize();

    let args: Vec<String> = env::args().collect();

    let command = args.get(1).map(|a| a.as_str()).unwrap_or(BUILD_COMMAND);

    read_files()?;

    match command {
        "build" => write_pages_to_files(),
        "serve" => serve().await,
        _ => Err(anyhow!(
            "Command '{}' not found. Use 'build' or 'serve'.",
            command
        )),
    }?;

    Ok(())
}
