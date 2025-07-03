use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use askama::Template;

use crate::{
    config::Config,
    errors::Error,
    reader::{Node, Page, Section},
    templates::{BaseTemplate, ContentTableTemplate},
};

impl Page {
    pub fn build(&self, root: &Section) -> Result<(), Error> {
        let content_table = ContentTableTemplate { root, page: self };

        let html = BaseTemplate {
            table: &content_table,
            page: self,
        }
        .render()
        .map_err(Error::from)?;

        let path =
            Path::new(&Config::get().public).join(self.url.strip_prefix("/").unwrap_or(&self.url));
        create_dir_all(&path).map_err(Error::from)?;

        let out_path = path.join("index.html");
        let mut file = File::create(&out_path).map_err(Error::from)?;

        file.write_all(html.as_bytes()).map_err(|e| Error::Build {
            message: e.to_string(),
        })?;

        println!(
            "Build page {} to {}",
            self.title,
            out_path.to_str().unwrap()
        );

        Ok(())
    }
}

impl Section {
    pub fn build(&self, root: &Section) -> Result<(), Error> {
        self.body.build(root)?;
        for child in self.children.iter() {
            child.build(root)?;
        }

        Ok(())
    }
}

impl Node {
    pub fn build(&self, root: &Section) -> Result<(), Error> {
        match self {
            Node::Section(section) => section.build(root),
            Node::Page(page) => page.build(root),
        }
    }
}
