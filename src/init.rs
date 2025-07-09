use std::{
    fs::{File, create_dir, exists},
    io::Write,
    path::PathBuf,
};

use anyhow::{Result, bail};
use include_dir::{Dir, include_dir};

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static/");
static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/");

pub fn init_files() -> Result<()> {
    // if either exists, or an error occured, exit
    if exists("static/").unwrap_or(true) || exists("templates/").unwrap_or(true) {
        bail!("'static/' or 'templates/' already exists, not creating either")
    }

    init_default_static()?;
    init_default_templates()?;

    Ok(())
}

pub fn init_default_static() -> Result<()> {
    create_dir("static/").or(anyhow::Ok(()))?;
    for file in STATIC_DIR.files() {
        let path = PathBuf::from("static/").join(file.path());
        File::create(path)?.write_all(file.contents())?;
    }

    Ok(())
}

pub fn init_default_templates() -> Result<()> {
    create_dir("templates/").or(anyhow::Ok(()))?;
    for file in TEMPLATE_DIR.files() {
        let path = PathBuf::from("templates/").join(file.path());
        File::create(path)?.write_all(file.contents())?;
    }

    Ok(())
}
