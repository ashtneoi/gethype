extern crate chrono;
extern crate curly;
extern crate futures;
extern crate hyper;
extern crate regex;

use chrono::{Local, NaiveDate};
use curly::render_file_to_string;
use futures::Future;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn_ok;
use regex::{Regex, Captures};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;

fn build_simple_error(status_code: u16) -> Response<Body> {
    let mut ctx = HashMap::new();
    ctx.insert("title".to_string(), format!("Error ({})", status_code));
    ctx.insert("style".to_string(), "".to_string());
    ctx.insert(
        "body".to_string(),
        format!("<h1>Error: HTTP {}</h1>", status_code),
    );
    match render_file_to_string(
        Path::new("tmpl/top.html"),
        &ctx,
    ) {
        Ok(b) => {
            Response::builder()
                .status(StatusCode::from_u16(status_code).unwrap())
                .body(Body::from(b)).unwrap()
        },
        Err(e) => {
            eprintln!("server error: {}", e);
            let fallback_body = "\
                <!DOCTYPE html><html lang=en>\
                <head>\
                <meta charset=utf-8>\
                <title>Server error while generating error page</title>\
                </head><body>\
                <h1>Server error while generating error page</h1>\
                </body></html>".to_string();
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(fallback_body)).unwrap()
        },
    }
}

fn today() -> String {
    let today = Local::today()
        .naive_local()
        .format("%Y-%m-%d"); // FIXME
    format!("{}", today)
}

static YMD_FORMAT: &str = "%Y-%m-%d";

fn note(req: &Request<Body>, cap: &Captures) -> Response<Body> {
    match req.method() {
        &Method::GET => {
            let mut ctx = HashMap::new();
            let date = match NaiveDate::parse_from_str(
                cap.get(1).unwrap().as_str(),
                "%Y-%m-%d",
            ) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("client error: {}", e);
                    return build_simple_error(400)
                },
            };
            ctx.insert("today".to_string(), today());
            ctx.insert(
                "date".to_string(),
                format!("{}", date.format(YMD_FORMAT)),
            );
            ctx.insert(
                "prev".to_string(),
                format!("{}", date.pred().format(YMD_FORMAT)),
            );
            ctx.insert(
                "next".to_string(),
                format!("{}", date.succ().format(YMD_FORMAT)),
            );
            let note_name = format!("data/notes/{}", date.format(YMD_FORMAT));
            let text = match File::open(&note_name) {
                Ok(mut f) => {
                    let mut s = String::new();
                    if let Err(e) = f.read_to_string(&mut s) {
                        eprintln!(
                            "server error: can't read note file '{}' ({})",
                            note_name,
                            e,
                        );
                        return build_simple_error(500);
                    }
                    assert!(s.ends_with('\n'));
                    s.pop();
                    s
                },
                Err(e) => {
                    if e.kind() == io::ErrorKind::NotFound {
                        "".to_string()
                    } else {
                        eprintln!(
                            "server error: can't read note file '{}' ({})",
                            note_name,
                            e,
                        );
                        return build_simple_error(500);
                    }
                },
            };
            ctx.insert("text".to_string(), text);
            match render_file_to_string(
                Path::new("tmpl/note.html"),
                &ctx,
            ) {
                Ok(b) => {
                    Response::new(Body::from(b))
                },
                Err(e) => {
                    eprintln!("server error: {}", e);
                    build_simple_error(500)
                },
            }
        },
        &Method::POST => {
            build_simple_error(500)
        },
        _ => build_simple_error(500),
    }
}

struct UrlRouter(Vec<(Regex, fn(&Request<Body>, &Captures) -> Response<Body>)>);

impl UrlRouter {
    fn new(
        mut routes: Vec<(
            &str,
            fn(&Request<Body>, &Captures) -> Response<Body>
        )>,
    ) -> Self {
        UrlRouter(routes.drain(..).map(
            |(s, f)| (Regex::new(s).unwrap(), f)
        ).collect())
    }

    fn route(&self, req: Request<Body>) -> Response<Body> {
        for (pat, f) in &self.0 {
            match pat.captures(req.uri().path()) {
                Some(c) => return f(&req, &c),
                None => (),
            }
        }
        build_simple_error(404)
    }
}


fn main() {
    let bind_addr = ([127, 0, 0, 1], 8000).into();
    let svc = || {
        let router = UrlRouter::new(vec![
            ("/note/([0-9]{4}-[0-9]{2}-[0-9]{2})", note),
        ]);
        service_fn_ok(move |req| router.route(req))
    };
    let server = Server::bind(&bind_addr)
        .serve(svc)
        .map_err(|e| { eprintln!("server error: {}", e) });
    hyper::rt::run(server);
}
