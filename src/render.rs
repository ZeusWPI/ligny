use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
    sync::Mutex,
};

use askama::Template;

use crate::{
    errors::Error,
    locator::{self, Locator},
    reader::{Node, Page, Section},
    templates::{BaseTemplate, ContentTableTemplate},
};

use std::sync::LazyLock;

pub static RENDERS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl Page {
    pub fn render(&self, root: &Section) -> Result<(), Error> {
        let content_table = ContentTableTemplate { root, page: self };

        let html = BaseTemplate {
            table: &content_table,
            page: self,
        }
        .render()
        .map_err(Error::from)?;

        let mut renders = RENDERS.lock().unwrap();
        renders.insert(self.loc.url(), html);

        println!("Rendered page {} for url {}", self.title, self.loc.url());

        Ok(())
    }
}

impl Section {
    pub fn render(&self, root: &Section) -> Result<(), Error> {
        self.body.render(root)?;
        for child in self.children.iter() {
            child.render(root)?;
        }

        Ok(())
    }
}

impl Node {
    pub fn render(&self, root: &Section) -> Result<(), Error> {
        match self {
            Node::Section(section) => section.render(root),
            Node::Page(page) => page.render(root),
        }
    }
}

pub fn build() -> Result<(), Error> {
    let renders = RENDERS.lock().unwrap();

    for (url, html) in renders.iter() {
        let loc = Locator::from_url(url);
        create_dir_all(loc.public_dir()).map_err(Error::from)?;

        let mut file = File::create(loc.public_path()).map_err(Error::from)?;

        file.write_all(html.as_bytes()).map_err(|e| Error::Build {
            message: e.to_string(),
        })?;

        println!("Build page {} to {}", url, loc.public_path());
    }

    Ok(())
}
