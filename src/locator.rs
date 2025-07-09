use std::fmt::Display;
use std::path::Path;
use std::{cmp::Eq, path::PathBuf};

use crate::config::Config;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Locator {
    components: Vec<String>,
}

use anyhow::{Result, anyhow};

impl Locator {
    pub fn new(base: &str) -> Self {
        let components = base
            .split("/")
            .map(String::from)
            .filter(|c| !c.is_empty() && c != "index.md")
            .map(|c| {
                c.split_once('_')
                    .map(|(_, e)| e.replace(".md", ""))
                    .unwrap_or(c)
            })
            .collect::<Vec<String>>();

        Locator { components }
    }

    pub fn from_url(url: &str) -> Self {
        let components: Vec<String> = url
            .split("/")
            .filter(|e| !e.is_empty())
            .map(|e| e.to_owned())
            .collect();
        Locator { components }
    }

    pub fn from_content_path(path: &Path) -> Result<Self> {
        let abs = path.canonicalize().unwrap_or(path.into());
        let stripped = abs.strip_prefix(Config::get().content.canonicalize()?);
        let components: Vec<String> = stripped
            .iter()
            .filter_map(|component| component.to_str().map(String::from))
            .filter(|c| !c.is_empty() && c != "index.md")
            .map(|c| {
                c.split_once('_')
                    .ok_or_else(|| {
                        anyhow!(
                            "Section or Filename does not contain an '_' separator: '{}'",
                            c
                        )
                    })
                    .map(|(_, e)| e.replace(".md", ""))
            })
            .collect::<Result<Vec<String>>>()?;
        Ok(Locator { components })
    }

    pub fn root() -> Result<Self> {
        Locator::from_content_path(&Config::get().content)
    }

    pub fn join(&self, other: &Locator) -> Self {
        let mut locator_new = self.clone();
        for component in other.clone().components {
            match component.as_str() {
                ".." => locator_new = locator_new.parent(),
                "." => (),
                _ => locator_new.components.push(component),
            }
        }
        locator_new
    }

    pub fn parent(&self) -> Self {
        let mut locator_new = self.clone();
        locator_new.components.pop();
        locator_new
    }

    pub fn url(&self) -> String {
        let mut url = self.components.join("/");
        url.insert(0, '/');
        url
    }

    pub fn public_path(&self) -> String {
        let path = Locator::new(&Config::get().public)
            .join(self)
            .join(&Locator::new("index.html"));
        path.components.join("/")
    }

    pub fn public_dir(&self) -> String {
        let path = Locator::new(&Config::get().public).join(self);
        path.components.join("/")
    }

    pub fn static_path(&self) -> PathBuf {
        Config::get().static_dir.join(self.components.join("/"))
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.components.join("/"))
    }
}
