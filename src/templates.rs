use crate::reader::{Node::*, Page, Section};
use anyhow::{Context, Result};
use tera::{Context as TContext, Tera};

fn base_template() -> Tera {
    Tera::new("templates/base.html").unwrap()
}

fn content_table_template() -> Tera {
    Tera::new("templates/content_table.html").unwrap()
}

pub struct BaseTemplate<'a> {
    pub table: &'a ContentTableTemplate<'a>,
    pub page: &'a crate::reader::Page,
}

pub struct ContentTableTemplate<'a> {
    pub root: &'a crate::reader::Section,
    pub curr_page: &'a Page,
}

impl <'a> BaseTemplate<'a> {
    pub fn render(&self, root: &Section) -> Result<String> {
        let mut t_context = TContext::new();
        t_context.insert("table", self.table);
        t_context.insert("page", &self.page.render(root)?);

        base_template()
            .render("base", &t_context)
            .context("failed to render basetemplate")
    }
}
