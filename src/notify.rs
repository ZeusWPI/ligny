use std::{
    fs::read_to_string,
    ops::DerefMut,
    path::Path,
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Result;

use crate::{
    config::Config,
    reader::{Node, READS, Section, ThreadNode, markdown_to_html},
};

use notify::{Event, EventKind, RecursiveMode, event::ModifyKind};
use notify_debouncer_full::{DebouncedEvent, new_debouncer};

pub fn spawn_watcher_thread() -> JoinHandle<()> {
    thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut debouncer =
            new_debouncer(Duration::from_millis(20), None, tx).expect("Could not create debouncer");

        debouncer
            .watch(Path::new(&Config::get().content), RecursiveMode::Recursive)
            .expect("Could not watch directory");

        println!("Watching {}", Config::get().content);

        for result in rx {
            match result {
                Ok(events) => events.iter().for_each(|e| handle_event(e).unwrap()),
                Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
            }
        }
    })
}

fn handle_event(event: &DebouncedEvent) -> Result<()> {
    if let DebouncedEvent {
        event:
            Event {
                kind: EventKind::Modify(ModifyKind::Data(_)),
                paths,
                ..
            },
        ..
    } = event
    {
        let reads = READS.lock().unwrap();
        for path in paths {
            if let Some(node) = reads.get(&path.canonicalize().unwrap()) {
                let root_path = Path::new(&Config::get().content).join("index.md");
                let root: Node = reads
                    .get(&root_path.canonicalize().unwrap())
                    .unwrap()
                    .into();

                let root: Section = match root {
                    Node::Section(section) => section,
                    Node::Page(_) => todo!(),
                };

                let text = read_to_string(path)?;
                match node.lock().unwrap().deref_mut() {
                    ThreadNode::Section(section) => {
                        let page_body = markdown_to_html(text, &section.body.loc)?;
                        section.body.content = page_body;
                        section.body.render(&root)?;
                    }
                    ThreadNode::Page(page) => {
                        let page_body = markdown_to_html(text, &page.loc)?;
                        page.content = page_body;
                        page.render(&root)?;
                    }
                }
            }
        }
    };

    Ok(())
}
