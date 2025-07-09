use std::{
    ffi::OsStr,
    fs::read_to_string,
    ops::{Deref, DerefMut},
    path::Path,
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result};
use color_print::ceprintln;
use hyper::body::Bytes;
use tokio::sync::broadcast::Sender;

use crate::{
    config::Config,
    locator::Locator,
    reader::{READS, ThreadNode, markdown_to_html, read},
    serve::send_reload,
};

use notify::{
    Event, EventKind, RecursiveMode,
    event::{CreateKind, ModifyKind, RemoveKind},
};
use notify_debouncer_full::{DebouncedEvent, new_debouncer};

pub fn spawn_watcher_thread(sse: Sender<Bytes>) -> JoinHandle<()> {
    thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut debouncer =
            new_debouncer(Duration::from_millis(20), None, tx).expect("Could not create debouncer");

        debouncer
            .watch(Path::new(&Config::get().content), RecursiveMode::Recursive)
            .expect("Could not watch directory");

        ceprintln!(
            "<blue>Watching {}</blue>",
            Config::get().content.to_string_lossy()
        );

        for result in rx {
            match result {
                Ok(events) => events.iter().for_each(|e| {
                    let now = SystemTime::now();
                    match handle_event(e) {
                        Ok(true) => {
                            if let Err(err) = send_reload(&sse) {
                                ceprintln!("<red>{err}</red>");
                            }
                            ceprintln!(
                                "<green>Elapsed Time: {}ms</green>",
                                now.elapsed().unwrap_or_default().as_micros() as f64 / 1000.0
                            );
                        }
                        Ok(false) => (),
                        Err(err) => ceprintln!("<red>{err}</red>"),
                    }
                }),
                Err(errors) => errors.iter().for_each(|err| ceprintln!("<red>{err}</red>")),
            }
        }
    })
}

fn handle_event(event: &DebouncedEvent) -> Result<bool> {
    let updated = match event {
        DebouncedEvent {
            event:
                Event {
                    kind: EventKind::Modify(ModifyKind::Data(_)),
                    paths,
                    ..
                },
            ..
        } => {
            let reads = READS.lock().unwrap();
            for path in paths {
                let loc = Locator::from_content_path(path)?;
                if let Some(node) = reads.get(&loc) {
                    let text = read_to_string(path)?;
                    match node.lock().unwrap().deref_mut() {
                        ThreadNode::Section(section) => {
                            let page_body = markdown_to_html(text, &section.body.loc)?;
                            section.body.content = page_body;
                        }
                        ThreadNode::Page(page) => {
                            let page_body = markdown_to_html(text, &page.loc)?;
                            page.content = page_body;
                        }
                    }

                    println!("Detected change for url: {}", loc.url());
                }
            }
            true
        }
        DebouncedEvent {
            event:
                Event {
                    kind: EventKind::Create(CreateKind::File),
                    paths,
                    ..
                },
            ..
        } => {
            let mut reads = READS.lock().unwrap();
            for path in paths {
                let path = path.canonicalize()?;

                let parent_locator = Locator::from_content_path(&path)?.parent();

                let mut parent = path.parent().with_context(|| {
                    format!("Could not get parent of created file: {}", path.display())
                })?;

                if path.file_stem() == Some(OsStr::new("index")) {
                    parent = parent.parent().with_context(|| {
                        format!(
                            "Could not get parent of parent of index.md: {}",
                            path.display()
                        )
                    })?;
                }

                let new_node = read(parent, &parent_locator.parent(), &mut reads)?;
                if let Some(parent_node) = reads.get(&parent_locator) {
                    let mut parent_node = parent_node.lock().unwrap();
                    let parent_section = parent_node
                        .get_section_mut()
                        .context("Impossible situation encountered on file create event!")?;
                    *parent_section = new_node;
                    println!("Detected added page");
                }
            }
            true
        }

        DebouncedEvent {
            event:
                Event {
                    kind: EventKind::Remove(RemoveKind::File | RemoveKind::Folder),
                    paths,
                    ..
                },
            ..
        } => {
            let mut reads = READS.lock().unwrap();
            for path in paths {
                let loc = Locator::from_content_path(path)?;

                reads.remove(&loc);

                let parent_locator = loc.parent();
                if let Some(parent_node) = reads.get(&parent_locator) {
                    let mut parent_node = parent_node.lock().unwrap();
                    let parent_section = parent_node
                        .get_section_mut()
                        .context("Impossible situation encountered on file delete event!")?;

                    parent_section
                        .children
                        .retain(|child| match child.lock().unwrap().deref() {
                            ThreadNode::Section(section) => section.body.loc != loc,
                            ThreadNode::Page(page) => page.loc != loc,
                        });
                    println!("Detected file removal");
                };
            }
            true
        }
        _ => false,
    };

    Ok(updated)
}
