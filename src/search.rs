use std::{collections::HashMap, fs::File, io::Write, ops::Deref, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    config::Config,
    locator::Locator,
    reader::{ThreadNode, ThreadNodeType},
};

#[derive(Serialize)]
pub struct Page {
    html: String,
    url: String,
    title: String,
}

type Index = Vec<Page>;

pub fn render_index(reads: &HashMap<Locator, ThreadNodeType>) -> Result<Index> {
    let mut index = vec![];
    for (loc, node) in reads.iter() {
        let page = match node.lock().unwrap().deref() {
            ThreadNode::Section(section) => section.body.clone(),
            ThreadNode::Page(page) => page.clone(),
        };

        index.push(Page {
            html: page.content.clone(),
            url: loc.url().clone(),
            title: page.title.clone(),
        });
    }

    Ok(index)
}

pub fn write_index(reads: &HashMap<Locator, ThreadNodeType>) -> Result<()> {
    let index = render_index(reads)?;
    let json = serde_json::to_string(&index)?;
    let path = Path::new(&Config::get().public).join(&Config::get().index_name);
    let mut file = File::create(&path)
        .with_context(|| format!("Failed to create file: '{}'", path.display()))?;

    file.write_all(json.as_bytes())
        .with_context(|| format!("Failed to write json to file: '{}'", path.display()))?;

    Ok(())
}
