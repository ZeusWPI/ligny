use std::{
    collections::HashMap,
    fs::{DirEntry, read_dir, read_to_string},
    ops::Deref,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use anyhow::{Context, Result, anyhow, bail};
use color_print::ceprintln;
use markdown_ppp::{
    self,
    ast::{Block, Inline},
    html_printer::{config::Config, render_html},
    parser::parse_markdown,
};

use std::sync::Arc;

use crate::locator::Locator;

pub type ThreadNodeType = Arc<Mutex<ThreadNode>>;

#[derive(Debug, Clone)]
pub enum ThreadNode {
    Section(ThreadSection),
    Page(Page),
}

#[derive(Debug)]
pub enum Node {
    Section(Section),
    Page(Page),
}

#[derive(Debug, Clone)]
pub struct ThreadSection {
    pub children: Vec<ThreadNodeType>,
    pub body: Page,
}

#[derive(Debug)]
pub struct Section {
    pub children: Vec<Node>,
    pub body: Page,
}

impl ThreadSection {
    pub fn new(body: Page) -> ThreadSection {
        ThreadSection {
            children: Vec::new(),
            body,
        }
    }
}

// Converting from ThreadSection -> Section should be avoided as it has to clone every Page.
impl From<&ThreadSection> for Section {
    fn from(thread_section: &ThreadSection) -> Self {
        let extracted_children: Vec<Node> = thread_section
            .children
            .iter()
            .map(|n| n.lock().unwrap().deref().into())
            .collect();
        Section {
            children: extracted_children,
            body: thread_section.body.clone(),
        }
    }
}

impl From<&ThreadNode> for Node {
    fn from(thread_node: &ThreadNode) -> Self {
        match thread_node {
            ThreadNode::Section(section) => Node::Section(section.into()),
            ThreadNode::Page(page) => Node::Page(page.clone()),
        }
    }
}

impl ThreadNode {
    pub fn get_section_mut(&mut self) -> Result<&mut ThreadSection> {
        match self {
            ThreadNode::Section(thread_section) => Ok(thread_section),
            ThreadNode::Page(_) => Err(anyhow!("Expected Node to be a section")),
        }
    }

    pub fn get_section(&self) -> Result<&ThreadSection> {
        match self {
            ThreadNode::Section(thread_section) => Ok(thread_section),
            ThreadNode::Page(_) => Err(anyhow!("Expected Node to be a section")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub title: String,
    pub loc: Locator,
    pub content: String,
}

pub static READS: LazyLock<Mutex<HashMap<Locator, ThreadNodeType>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn read(
    path: &Path,
    loc: &Locator,
    reads: &mut HashMap<Locator, ThreadNodeType>,
) -> Result<ThreadSection> {
    let index_path = path.join("index.md");
    let section_name = file_title(path)?;

    let loc = if path
        .canonicalize()?
        .eq(&crate::config::Config::get().content.canonicalize()?)
    {
        loc.clone()
    } else {
        loc.join(&Locator::new(&section_name))
    };

    let file_content = read_to_string(&index_path)
        .with_context(|| format!("Failed reading index file {index_path:?}"))?;
    let content = markdown_to_html(file_content, &loc)
        .with_context(|| format!("Failed converting markdown to HTML in file {index_path:?}"))?;

    // make section with index page
    let mut section = ThreadSection::new(Page {
        title: section_name.clone(),
        loc: loc.clone(),
        content,
    });

    // read files, filter index and sort by number
    let files = read_dir(path)
        .with_context(|| format!("Failed to read dir with path {}", path.display()))?;
    let mut files = files
        .filter_map(|x| {
            let entry = match x {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Skipping entry due to I/O error: {e}");
                    return None;
                }
            };

            if entry.path().ends_with("index.md") {
                return None;
            }

            match file_order_index(&entry.path()) {
                Ok(index) => Some((index, entry)),
                Err(err) => {
                    ceprintln!("<yellow>Skipping file with reason:\n{err:?}</yellow>\n");
                    None
                }
            }
        })
        .collect::<Vec<(u32, DirEntry)>>();

    files.sort_by_key(|(index, _)| *index);

    // loop over nodes and add them to the section
    for (_, item) in files {
        let file_type = item.file_type()?;
        if file_type.is_dir() {
            let child_node = read(&item.path(), &loc, reads)?;
            let loc = child_node.body.loc.clone();
            let thread_section = Arc::new(Mutex::new(ThreadNode::Section(child_node)));
            reads.insert(loc, thread_section.clone());
            section.children.push(thread_section);
        } else if file_type.is_file() {
            let page = read_page(&item.path(), &loc)?;
            let loc = page.loc.clone();
            let thread_node = Arc::new(Mutex::new(ThreadNode::Page(page)));
            section.children.push(Arc::clone(&thread_node));
            reads.insert(loc, thread_node);
        } else {
            continue;
        };
    }

    Ok(section)
}

/// given a markdown file path, reads the contents and converts it to HTML
pub fn read_page(file_path: &PathBuf, loc: &Locator) -> Result<Page> {
    let file_content = read_to_string(file_path)
        .with_context(|| format!("Can't read file: '{}'", file_path.display()))?;
    let page_content = markdown_to_html(file_content, loc)
        .with_context(|| format!("Can't convert markdown to html: '{}'", file_path.display()))?;

    let file_name = file_title(file_path)?;
    Ok(Page {
        title: file_name.clone(),
        loc: loc.join(&Locator::new(&file_name)),
        content: page_content,
    })
}

/// returns the index at the start of the file name
pub fn file_order_index(path: &Path) -> Result<u32> {
    let stem = get_stem(path)?;
    stem.split('_')
        .next()
        .ok_or_else(|| anyhow!("Filename does not contain an '_' separator: '{}'", stem))?
        .parse()
        .with_context(|| {
            format!(
                "Could not parse order index in '{}' from {}",
                stem,
                path.display()
            )
        })
}

/// gets the title from a filename
///
/// strips leading order index and extension
fn file_title(path: &Path) -> Result<String> {
    let stem = get_stem(path)?;
    let filename_parts = stem.split("_").skip(1);

    let file_title = filename_parts.collect::<Vec<&str>>().join(" ");
    if file_title.is_empty() {
        bail!("Filename does not have a title: '{}'", path.display());
    }

    Ok(file_title)
}

fn get_stem(path: &Path) -> Result<&str> {
    path.file_stem()
        .ok_or_else(|| anyhow!("File has no stem: '{}'", path.display()))?
        .to_str()
        .ok_or_else(|| anyhow!("Filename is not valid UTF-8: '{}'", path.display()))
}

/// convert markdown into HTML
pub fn markdown_to_html(content: String, loc: &Locator) -> Result<String> {
    let state = markdown_ppp::parser::MarkdownParserState::default();
    let mut doc = parse_markdown(state, &content)
        .map_err(|e| anyhow!("Failed to parse markdown with nom error: {e}"))?;

    rewrite_links(&mut doc.blocks, loc)?;

    Ok(render_html(&doc, Config::default()))
}

/// rewrite relative links inside the markdown to valid relative urls
fn rewrite_links(blocks: &mut Vec<Block>, loc: &Locator) -> Result<()> {
    for item in blocks {
        if let Block::Paragraph(p_items) = item {
            for p_item in p_items {
                if let Inline::Link(link) = p_item {
                    if link.destination.starts_with("http") {
                        continue;
                    } // TODO make better
                    link.destination = rewrite_internal_link(link.destination.clone(), loc)
                        .with_context(|| {
                            format!("Can't rewrite link with destination {}", link.destination)
                        })?;
                }
            }
        }
    }

    Ok(())
}

fn rewrite_internal_link(link: String, loc: &Locator) -> Result<String> {
    let ok = loc.join(&Locator::new(&link)).url();
    Ok(ok)
}
