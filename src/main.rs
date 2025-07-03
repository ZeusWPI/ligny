mod errors;
mod render;
mod templates;

use std::path::Path;

use reader::read;

mod reader;

fn main() {
    let section = read(Path::new("0_content/"));
    let res = section.body.render(&section);
    dbg!(res);
}

// markdown files -> parsing markdown to html -> converting to Nodes -> rendering page contents
// using templates
