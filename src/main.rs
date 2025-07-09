mod config;
mod init;
mod locator;
mod notify;
mod reader;
mod render;
mod search;
mod serve;
mod templates;

use std::env;

use anyhow::{Ok, Result, bail};
use config::Config;
use render::{read_files, write_pages_to_files};
use serve::serve;

use crate::init::init_files;

static BUILD_COMMAND: &str = "build";

#[tokio::main]
async fn main() -> Result<()> {
    Config::initialize();

    let args: Vec<String> = env::args().collect();

    let command = args.get(1).map(|a| a.as_str()).unwrap_or(BUILD_COMMAND);

    match command {
        "build" => {
            read_files()?;
            write_pages_to_files()
        }
        "serve" => {
            read_files()?;
            serve().await
        }
        "init" => init_files(),
        _ => bail!("Command '{}' not found. Use 'build' or 'serve'.", command),
    }?;

    Ok(())
}
