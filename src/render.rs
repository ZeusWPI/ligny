use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::Write,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use askama::Template;

use crate::{
    config::Config,
    locator::Locator,
    reader::{Node, Page, READS, Section, ThreadNode, ThreadNodeType, ThreadSection, read},
    search::write_index,
    templates::{BaseTemplate, ContentTableTemplate},
};

impl Page {
    pub fn render(&self, root: &Section) -> Result<String> {
        let content_table = ContentTableTemplate {
            root,
            curr_page: self,
        };

        let html = BaseTemplate {
            table: &content_table,
            page: self,
        }
        .render()
        .with_context(|| {
            format!(
                "Failed to render template of '{}' for location: '{}'",
                self.title, self.loc
            )
        })?;

        println!("Rendered page {} for url {}", self.title, self.loc.url());

        Ok(html)
    }
}

pub fn get_root(reads: &HashMap<Locator, ThreadNodeType>) -> Result<Section> {
    let root: Node = reads
        .get(&Locator::root()?)
        .context("Could not retrieve root section")?
        .into();

    let root: Section = match root {
        Node::Section(section) => section,
        Node::Page(_) => todo!(),
    };

    Ok(root)
}

/// write all rendered pages to files
pub fn write_pages_to_files() -> Result<()> {
    let reads = READS.lock().unwrap();

    let root = get_root(&reads)?;

    for (loc, node) in reads.iter() {
        let page = match node.lock().unwrap().deref() {
            ThreadNode::Section(section) => section.body.clone(),
            ThreadNode::Page(page) => page.clone(),
        };

        let html = page.render(&root)?;
        create_dir_all(loc.public_dir()).with_context(|| {
            format!("Failed to create all dirs for path: '{}'", loc.public_dir())
        })?;

        let mut file = File::create(loc.public_path())
            .with_context(|| format!("Failed to create file: '{}'", loc.public_dir()))?;

        file.write_all(html.as_bytes())
            .with_context(|| format!("Failed to write html to file: '{}'", loc.public_dir()))?;

        println!("Build page {} to {}", loc.url(), loc.public_path());
    }

    write_index(&reads)?;

    Ok(())
}

/// read all files in the content directory and render them using templates to memory
pub fn read_files() -> Result<()> {
    let mut reads = READS.lock().unwrap();
    let root: ThreadSection = read(
        Path::new(&Config::get().content),
        &Locator::new(""),
        &mut reads,
    )?;

    let _ = reads.insert(
        Locator::root()?,
        Arc::new(Mutex::new(ThreadNode::Section(root))),
    );
    Ok(())
}
