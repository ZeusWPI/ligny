use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::ops::Deref;

use futures_util::TryStreamExt;
use http_body_util::StreamBody;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::Method;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use anyhow::{Context, Result};
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::wrappers::BroadcastStream;

use crate::Config;
use crate::locator::Locator;
use crate::notify::spawn_watcher_thread;
use crate::reader::ThreadNode;
use crate::reader::{READS, ThreadNodeType};
use crate::render::get_root;
use crate::search::render_index;

pub async fn serve() -> Result<()> {
    let addr: SocketAddr = SocketAddr::from((Config::get().address, Config::get().port));

    println!("Listening on {addr}");

    let (tx, _rx) = broadcast::channel(32);

    let _ = spawn_watcher_thread(tx.clone());

    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let new_tx = tx.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|req| reponse(req, new_tx.clone())))
                .await
                && !err.is_incomplete_message()
            {
                print!("Failed to serve connection: {err:?}");
            }
        });
    }
}

pub fn send_reload(tx: &Sender<Bytes>) -> Result<()> {
    tx.send("event: reload\ndata: \n\n".into())
        .with_context(|| "Could not send reload event")?;
    Ok(())
}

async fn reponse(
    req: Request<hyper::body::Incoming>,
    tx: Sender<Bytes>,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/sse") => event_stream(tx).await,
        (&Method::GET, path) => page_send(path),
        _ => not_found(),
    }
}

fn not_found() -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    Ok(Response::builder().status(StatusCode::NOT_FOUND).body(
        Full::new("NOT FOUND".into())
            .map_err(|e| match e {})
            .boxed(),
    )?)
}

fn page_send(url: &str) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    let reads = READS.lock().unwrap();
    let index_url = format!("/{}", Config::get().index_name);
    if url == index_url {
        index_send(&reads)
    } else {
        match reads.get(&Locator::from_url(url)) {
            Some(node) => {
                let root = get_root(&reads)?;
                let node = node.lock().unwrap();

                let page = match node.deref() {
                    ThreadNode::Section(section) => &section.body,
                    ThreadNode::Page(page) => page,
                };

                let html = page.render(&root)?;

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Full::new(html.into()).map_err(|e| match e {}).boxed())?)
            }
            None => static_file_serve(url),
        }
    }
}

fn index_send(
    reads: &HashMap<Locator, ThreadNodeType>,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    let index = render_index(reads)?;
    let json = serde_json::to_string(&index)?;
    let body = Full::new(json.into()).map_err(|e| match e {}).boxed();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body)?;
    Ok(response)
}

async fn event_stream(tx: Sender<Bytes>) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    let rx2 = tx.subscribe();
    let stream = BroadcastStream::from(rx2);

    let reader_stream = stream
        .map_ok(hyper::body::Frame::data)
        .map_err(|_item| panic!());

    let stream = StreamBody::new(reader_stream);
    let boxed_body = stream.boxed();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .body(boxed_body)?;

    Ok(response)
}

fn static_file_serve(url: &str) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    let loc = Locator::from_url(url);
    let file = File::open(loc.static_path());
    if let Ok(mut file) = file {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(content.into()).map_err(|e| match e {}).boxed())?)
    } else {
        not_found()
    }
}
