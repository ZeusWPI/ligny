use askama::Template;

use crate::{
    errors::Error,
    reader::{Page, Section},
    templates::{BaseTemplate, ContentTableTemplate},
};

impl Page {
    pub fn render(&self, root: &Section) -> Result<String, Error> {
        let content_table = ContentTableTemplate { root, page: self };

        BaseTemplate {
            table: &content_table,
            page: self,
        }
        .render()
        .map_err(Error::from)
    }
}
