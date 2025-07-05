mod config;
mod errors;
mod locator;
mod notify;
mod render;
mod search;
mod serve;
mod templates;

use std::path::Path;

use anyhow::{Result, anyhow};
use config::Config;
use errors::Error;
use locator::Locator;
use notify::spawn_watcher_thread;
use reader::{Node, READS, Section, read};
use render::build;
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

    render()?;
    build_index()?;

    match command {
        "build" => build(),
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

fn render() -> anyhow::Result<()> {
    let mut reads = READS.lock().unwrap();
    let root: Node = (&read(
        Path::new(&Config::get().content),
        &Locator::new(""),
        &mut reads,
    )?)
        .into();

    match root {
        Node::Section(section) => {
            let root: Section = section;
            root.render(&root)
                .map_err(|e| anyhow!("Failed to render root with error: {e}"))
        }
        Node::Page(_) => Ok(()),
    }
}
