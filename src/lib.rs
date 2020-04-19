#[macro_use]
extern crate diesel;
#[macro_use] extern crate log;

use warp::{Reply, Filter, Rejection};
use warp::http;
use warp::hyper::Body;
use warp::reply::{Response};
use warp::reject::{custom, not_found};

use hyper;
use askama::Template;
use env_logger;
use db::status::{NoteInput, Note};
use db::user::{User, NewUser};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::insert_into;
use serde::{Deserialize, Serialize};
use session::{Session};

mod db;
mod session;

fn establish_connection() -> SqliteConnection {
    let url = ::std::env::var("DATABASE_URL").unwrap();
    let conn = SqliteConnection::establish(&url).unwrap();
    conn
}
// TODO split into separate templates. not sure how
#[derive(Template)]
#[template(path = "timeline.html")] 
struct TimelineTemplate<'a>{
    global: Global<'a>,
    page: &'a str,
    notes: Vec<Note>,
} 

struct Global<'a> {
    title: &'a str,
    username: String,
    logged_in: bool,
}

impl<'a> Global<'a> {
    fn from_user(user: Option<User>) -> Self {
        match user {
            Some(u) => Global {
            logged_in: true,
            title: "gourami",
            username: u.username.clone(),
        },
            None => Global {
            logged_in: false,
            title: "gourami",
            username: String::from("anonymous"),
        }
        }
    }
}
// impl default

#[derive(Template)]
#[template(path = "notifications.html")] 
struct NotificationTemplate<'a>{
    name: &'a str,
}

pub fn render_template<T: askama::Template>(t: &T) -> http::Response<hyper::body::Body> {
    match t.render() {
        Ok(body) => http::Response::builder()
            .status(http::StatusCode::OK)
            // TODO add headers etc
            .body(body.into()),
        Err(_) => http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty()),
    }
    .unwrap()
}

fn delete_note(note_id: i32) -> impl Reply {
    use db::schema::notes::dsl::*;
    let conn = establish_connection();
    diesel::delete(notes.filter(id.eq(note_id))).execute(&conn).unwrap();
    warp::redirect::redirect(warp::http::Uri::from_static("/"))
}

#[derive(Deserialize)]
struct NewNoteRequest {
    note_input: String, // has to be String
}

fn new_note(auth_cookie: Option<String>, req: &NewNoteRequest) -> impl Reply {
    use db::schema::notes::dsl::*;
    // create activitypub activity object
    // TODO -- micropub?
    if let Some(k) = auth_cookie {
        let conn = establish_connection();
        let user = Session::from_key(&conn, &k).user.unwrap();
        let new_note = NoteInput{
            creator_id: user.id,
            parent_id: None,
            content: req.note_input.clone(), // how to avoid clone here?
        };
        insert_into(notes).values(new_note).execute(&conn).unwrap();
        return warp::redirect::redirect(warp::http::Uri::from_static("/"))
    } else {
        return warp::redirect::redirect(warp::http::Uri::from_static("/"))
    }

    // generate activitypub object from post request
    // send to outbox
    // if request made from web form
}

// ActivityPub outbox 
fn send_to_outbox(activity: bool) { // activitystreams object
    // fetch/store from db.
    // db objects need to serialize/deserialize this object
    // if get -> fetch from db
    // if post -> put to db, send to inbox of followers
    // send to inbox of followers
}


#[derive(Template)]
#[template(path = "register.html")] 
struct RegisterTemplate<'a>{
    page: &'a str,
    global: Global<'a>,
} 

fn register_page() -> impl Reply {
    let global = Global::from_user(None); 
    render_template(&RegisterTemplate{page: "register", global:global})
}


#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    password: String,
    email: String,
}


impl RegisterForm {
    fn validate(self) -> Result<Self, &'static str> {
        if self.email.is_empty() {
            Err("A email must be given")
        } else if self.password.len() < 3 {
            Err("Please use a better password")
        } else {
            Ok(self)
        }
    }
}

