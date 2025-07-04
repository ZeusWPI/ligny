use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{DirEntry, read_dir, read_to_string},
    io::Result,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use markdown_ppp::{
    self,
    ast::{Block, Inline, Link},
    html_printer::{config::Config, render_html},
    parser::parse_markdown,
};

use std::sync::Arc;

use crate::locator::Locator;

type ThreadNodeType = Arc<Mutex<ThreadNode>>;

#[derive(Debug, Clone)]
pub enum ThreadNode {
    Section(ThreadSection),
    Page(Page),
}

#[derive(Debug, Clone)]
pub enum Node {
    Section(Section),
    Page(Page),
}

#[derive(Debug, Clone)]
pub struct ThreadSection {
    pub children: Vec<ThreadNodeType>,
    pub body: Page,
}

#[derive(Debug, Clone)]
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

impl From<ThreadSection> for Section {
    fn from(thread_section: ThreadSection) -> Self {
        let extracted_children: Vec<Node> = thread_section
            .children
            .iter()
            .map(|n| n.lock().unwrap().to_owned().into())
            .collect();
        Section {
            children: extracted_children,
            body: thread_section.body,
        }
    }
}

impl From<ThreadNode> for Node {
    fn from(thread_node: ThreadNode) -> Self {
        match thread_node {
            ThreadNode::Section(section) => Node::Section(section.into()),
            ThreadNode::Page(page) => Node::Page(page),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub title: String,
    pub loc: Locator,
    pub content: String,
}

pub static READS: LazyLock<Mutex<HashMap<PathBuf, ThreadNodeType>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn read(
    path: &Path,
    loc: &Locator,
    reads: &mut HashMap<PathBuf, ThreadNodeType>,
) -> ThreadSection {
    let index_path = path.join("index.md");
    let (_, section_name) = filename_info(path.file_stem().unwrap());

    let loc = if path.eq(Path::new(&crate::config::Config::get().content)) {
        loc.clone()
    } else {
        loc.join(&Locator::new(&section_name))
    };

    let index = read_markdown(read_to_string(&index_path).unwrap(), &loc);

    // make section with index page
    let mut section = ThreadSection::new(Page {
        title: section_name.clone(),
        loc: loc.clone(),
        content: index,
    });

    // read files, filter index and sort by number
    let files = read_dir(path).unwrap();
    let mut files = files
        .filter(|x| {
            if let Ok(item) = x {
                item.file_name() != "index.md"
            } else {
                false
            }
        })
        .map(|x| x.map(|x| (filename_info(x.path().file_stem().unwrap()), x)))
        .collect::<Result<Vec<((u32, String), DirEntry)>>>()
        .unwrap();

    files.sort_by_key(|x| x.0.0);

    // loop over nodes and add them to the section
    for ((_, file_name), item) in files {
        let file_type = item.file_type().unwrap();
        if file_type.is_dir() {
            let child_section = read(&item.path(), &loc, reads);
            let thread_node = Arc::new(Mutex::new(ThreadNode::Section(child_section)));
            section.children.push(thread_node.clone());
            reads.insert(item.path(), thread_node);
        } else if file_type.is_file() {
            let text = read_to_string(item.path()).unwrap();
            let page_body = read_markdown(text, &loc);

            let thread_node = Arc::new(Mutex::new(ThreadNode::Page(Page {
                title: file_name.clone(),
                loc: loc.join(&Locator::new(&file_name)),
                content: page_body,
            })));

            section.children.push(thread_node.clone());
            reads.insert(item.path(), thread_node);
        }
    }

    section
}

fn filename_info(filename: &OsStr) -> (u32, String) {
    let filename = filename.to_str().unwrap();
    let mut filename_parts = filename.split("_");

    let number = filename_parts.next().unwrap().parse().unwrap();
    let name = filename_parts.collect::<Vec<&str>>().join(" ");

    (number, name)
}

fn read_markdown(content: String, loc: &Locator) -> String {
    let state = markdown_ppp::parser::MarkdownParserState::default();
    let mut doc = parse_markdown(state, &content).expect("failed to parse markdown");

    rewrite_links(&mut doc.blocks, loc);

    render_html(&doc, Config::default())
}

fn rewrite_links(blocks: &mut Vec<Block>, loc: &Locator) {
    for item in blocks {
        if let Block::Paragraph(p_items) = item {
            for p_item in p_items {
                if let Inline::Link(link) = p_item {
                    // TODO determine if internal link
                    link.destination = rewrite_internal_link(link.destination.clone(), loc);
                }
            }
        }
    }
}

fn rewrite_internal_link(link: String, loc: &Locator) -> String {
    let path = PathBuf::from(&link);
    let (_, filename) = filename_info(path.file_stem().unwrap());
    loc.join(&Locator::new(&filename)).url()
}
