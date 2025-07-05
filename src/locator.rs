use std::fmt::Display;

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct Locator {
    components: Vec<String>,
}

impl Locator {
    pub fn new(base: &str) -> Self {
        let components = if base.is_empty() {
            vec![]
        } else {
            vec![base.to_string()]
        };
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

    pub fn join(&self, other: &Locator) -> Self {
        let mut locator_new = self.clone();
        locator_new.components.append(&mut other.components.clone());
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
}

impl Display for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.components.join("/"))
    }
}
