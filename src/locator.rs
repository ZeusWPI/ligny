use std::fmt::Display;
use std::hash::Hash;
use std::path::Path;
use std::{cmp::Eq, path::PathBuf};

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct Locator {
    components: Vec<String>,
}

use anyhow::{Result, anyhow};

impl Locator {
    pub fn new(base: &str) -> Self {
        let components = base
            .split("/")
            .map(String::from)
            .filter(|c| !c.is_empty())
            .map(|c| c.split_once('_').map(|(_, e)| String::from(e)).unwrap_or(c))
            .map(|e| e.replace(".md", ""))
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
        let stripped = abs.strip_prefix(Config::get().content.canonicalize()?)?;
        let components: Vec<String> = stripped
            .iter()
            .filter_map(|component| component.to_str().map(String::from))
            .filter(|c| !c.is_empty())
            .map(|c| {
                c.split_once('_')
                    .map_or_else(
                        || {
                            if c == "index.md" {
                                Ok("index.md")
                            } else {
                                Err(anyhow!(
                                    "Section or Filename does not contain an '_' separator: '{}'",
                                    c
                                ))
                            }
                        },
                        |(_, e)| Ok(e),
                    )
                    .map(|e| e.replace(".md", ""))
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
        let mut url = self.join_components();
        url.insert(0, '/');
        url
    }

    fn join_components(&self) -> String {
        self.components
            .iter()
            .filter(|component| !component.eq(&"index"))
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn public_path(&self) -> PathBuf {
        let path = Locator::new(&Config::get().public)
            .join(self)
            .join(&Locator::new("index.html"));
        PathBuf::from(path.join_components())
    }

    pub fn public_dir(&self) -> PathBuf {
        let path = Locator::new(&Config::get().public).join(self);
        PathBuf::from(path.join_components())
    }

    pub fn static_path(&self) -> PathBuf {
        Config::get().static_dir.join(self.join_components())
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

impl Hash for Locator {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let url = self.url();
        url.hash(state);
    }
}

impl PartialEq for Locator {
    fn eq(&self, other: &Self) -> bool {
        self.url() == other.url()
    }
}

impl Eq for Locator {}
