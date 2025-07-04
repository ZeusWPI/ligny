mod config;
mod errors;
mod locator;
mod notify;
mod render;
mod serve;
mod templates;

use std::path::Path;

use config::Config;
use errors::Error;
use locator::Locator;
use notify::spawn_watcher_thread;
use reader::{Node, READS, Section, read};
use render::build;
use serve::serve;

mod reader;
use std::env;

static BUILD_COMMAND: &str = "build";

#[tokio::main]
async fn main() {
    Config::initialize();

    let args: Vec<String> = env::args().collect();

    let command = args.get(1).map(|a| a.as_str()).unwrap_or(BUILD_COMMAND);

    println!("{:?}", render());

    let handle = spawn_watcher_thread();

    let out = match command {
        "build" => build(),
        "serve" => serve().await,
        _ => Err(Error::CommandNotFound),
    };

    let _ = handle.join();

    println!("{out:?}");
}

fn render() -> Result<(), Error> {
    let mut reads = READS.lock().unwrap();
    let root: Node = (&read(
        Path::new(&Config::get().content),
        &Locator::new(""),
        &mut reads,
    ))
        .into();

    match root {
        Node::Section(section) => {
            let root: Section = section;
            root.render(&root)
        }
        Node::Page(_) => Ok(()),
    }
}
