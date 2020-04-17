#[macro_use]
extern crate diesel;

use gourami_social::*;
use warp::Filter;
use askama::Template;
use warp::http::{self, header, StatusCode};
use warp::hyper::Body;
use warp::reply::{Response, Reply};
use env_logger;
use db::status::{NoteInput, Note};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::insert_into;
use serde::{Deserialize, Serialize};

mod db;

fn establish_connection() -> SqliteConnection {
    let url = ::std::env::var("DATABASE_URL").unwrap();
    let conn = SqliteConnection::establish(&url).unwrap();
    conn
}
// TODO split into separate templates. not sure how
#[derive(Template)]
#[template(path = "timeline.html")] 
struct TimelineTemplate<'a>{
    page: &'a str,
    title: &'a str,
    username: &'a str,
    logged_in: bool,
    notes: Vec<Note>
} 

// impl default

#[derive(Template)]
#[template(path = "notifications.html")] 
struct NotificationTemplate<'a>{
    name: &'a str,
}

pub fn render_template<T: askama::Template>(t: &T) -> Response {
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

fn delete_note(note_id: i32) -> impl Reply {
    use db::schema::note::dsl::*;
    let conn = establish_connection();
    diesel::delete(note.filter(id.eq(note_id))).execute(&conn).unwrap();
    warp::redirect::redirect(warp::http::Uri::from_static("/"))
}

#[derive(Deserialize)]
struct NewNoteRequest {
    note_input: String, // has to be String
}

fn new_note(req: &NewNoteRequest) -> impl Reply {
    use db::schema::note::dsl::*;
    // create activitypub activity object
    // TODO -- micropub?
    let conn = establish_connection();
    let new_note = NoteInput{
        creator_id: 1,
        parent_id: None,
        published: String::from("now"),
        content: req.note_input.clone(), // how to avoid clone here?
    };
    insert_into(note).values(new_note).execute(&conn).unwrap();

    // generate activitypub object from post request
    // send to outbox
    // if request made from web form
    warp::redirect::redirect(warp::http::Uri::from_static("/"))
}

// ActivityPub outbox 
fn send_to_outbox(activity: bool) { // activitystreams object
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
    
    // How does this interact with tokio? who knows!
    let test = warp::path("test").map(|| "Hello world");

    // post
    // user
    // default page -- timeline
    let home = warp::path::end()
        .map(|| render_template(&TimelineTemplate{
            page: "timeline",
            logged_in: true,
            notes: Note::get_for_user(&SqliteConnection::establish("sample.db").unwrap(), 1),
            username: "alex", 
            title: "gourami"}));

    let static_files = warp::path("static")
            .and(warp::fs::dir("./static"));

    // https://github.com/seanmonstar/warp/issues/42 -- how to set up diesel
    // TODO set content length limit 
    // TODO redirect via redirect in request
    // TODO secure against xss
    let create_note = warp::path("create_note")
        .and(warp::body::form())
        .map(|note_req: NewNoteRequest| new_note(&note_req));

    let delete_note = warp::path::param::<i32>()
        .and(warp::path("delete"))
        .map(|note_id| delete_note(note_id));

    // catch all for any other paths
    let not_found = warp::any().map(|| "404 not found");

    let routes = warp::get().and(
        home.or(test).or(static_files).or(not_found))
        .or(warp::post().and(create_note.or(delete_note)));
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
    }
