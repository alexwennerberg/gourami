// auth functions]
#[macro_use]
extern crate diesel;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

use std::convert::Infallible;

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
use std::sync::{Arc, Mutex};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

mod db;
mod session;

// We use a global shared sqlite connection because it's simple and performance is not 
// very important

fn pooled_sqlite() -> SqlitePool {
    let manager = ConnectionManager::<SqliteConnection>::new(std::env::var("DATABASE_URL").unwrap());
    Pool::new(manager).expect("Postgres connection pool could not be created")
}


lazy_static! {
    static ref POOL: SqlitePool = pooled_sqlite();
}

// fn POOL.get().unwrap() -> diesel::SqliteConnection {
//     return *POOL.get().unwrap();


#[derive(Template)]
#[template(path = "user.html")] 
struct UserTemplate<'a>{
    global: Global<'a>,
    page: &'a str,
    notes: Vec<Note>,
    user: &'a User
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
    user: User,
    logged_in: bool,
}

impl<'a> Global<'a> {
    fn from_session(session: Option<Session>) -> Self {
        match session {
            Some(s) => Global {
            logged_in: true,
            title: "gourami",
            user: s.user.clone(),
        },
            None => Global {
            logged_in: false,
            title: "gourami",
            user: User::default(),
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

fn delete_note(session: Option<Session>, note_id: i32) -> impl Reply {
    use db::schema::notes::dsl::*;
    diesel::delete(notes.filter(id.eq(note_id))).execute(&POOL.get().unwrap()).unwrap();
    warp::redirect::redirect(warp::http::Uri::from_static("/"))
}

#[derive(Deserialize)]
struct NewNoteRequest {
    note_input: String, // has to be String
}

fn new_note(session: Option<Session>, req: NewNoteRequest) -> impl Reply {
    use db::schema::notes::dsl::*;
    // create activitypub activity object
    // TODO -- micropub?
    if let Some(s) = session {
        let new_note = NoteInput{
            creator_id: s.user.id,
            parent_id: None,
            content: req.note_input.clone(), // how to avoid clone here?
        };
        insert_into(notes).values(new_note).execute(&POOL.get().unwrap()).unwrap();
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

fn register_page() -> impl warp::Reply {
    let global = Global::from_session(None); 
    // TODO -- do... something if session is not none
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
    insert_into(users).values(new_user).execute(&POOL.get().unwrap()).unwrap();

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
    let global = Global::from_session(None); 
    render_template(&LoginTemplate{page: "login", login_failed: false, global:global})
}

fn do_login(form: LoginForm) -> impl Reply {
    if let Some(cookie) = Session::authenticate(&POOL.get().unwrap(), &form.username, &form.password) {
        http::Response::builder()
            .status(http::StatusCode::FOUND)
            .header(http::header::LOCATION, "/")
            .header(
                http::header::SET_COOKIE,
                format!("EXAUTH={}; SameSite=Strict; HttpOpnly", cookie),
        )
        .body(Body::empty()).unwrap()
    } else {
        let global = Global::from_session(None); 
        render_template(&LoginTemplate{page: "login", login_failed: true, global:global})
        // TODO -- better error handling
    }
}

fn render_timeline(session: Option<Session>) -> impl Reply {
    // no session -- anonymous
    let global = Global::from_session(session); 
    use db::schema::notes::dsl::*;
    let results = notes
        .load::<Note>(&POOL.get().unwrap())
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
    // cors filters etc
    let session_filter = move || session::create_session_filter().clone();

    use warp::{path, body::form};

    let home = warp::path::end()
        .and(session_filter())
        .map(render_timeline);

    // auth functions
    let register_page = path("register")
        .map(|| register_page());

    let do_register = path("register")
        .and(form())
        .map(do_register);

    let login_page = path("login")
        .map(|| login_page());

    let do_login = path("login")
        .and(form())
        .map(do_login);

    // CRUD actions
    let create_note = path("create_note")
        .and(session_filter())
        .and(form())
        .map(new_note);

    let delete_note = session_filter()
        .and(warp::path::param::<i32>())
        .and(warp::path("delete"))
        .map(delete_note);



    let static_files = warp::path("static")
        .and(warp::fs::dir("./static"));

    // https://github.com/seanmonstar/warp/issues/42 -- how to set up diesel
    // TODO set content length limit 
    // TODO redirect via redirect in request
    // TODO secure against xss
        // used for api based authentication
    // let api_filter = session::create_session_filter(&POOL.get());
    let html_renders = home.or(login_page).or(register_page);
    let forms = login_page.or(do_register).or(do_login).or(create_note).or(delete_note);
    // let api
    // catch all for any other paths
    let not_found = warp::any().map(|| "404 not found");

    let routes = warp::get().and(html_renders)
        .or(
            warp::post()
            .and(warp::body::content_length_limit(1024 * 32))
            .and(forms))
        .or(static_files)
        .or(not_found)
        .with(warp::log("server"));

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
    }
