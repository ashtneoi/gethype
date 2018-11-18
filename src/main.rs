extern crate chrono;
extern crate futures;
extern crate hyper;

use chrono::Local;
use futures::Future;
use hyper::{Body, Method, Request, Response, Server};
use hyper::service::service_fn_ok;

fn date(_req: &Request<Body>) -> Response<Body> {
    let today = Local::today()
        .naive_local()
        .format("%Y-%m-%d"); // FIXME
    let today_fmt = format!("{}", today);
    Response::new(Body::from(today_fmt))
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
