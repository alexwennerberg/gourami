use warp::Filter;
use askama::Template;
use warp::http::{self, header, StatusCode};
use warp::hyper::Body;
use warp::reply::Response;
use env_logger;

#[derive(Template)]
#[template(path = "timeline.html")] 
struct HomeTemplate<'a>{
    name: &'a str,
    _parent: BaseTemplate<'a>
} 


// all the info on every page
#[derive(Template)]
#[template(path = "base.html")] 
struct BaseTemplate<'a>{
    title: &'a str,
}

#[derive(Template)]
#[template(path = "notifications.html")] 
struct NotificationTemplate<'a>{
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
    env_logger::init();
    let notifications = warp::path("notifications");
    let test = warp::path("test").map(|| "Hello world");
    // post
    // user
    // default page -- timeline
    let home = warp::path::end()
        .map(|| reply(&HomeTemplate{name: "world", _parent: BaseTemplate { title: "gourami"}}));

    let static_files = warp::path("static")
            .and(warp::fs::dir("./static"));

    let routes = warp::get().and(home.or(test).or(static_files));
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
