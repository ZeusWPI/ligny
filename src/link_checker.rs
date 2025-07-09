use anyhow::Result;
use std::collections::HashMap;

use crate::{
    CONTEXT,
    locator::Locator,
    reader::{Page, Section, ThreadNodeType},
    render::get_root,
};
use color_print::ceprintln;

pub fn check_links(page: &Page, reads: &HashMap<Locator, ThreadNodeType>) {
    for link in &page.links {
        if !reads.contains_key(link) {
            ceprintln!(
                "<yellow>Dead link in page {}, pointing to non-existing {link}</yellow>",
                page.title
            );
        }
    }
}

pub fn check_links_root() -> Result<()> {
    let context = CONTEXT.lock().unwrap();
    let root = get_root(&context.reads)?;
    check_links_section(&root, &context.reads);

    Ok(())
}

pub fn check_links_section(section: &Section, reads: &HashMap<Locator, ThreadNodeType>) {
    check_links(&section.body, reads);
    for child in &section.children {
        match child {
            crate::reader::Node::Section(section) => check_links_section(section, reads),
            crate::reader::Node::Page(page) => check_links(page, reads),
        }
    }
}
