use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use tracing::{span, warn, Instrument as _, Level, Span};
use mime_guess;

const DOC_ROOT : &str = "/data/www";

async fn file_mod(r: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    response.headers_mut().insert(hyper::header::SERVER, HeaderValue::from_static("rws"));
    for c in r.uri().path().chars() {
        if c.is_ascii_control() {
            *response.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(response);
        }
    }
    let mut file_path = PathBuf::new();
    file_path.push(format!("{}{}", DOC_ROOT, r.uri().path()));
    let mut file = match File::open(&file_path) {
        Err(_e) => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            return Ok(response)
        },
        Ok(file) => file,
    };

    let mut content = Vec::new();
    match file.read_to_end(&mut content) {
        Err(e) => {
            println!("{}", e.to_string());
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        },
        Ok(_) => {
            *response.body_mut() = Body::from(content);
            let mime = mime_guess::from_path(file_path.as_os_str()).first().unwrap();
            let mime_str = mime.to_string();
            let content_type = HeaderValue::from_str(&mime_str).unwrap();
            response.headers_mut().insert(hyper::header::CONTENT_TYPE, content_type);
        },
    };

    Ok(response)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(file_mod))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
