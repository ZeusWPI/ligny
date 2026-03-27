use askama::Template;

use crate::PRIVATE_SECTION;

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

impl<'a> ContentTableTemplate<'a> {
    pub fn filter_private_sections(
        mut self,
    ) -> Self {
        if self.curr_page.loc.components[0] != PRIVATE_SECTION {
            self.pages = self
                .pages
                .iter()
                .filter(|p| {
                    let components = &p.loc.components;
                    !(components[0] == PRIVATE_SECTION && components[1] != "index")
                })
                .map(|e| *e)
                .collect();
        }
        self
    }
}
