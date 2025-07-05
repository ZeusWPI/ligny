use std::net::SocketAddr;
use std::ops::Deref;

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

use anyhow::Result;

use crate::Config;
use crate::locator::Locator;
use crate::reader::READS;
use crate::reader::ThreadNode;
use crate::render::get_root;

pub async fn serve() -> Result<()> {
    let addr: SocketAddr = SocketAddr::from((Config::get().address, Config::get().port));

    println!("Listening on {addr}");

    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        if let Err(err) = http1::Builder::new()
            .serve_connection(io, service_fn(reponse))
            .await
        {
            print!("Failed to serve connection: {err:?}");
        }
    }
}

async fn reponse(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, path) => page_send(path).await,
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

async fn page_send(url: &str) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    let reads = READS.lock().unwrap();

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
        None => not_found(),
    }
}
