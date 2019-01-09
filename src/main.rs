use actix;
use actix_web;
use futures;

use actix_web::{
    client, server, App, AsyncResponder, Body, Error, HttpMessage,
    HttpResponse, http, Path, HttpRequest, FutureResponse
};
use clap;
use futures::{Future, Stream};
use md5::{Md5, Digest};

/// streaming client request to a streaming server response
fn streaming((path, req): (Path<String>, HttpRequest<State>)) -> FutureResponse<HttpResponse> {
    let query = req.query_string();
    let password = &req.state().password;

    let fernet = fernet::Fernet::new(&password).unwrap();
    let email = fernet.decrypt(&path.as_ref());
    let email = email.unwrap();

    let hash = Md5::new().chain(email).result();
    // send client request
    client::ClientRequest::get(format!("https://www.gravatar.com/avatar/{:x}?{}", hash, query))
        .finish().unwrap()
        .send()                         // <- connect to host and send request
        .map_err(Error::from)           // <- convert SendRequestError to an Error
        .and_then(|resp| {              // <- we received client response
            let mut new_response = HttpResponse::Ok();
            for (key, value) in resp.headers().into_iter() {
                if key == http::header::CACHE_CONTROL ||
                    key == http::header::DATE ||
                    key == http::header::EXPIRES
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
    password: String,
}

fn main() {
    let sys = actix::System::new("http-proxy");

    let matches =
        clap::App::new("gravatar-proxy")
            .about("Gravatar proxy")
            .version("0.1.0")
            .arg(clap::Arg::with_name("bind")
                .help("Bind to a specific address (ip:port)")
                .long("bind")
                .value_name("ADDR")
                .default_value("localhost:6000"))
            .arg(clap::Arg::with_name("password")
                .help("Password for encrypting the email addresses")
                .long("password")
                .value_name("PASSWORD")
                .default_value("password"))
            .get_matches();

    let bind = matches.value_of("bind").unwrap();
    let password = matches.value_of("password").unwrap().to_string();

    server::new(move || {
        App::with_state(State{password: password.clone()})
            .resource("/avatar/{path}", |r| r.method(http::Method::GET).with(streaming))
    }).workers(1)
        .bind(bind)
        .unwrap()
        .start();

    let _ = sys.run();
}
