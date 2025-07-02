use askama::Template;

use crate::Node::{Page, Section};

#[derive(Template)]
#[template(path = "content_table.html")]
pub struct ContentTableTemplate<'a> {
    pub root: &'a crate::Section,
    pub page: &'a crate::Page,
}

#[derive(Template)]
#[template(path = "base.html", escape = "none")]
pub struct BaseTemplate<'a> {
    pub table: &'a ContentTableTemplate<'a>,
    pub page: &'a crate::Page,
}
