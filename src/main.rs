use warp::Filter;
use askama::Template;
use warp::http::{self, header, StatusCode};
use warp::hyper::Body;
use warp::reply::Response;

#[derive(Template)]
#[template(path = "hello.html")] 
struct HelloTemplate<'a>{
    name: &'a str,
}

pub fn reply<T: askama::Template>(t: &T) -> Response {
    match t.render() {
        Ok(body) => http::Response::builder()
            .status(StatusCode::OK)
            // TODO add headers etc
            .body(body.into()),
        Err(_) => http::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty()),
    }
    .unwrap()
}

#[tokio::main]
async fn main() {
    let hello = warp::path!("test")
        .map(|| reply(&HelloTemplate{name: "world"}));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
