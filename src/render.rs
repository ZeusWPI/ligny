use std::{
    collections::HashMap, fs::{create_dir_all, File}, io::Write, path::Path, sync::Mutex
};

use anyhow::{Context, Result, anyhow};
use askama::Template;

use crate::{
    config::Config, locator::Locator, reader::{read, Node, Page, Section, READS}, templates::{BaseTemplate, ContentTableTemplate}
};

use std::sync::LazyLock;

pub static RENDERS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl Page {
    pub fn render(&self, root: &Section) -> Result<()> {
        let content_table = ContentTableTemplate { root, curr_page: self };

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

        let mut renders = RENDERS.lock().unwrap();
        renders.insert(self.loc.url(), html);

        println!("Rendered page {} for url {}", self.title, self.loc.url());

        Ok(())
    }
}

impl Section {
    pub fn render(&self, root: &Section) -> Result<()> {
        self.body.render(root)?;
        for child in self.children.iter() {
            child.render(root)?;
        }

        Ok(())
    }
}

impl Node {
    pub fn render(&self, root: &Section) -> Result<()> {
        match self {
            Node::Section(section) => section.render(root),
            Node::Page(page) => page.render(root),
        }
    }
}

/// write all rendered pages to files
pub fn write_pages_to_files() -> Result<()> {
    let renders = RENDERS.lock().unwrap();

    for (url, html) in renders.iter() {
        let loc = Locator::from_url(url);
        create_dir_all(loc.public_dir()).with_context(|| {
            format!("Failed to create all dirs for path: '{}'", loc.public_dir())
        })?;

        let mut file = File::create(loc.public_path())
            .with_context(|| format!("Failed to create file: '{}'", loc.public_dir()))?;

        file.write_all(html.as_bytes())
            .with_context(|| format!("Failed to write html to file: '{}'", loc.public_dir()))?;

        println!("Build page {} to {}", url, loc.public_path());
    }

    Ok(())
}

/// read all files in the content directory and render them using templates to memory
pub fn read_files_and_render_templates() -> Result<()> {
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
