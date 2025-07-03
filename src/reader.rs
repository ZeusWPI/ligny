use std::{
    ffi::{OsStr, OsString},
    fs::{read_dir, read_to_string, DirEntry},
    io::Result,
    path::Path,
};

use markdown::{self, ParseOptions};

#[derive(Debug)]
pub enum Node {
    Section(Section),
    Page(Page),
}

#[derive(Debug)]
pub struct Section {
    pub children: Vec<Node>,
    pub body: Page,
}

impl Section {
    pub fn new(body: Page) -> Section {
        Section {
            children: Vec::new(),
            body,
        }
    }
}

#[derive(Debug)]
pub struct Page {
    pub title: String,
    pub url: String,
    pub content: String,
}

pub fn read(path: &Path, url: &str) -> Section {
    dbg!(path);
    dbg!(&url);
    let index_path = path.join("index.md");
    let index = read_markdown(read_to_string(&index_path).unwrap());
    let (_, section_name) = filename_info(path.file_stem().unwrap());

    let next_url = if url.is_empty() {
        "".into()
    } else {
        format!("{}/{}", url, &section_name)
    };

    // make section with index page
    let mut section = Section::new(Page {
        title: section_name.clone(),
        url: next_url.to_string(),
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

    files.sort_by_key(|x| x.0 .0);

    // loop over nodes and add them to the section
    for ((_, file_name), item) in files {
        let file_type = item.file_type().unwrap();
        if file_type.is_dir() {
            let child_section = read(&item.path(), &next_url);
            section.children.push(Node::Section(child_section));
        } else if file_type.is_file() {
            let text = read_to_string(item.path()).unwrap();
            let page_body = read_markdown(text);

            section.children.push(Node::Page(Page {
                title: file_name.clone(),
                url: format!("{}/{}", next_url, &file_name),
                content: page_body,
            }));
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

fn read_markdown(content: String) -> String {
    let mut parse_options = markdown::ParseOptions::default();
    parse_options.constructs.frontmatter = true;

    markdown::to_html_with_options(
        &content,
        &markdown::Options {
            parse: parse_options,
            ..Default::default()
        },
    )
    .expect("cant read index")
}
