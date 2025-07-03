use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;

use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Method;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::build;
use crate::Config;
use crate::Error;

pub async fn serve() -> Result<(), Error> {
    build()?;
    println!("Build complete");

    let addr: SocketAddr = SocketAddr::from((Config::get().address, Config::get().port));

    println!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        if let Err(err) = http1::Builder::new()
            .serve_connection(io, service_fn(reponse))
            .await
        {
            print!("Failed to serve connection: {:?}", err);
        }
    }
}

async fn reponse(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, path) => file_send(path).await,
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

async fn file_send(url: &str) -> Result<Response<BoxBody<Bytes, std::io::Error>>, Error> {
    let path = Path::new(&Config::get().public)
        .join(url.strip_prefix("/").unwrap_or(url))
        .join("index.html");

    let file = File::open(path);

    match file {
        Ok(mut f) => {
            let mut body: String = String::new();
            f.read_to_string(&mut body)?;

            Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(body.into()).map_err(|e| match e {}).boxed())
                .map_err(Error::from)
        }
        Err(_) => not_found(),
    }
}
