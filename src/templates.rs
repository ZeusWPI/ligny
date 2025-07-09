use askama::Template;

#[derive(Template)]
#[template(path = "content_table.html")]
pub struct ContentTableTemplate<'a> {
    pub pages: Vec<&'a crate::reader::Page>,
    pub curr_page: &'a crate::reader::Page,
}

#[derive(Template)]
#[template(path = "base.html", escape = "none")]
pub struct BaseTemplate<'a> {
    pub table: &'a ContentTableTemplate<'a>,
    pub page: &'a crate::reader::Page,
}
