mod config;
mod init;
mod link_checker;
mod locator;
mod reader;
mod render;
mod search;
mod serve;
mod templates;
mod watcher;

use std::{
    collections::HashMap,
    env,
    sync::{LazyLock, Mutex},
};

use anyhow::{Ok, Result, bail};
use config::Config;
use link_checker::check_links_root;
use locator::Locator;
use reader::ThreadNodeType;
use render::{read_files, write_pages_to_files};
use serve::serve;

use crate::init::init_files;

static BUILD_COMMAND: &str = "build";

pub struct Static {
    reads: HashMap<Locator, ThreadNodeType>,
}

pub static CONTEXT: LazyLock<Mutex<Static>> = LazyLock::new(|| {
    Mutex::new(Static {
        reads: HashMap::new(),
    })
});

#[tokio::main]
async fn main() -> Result<()> {
    Config::initialize();

    let args: Vec<String> = env::args().collect();

    let command = args.get(1).map(|a| a.as_str()).unwrap_or(BUILD_COMMAND);

    match command {
        "build" => {
            read_files()?;
            check_links_root()?;
            write_pages_to_files()
        }
        "serve" => {
            read_files()?;
            check_links_root()?;
            serve().await
        }
        "init" => init_files(),
        _ => bail!(
            "Command '{}' not found. Use 'init', 'build' or 'serve'.",
            command
        ),
    }?;

    Ok(())
}
