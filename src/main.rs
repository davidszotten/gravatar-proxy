use actix;
use actix_web;
// use env_logger;
use futures;

use actix_web::{
    client, server, App, AsyncResponder, Body, Error, HttpMessage,
    HttpResponse, http, Path, HttpRequest, FutureResponse
};
use futures::{Future, Stream};
use md5::{Md5, Digest};

/// streaming client request to a streaming server response
fn streaming((path, req): (Path<String>, HttpRequest)) -> FutureResponse<HttpResponse> {
// fn streaming(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
// fn streaming(req: &HttpRequest) -> FutureResponse<HttpResponse> {
    let query = req.query_string();
    println!("{:?}", query);
    // let email = query.get("email".into());
    // let hash = if let Some(email) = email {
    //     // println!("{:?}", email);
    //     Md5::new().chain(email).result()
    //     // let mut hasher = Md5::new();
    //     // hasher.input(email);
    //     // hasher.result()
    // } else {
    //     return Box::new(future::result::<HttpResponse, Error>(Ok(HttpResponse::build(http::StatusCode::NOT_FOUND).finish())))
    // };

    println!("path: {}", path);
    let hash = Md5::new().chain(path.as_ref()).result();
    // send client request
    client::ClientRequest::get(format!("https://www.gravatar.com/avatar/{:x}?{}", hash, query))
        .finish().unwrap()
        .send()                         // <- connect to host and send request
        .map_err(Error::from)           // <- convert SendRequestError to an Error
        .and_then(|resp| {              // <- we received client response
            Ok(HttpResponse::Ok()
               // read one chunk from client response and send this chunk to a server response
               // .from_err() converts PayloadError to an Error
               .body(Body::Streaming(Box::new(resp.payload().from_err()))))
        })
        .responder()
}

fn main() {
    // ::std::env::set_var("RUST_LOG", "actix_web=info");
    // env_logger::init();
    let sys = actix::System::new("http-proxy");

    server::new(|| {
        App::new()
            // .middleware(middleware::Logger::default())
            // .resource("/", |r| r.f(streaming))
            .resource("/{path}", |r| r.method(http::Method::GET).with(streaming))
    }).workers(1)
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();

    // println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
