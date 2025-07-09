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
    CONTEXT,
    config::Config,
    locator::Locator,
    reader::{Page, Section, ThreadNode, ThreadNodeType, ThreadSection, read},
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
    let root = reads
        .get(&Locator::root()?)
        .with_context(|| "Could not retrieve root section")?;

    Ok(root.lock().unwrap().get_section()?.into())
}

/// write all rendered pages to files
pub fn write_pages_to_files() -> Result<()> {
    let context = CONTEXT.lock().unwrap();

    let root = get_root(&context.reads)?;

    for (loc, node) in context.reads.iter() {
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

        println!(
            "Build page {} to {}",
            loc.url(),
            loc.public_path().display()
        );
    }

    write_index(&context.reads)?;

    Ok(())
}

/// read all files in the content directory and render them using templates to memory
pub fn read_files() -> Result<()> {
    let mut context = CONTEXT.lock().unwrap();
    let root: ThreadSection = read(
        Path::new(&Config::get().content),
        &Locator::new(""),
        &mut context,
    )?;

    let _ = context.reads.insert(
        Locator::root()?,
        Arc::new(Mutex::new(ThreadNode::Section(root.clone()))),
    );

    Ok(())
}
