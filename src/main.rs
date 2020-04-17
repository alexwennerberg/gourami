#[macro_use]
extern crate diesel;

use gourami_social::*;
use warp::Filter;
use askama::Template;
use warp::http::{self, header, StatusCode};
use warp::hyper::Body;
use warp::reply::Response;
use env_logger;
use db::status::Status;

mod db;

// TODO split into separate templates. not sure how
#[derive(Template)]
#[template(path = "timeline.html")] 
struct TimelineTemplate<'a>{
    page: &'a str,
    title: &'a str,
    username: &'a str,
    logged_in: bool,
    statuses: Vec<&'a Status>
} 

// impl default

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

fn new_note() {
    // create activitypub activity object
    // TODO -- micropub?
    // generate activitypub object from post request
    // send to outbox
}

// ActivityPub outbox 
fn outbox() {
    // fetch/store from db.
    // db objects need to serialize/deserialize this object
    // if get -> fetch from db
    // if post -> put to db, send to inbox of followers
    // send to inbox of followers
}

// ActivityPub inbox
fn inbox() {
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
        .map(|| reply(&TimelineTemplate{
            page: "timeline",
            logged_in: true,
            statuses: vec![],
            username: "alex", 
            title: "gourami"}));

    let static_files = warp::path("static")
            .and(warp::fs::dir("./static"));

    let routes = warp::get().and(home.or(test).or(static_files));
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
