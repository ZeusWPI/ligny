mod errors;
mod render;
mod templates;

enum Node {
    Section(Section),
    Page(Page),
}

struct FrontMatter {
    title: String,
}

struct Section {
    children: Vec<Node>,
    body: Page,
}

struct Page {
    frontmatter: FrontMatter,
    content: String,
    path: String,
}

fn main() {}

// markdown files -> parsing markdown to html -> converting to Nodes -> rendering page contents
// using templates
