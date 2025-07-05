mod config;
mod locator;
mod notify;
mod render;
mod search;
mod serve;
mod templates;

use anyhow::{Result, anyhow};
use config::Config;
use notify::spawn_watcher_thread;
use render::{write_pages_to_files, read_files_and_render_templates};
use search::build_index;
use serve::serve;

mod reader;
use std::env;

static BUILD_COMMAND: &str = "build";

#[tokio::main]
async fn main() -> Result<()> {
    Config::initialize();

    let args: Vec<String> = env::args().collect();

    let command = args.get(1).map(|a| a.as_str()).unwrap_or(BUILD_COMMAND);

    read_files_and_render_templates()?;
    build_index()?;

    match command {
        "build" => write_pages_to_files(),
        "serve" => {
            let handle = spawn_watcher_thread();
            let result = serve().await;
            let _ = handle.join();
            result
        }
        _ => Err(anyhow!(
            "Command '{}' not found. Use 'build' or 'serve'.",
            command
        )),
    }?;

    Ok(())
}
