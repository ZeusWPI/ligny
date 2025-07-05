use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::render::RENDERS;
use anyhow::{Context, Result};
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

pub fn build_index() -> Result<()> {
    let renders = RENDERS.lock().unwrap();

    let mut index = Index { pages: vec![] };
    for (url, html) in renders.iter() {
        index.pages.push(Page {
            html: html.clone(),
            url: url.clone(),
        });
    }

    let json = serde_json::to_string(&index)?;

    let path = Path::new(&Config::get().public).join(&Config::get().index_name);
    let mut file = File::create(&path)
        .with_context(|| format!("Failed to create file: '{}'", path.display()))?;

    file.write_all(json.as_bytes())
        .with_context(|| format!("Failed to write json to file: '{}'", path.display()))?;

    Ok(())
}
