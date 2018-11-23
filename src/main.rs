extern crate chrono;
extern crate curly;
extern crate futures;
extern crate hyper;

use chrono::Local;
use curly::render_file_to_string;
use futures::Future;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn_ok;
use std::collections::HashMap;
use std::path::Path;

fn date(_req: &Request<Body>) -> Response<Body> {
    let today = Local::today()
        .naive_local()
        .format("%Y-%m-%d"); // FIXME
    let today_fmt = format!("{}", today);
    let mut ctx = HashMap::new();
    ctx.insert("date".to_string(), today_fmt);
    match render_file_to_string(
        Path::new("tmpl/date.html"),
        &ctx,
    ) {
        Ok(b) => {
            return Response::new(Body::from(b));
        },
        Err(e) => {
            eprintln!("error: {}", e);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty()).unwrap();
        },
    };
}

fn route(req: Request<Body>) -> Response<Body> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/date") => date(&req),
        _ => Response::new(Body::from("what?")),
    }
}

fn main() {
    let bind_addr = ([127, 0, 0, 1], 8000).into();
    let svc = || { service_fn_ok(route) };
    let server = Server::bind(&bind_addr)
        .serve(svc)
        .map_err(|e| { eprintln!("error: {}", e) });
    hyper::rt::run(server);
}