// TODO move all authentication
fn do_register(form: RegisterForm) -> impl Reply{
    use db::schema::users::dsl::*;
    let hash = bcrypt::hash(&form.password, bcrypt::DEFAULT_COST).unwrap();
    let new_user = NewUser {username: &form.username, password: &hash, email: &form.email};
    // todo data validation
    let conn = establish_connection();
    insert_into(users).values(new_user).execute(&conn).unwrap();

    // insert into database
    do_login(LoginForm{username: form.username, password: form.password})
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}


#[derive(Template)]
#[template(path = "login.html")] 
struct LoginTemplate<'a>{
    page: &'a str,
    login_failed: bool,
    global: Global<'a>,
} 

fn login_page() -> impl Reply {
    // dont let you access this page if logged in
    let global = Global::from_user(None); 
    render_template(&LoginTemplate{page: "login", login_failed: false, global:global})
}

fn do_login(form: LoginForm) -> impl Reply {
    let conn = establish_connection();
    if let Some(cookie) = Session::authenticate(&conn, &form.username, &form.password) {
        http::Response::builder()
            .status(http::StatusCode::FOUND)
            .header(http::header::LOCATION, "/")
            .header(
                http::header::SET_COOKIE,
                format!("EXAUTH={}; SameSite=Strict; HttpOpnly", cookie),
        )
        .body(Body::empty()).unwrap()
    } else {
        let global = Global::from_user(None); 
        render_template(&LoginTemplate{page: "login", login_failed: true, global:global})
        // TODO -- better error handling
    }
}

fn timeline(auth_cookie: Option<String>) -> impl Reply {
    // no session -- anonymous
    let conn = establish_connection();
    let session = Session::from_key(&conn, &auth_cookie.unwrap());
    let global = Global::from_user(session.user); 
    //ownership?
    use db::schema::notes::dsl::*;
    let results = notes
        .load::<Note>(&conn)
        .expect("Error loading posts");
    render_template(&TimelineTemplate{
    page: "timeline",
    global: global,
    notes: results,
    })

}
// fn do_logout(mut session: Session) -> Result<impl Reply, Rejection> {
//     session.clear();
//     Response::builder()
//         .status(StatusCode::FOUND)
//         .header(header::LOCATION, "/")
//         .header(
//             header::SET_COOKIE,
//             "EXAUTH=; Max-Age=0; SameSite=Strict; HttpOpnly",
//         )
//         .body(b"".to_vec())
//         .map_err(custom)
// }

fn logout() {
}
// ActivityPub inbox
fn inbox() {
}

pub async fn run_server() {
    env_logger::init();

    let notifications = warp::path("notifications");
    
    // How does this interact with tokio? who knows!
    let test = warp::path("test").map(|| "Hello world");

    let register_page = warp::path("register").map(|| register_page());
    let do_register = warp::path("register")
        .and(warp::body::form())
        .map(|f: RegisterForm| do_register(f));

    let login_page = warp::path("login").map(|| login_page());
    let do_login = warp::path("login")
        .and(warp::body::form())
        .map(|f: LoginForm| do_login(f));

    let logout = warp::path("logout").map(|| "Hello from logout");

    // post
    // user
    // default page -- timeline
    let home = warp::path::end()
        .and(warp::filters::cookie::optional("EXAUTH"))
        .map(|auth_cookie| timeline(auth_cookie)); 

    let static_files = warp::path("static")
            .and(warp::fs::dir("./static"));

    // https://github.com/seanmonstar/warp/issues/42 -- how to set up diesel
    // TODO set content length limit 
    // TODO redirect via redirect in request
    // TODO secure against xss
    let create_note = warp::path("create_note")
        .and(warp::filters::cookie::optional("EXAUTH"))
        .and(warp::body::form())
        .map(|auth_cookie, note_req: NewNoteRequest| new_note(auth_cookie, &note_req));

    let delete_note = warp::path::param::<i32>()
        .and(warp::path("delete"))
        .map(|note_id| delete_note(note_id));

    // catch all for any other paths
    let not_found = warp::any().map(|| "404 not found");

    let routes = warp::get().and(
        home.or(test).or(static_files).or(login_page).or(register_page).or(not_found))
        .or(warp::post().and(create_note.or(delete_note).or(do_login).or(do_register)))
        .with(warp::log("server"));
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
    }
