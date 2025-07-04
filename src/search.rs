use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::errors::Error;
use crate::render::RENDERS;
use serde::Serialize;

#[derive(Serialize)]
struct Index {
    pages: Vec<Page>,
}

#[derive(Serialize)]
struct Page {
    html: String,
    url: String,
}

pub fn build_index() -> Result<(), Error> {
    let renders = RENDERS.lock().unwrap();

    let mut index = Index { pages: vec![] };
    for (url, html) in renders.iter() {
        index.pages.push(Page {
            html: html.clone(),
            url: url.clone(),
        });
    }

    let json = serde_json::to_string(&index)?;

    let mut file = File::create(Path::new(&Config::get().public).join(&Config::get().index_name))?;

    file.write_all(json.as_bytes()).map_err(|e| Error::Build {
        message: e.to_string(),
    })?;

    println!("Build index");

    Ok(())
}
