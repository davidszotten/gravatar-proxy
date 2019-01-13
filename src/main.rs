use actix;
use actix_web;
use futures;

use actix_web::{
    client, http, server, App, AsyncResponder, Body, Error, FutureResponse, HttpMessage,
    HttpRequest, HttpResponse, Path,
};
use clap;
use futures::{Future, Stream};
use md5::{Digest, Md5};

fn streaming((path, req): (Path<String>, HttpRequest<State>)) -> FutureResponse<HttpResponse> {
    // fetch query params to pass on to gravatar
    let query = req.query_string();

    let fernet = &req.state().fernet;

    let email = match fernet.decrypt(&path.as_ref()) {
        Ok(value) => value,
        Err(_) => {
            return Box::new(futures::future::result::<HttpResponse, Error>(Ok(
                HttpResponse::build(http::StatusCode::BAD_REQUEST).finish(),
            )));
        }
    };

    // gravatar hash of email address
    let hash = Md5::new().chain(email).result();

    client::ClientRequest::get(format!(
        "https://www.gravatar.com/avatar/{:x}?{}",
        hash, query
    ))
    .finish()
    .unwrap()
    .send() // <- connect to host and send request
    .map_err(Error::from) // <- convert SendRequestError to an Error
    .and_then(|resp| {
        // <- we received client response
        let mut new_response = HttpResponse::Ok();

        // copy over cache headers
        // (some of the other headers include e.g. the email hash so we want
        // to be selctive)
        for (key, value) in resp.headers().into_iter() {
            if key == http::header::CACHE_CONTROL
                || key == http::header::DATE
                || key == http::header::EXPIRES
            {
                new_response.header(key.clone(), value.clone());
            }
        }
        Ok(new_response
            // read one chunk from client response and send this chunk to a server response
            // .from_err() converts PayloadError to an Error
            .body(Body::Streaming(Box::new(resp.payload().from_err()))))
    })
    .responder()
}

struct State {
    fernet: fernet::Fernet,
}

fn main() {
    let sys = actix::System::new("http-proxy");

    let matches = clap::App::new("gravatar-proxy")
        .about("Gravatar proxy")
        .version("0.1.0")
        .arg(
            clap::Arg::with_name("key")
                .help("Key for encrypting the email addresses")
                .long("key")
                .value_name("KEY")
                .required(true)
                .index(1)
                .validator(|v| match fernet::Fernet::new(&v) {
                    Some(_) => Ok(()),
                    None => Err(String::from("Invalid Fernet key")),
                }),
        )
        .arg(
            clap::Arg::with_name("bind")
                .help("Bind to a specific address (ip:port)")
                .long("bind")
                .value_name("ADDR")
                .default_value("localhost:6000"),
        )
        .get_matches();

    // clap makes sure these are set. ok to unwrap
    let bind = matches.value_of("bind").unwrap();
    let key = matches.value_of("key").unwrap().to_string();

    server::new(move || {
        App::with_state(State {
            // clap validator checks this is ok. fine to unwrap
            fernet: fernet::Fernet::new(&key).unwrap(),
        })
        .resource("/avatar/{path}", |r| {
            r.method(http::Method::GET).with(streaming)
        })
    })
    .workers(1)
    .bind(bind)
    .unwrap()
    .start();

    let _ = sys.run();
}
