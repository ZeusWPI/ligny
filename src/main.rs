mod config;
mod errors;
mod render;
mod serve;
mod templates;

use std::path::Path;

use config::Config;
use errors::Error;
use reader::read;
use serve::serve;

mod reader;
use std::env;

static BUILD_COMMAND: &str = "build";

#[tokio::main]
async fn main() {
    Config::initialize();

    let args: Vec<String> = env::args().collect();

    let command = args.get(1).map(|a| a.as_str()).unwrap_or(BUILD_COMMAND);

    let out = match command {
        "build" => build(),
        "serve" => serve().await,
        _ => Err(Error::CommandNotFound),
    };

    println!("{:?}", out);
}

pub fn build() -> Result<(), errors::Error> {
    let section = read(Path::new(&Config::get().content), "");
    section.build(&section)
}
