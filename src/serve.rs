use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;

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

use crate::Config;
use crate::Error;
use crate::render::RENDERS;

pub async fn serve() -> Result<(), Error> {
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
) -> Result<Response<BoxBody<Bytes, std::io::Error>>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, path) => page_send(path).await,
        _ => not_found(),
    }
}

fn not_found() -> Result<Response<BoxBody<Bytes, std::io::Error>>, Error> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(
            Full::new("NOT FOUND".into())
                .map_err(|e| match e {})
                .boxed(),
        )
        .map_err(Error::from)
}

async fn page_send(url: &str) -> Result<Response<BoxBody<Bytes, std::io::Error>>, Error> {
    let renders = RENDERS.lock().unwrap();

    match renders.get(url) {
        Some(page) => Response::builder()
            .status(StatusCode::OK)
            .body(
                Full::new(page.clone().into())
                    .map_err(|e| match e {})
                    .boxed(),
            )
            .map_err(Error::from),
        None => not_found(),
    }
}
