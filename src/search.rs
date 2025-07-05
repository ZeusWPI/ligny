use std::{fs::File, io::Write, ops::Deref, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    config::Config,
    reader::{READS, ThreadNode},
    render::get_root,
};

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
    let reads = READS.lock().unwrap();
    let root = get_root(&reads)?;

    let mut index = Index { pages: vec![] };
    for (loc, node) in reads.iter() {
        let page = match node.lock().unwrap().deref() {
            ThreadNode::Section(section) => section.body.clone(),
            ThreadNode::Page(page) => page.clone(),
        };

        let html = page.render(&root)?;
        index.pages.push(Page {
            html: html.clone(),
            url: loc.url().clone(),
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
