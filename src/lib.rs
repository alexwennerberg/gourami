// auth functions]
#[macro_use]
extern crate diesel;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;

use std::convert::Infallible;
use zxcvbn::zxcvbn;

use warp::{reject, reject::Reject, Reply, Filter, Rejection};
use warp::{redirect::redirect};
use warp::filters::path::FullPath;
use warp::http;
use warp::hyper::Body;

use hyper;
use askama::Template;
use env_logger;
use db::note::{NoteInput, Note};
use db::note;
use db::user::{RegistrationKey, User, NewUser};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::insert_into;
use serde::{Deserialize};
use session::{Session};
use diesel::r2d2::{ConnectionManager, Pool};

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

mod db;
mod session;
mod ap;


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


struct Global<'a> { // variables used on all pages w header
    title: &'a str,
    page: &'a str,
    page_title: &'a str,
    me: User,
    logged_in: bool,
}

impl<'a> Global<'a> {
    fn create(user: Option<User>, page: &'a str) -> Self {
        match user { 
        Some(u) => Self {
            me: u,
            page: page, // remove leading slash
            page_title: &page[1..], // remove leading slash
            logged_in: true,
            ..Default::default()
        },
        None => Self {
            page: page,
            ..Default::default()
        }
        }
    }
}
impl<'a> Default for Global<'a> {
    fn default() -> Self {
        Global {
            title: "gourami", // todo set with config
            me: User::default(),
            page: "",
            page_title: "",
            logged_in: false,
        }
    }
}

pub fn render_template<T: askama::Template>(t: &T) -> http::Response<hyper::body::Body> {
    match t.render() {
        Ok(body) => http::Response::builder()
            .status(http::StatusCode::OK)
            // TODO add headers etc
            .body(body.into()),
        Err(_) => http::Response::builder()
            // pretty sure it will never get here
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty()),
    }
    .unwrap()
}

#[derive(Deserialize)]
struct DeleteNoteRequest {
    note_id: i32, // has to be String
    redirect_url: String
}

fn delete_note(note_id: i32)-> Result<(), Box<dyn std::error::Error>> {
    use db::schema::notes::dsl::*;
    diesel::delete(notes.filter(id.eq(note_id))).execute(&POOL.get()?)?;
    Ok(())
}

#[derive(Deserialize)]
struct NewNoteRequest {
    note_input: String, // has to be String
    redirect_url: String,
}

fn new_note(auth_user: User, note_input: &str) -> Result<(), Box<dyn std::error::Error>> {
    use db::schema::notes::dsl::*;
    // create activitypub activity object
    // TODO -- micropub?
    let new_note = NoteInput{
        user_id: auth_user.id,
        parent_id: None,
        content: note::parse_note_text(note_input), 
    };
    insert_into(notes).values(new_note).execute(&POOL.get()?)?;
    // generate activitypub object from post request
    // send to outbox
    // if request made from web form
    Ok(())
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
    keyed: bool,
    key: &'a str,
    global: Global<'a>,
}

#[derive(Deserialize)]
struct QueryParams {
    key: Option<String>,
}

fn register_page(query_params: QueryParams) -> impl warp::Reply {
    let mut keyed = false;
    let mut key_str = "";
    let global = Global::create(None, "register");
    if let Some(k) = query_params.key {
        key_str = k.as_str();
        keyed = RegistrationKey::is_valid(&POOL.get().unwrap(), &key_str);
        render_template(&RegisterTemplate{keyed: keyed, key: key_str, global: global})
    }
    else {
        render_template(&RegisterTemplate{keyed: keyed, key: key_str, global: global})
    }
}


#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    password: String,
    email: String,
}


impl RegisterForm {
    fn validate(&self) -> Result<(), &'static str> {
        if zxcvbn(&self.password, &[]).unwrap().score() < 1 {
            return Err("Please come up with a more secure password.")
        }
        Ok(())
    }
}

// TODO move all authentication
fn do_register(form: RegisterForm, query_params: serde_json::Value) -> impl Reply {
    let conn = &POOL.get().unwrap();
    use db::schema::users::dsl::*;
    if form.validate().is_err(){ // TODO catch better
        return do_login(LoginForm{username: form.username, password: form.password})
    }
    if let Some(k) = query_params.get("key") {
        let k_string = &k.as_str().unwrap();
        let keyed = RegistrationKey::is_valid(conn, k_string);
        RegistrationKey::clear_key(conn, k_string);
        if keyed {
            let hash = bcrypt::hash(&form.password, bcrypt::DEFAULT_COST).unwrap();
            let new_user = NewUser {username: &form.username, password: &hash, email: &form.email};
            // todo data validation
            insert_into(users)
                .values(new_user)
                .execute(conn).unwrap();

        // insert into database
        }
    }
    // not good
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
    login_failed: bool, // required for redirects.
    global: Global<'a>,
} 

fn login_page() -> impl Reply {
    // dont let you access this page if logged in
    render_template(&LoginTemplate{login_failed: false, global: Global{page: "login", ..Default::default()}})
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
        render_template(&LoginTemplate{login_failed: true, global:Global{page: "login", ..Default::default()}})
    }
}

fn do_logout(cookie: String) -> impl Reply {
    use db::schema::sessions::dsl::*;
    diesel::delete(sessions.filter(cookie.eq(cookie))).execute(&POOL.get().unwrap()).unwrap();
    redirect(warp::http::Uri::from_static("/"))
}

// TODO split into separate templates. not sure how
#[derive(Template)]
#[template(path = "timeline.html")] 
struct TimelineTemplate<'a>{
    global: Global<'a>,
    notes: Vec<UserNote>,
} 

#[derive(Deserialize)]
struct GetPostsParams {
    #[serde(default = "default_page")]
    page_num: i64,
    user_id: Option<i32>
}
fn default_page() -> i64 {
    1
}

impl Default for GetPostsParams {
    fn default() -> Self {
        GetPostsParams {
            page_num: 1,
            user_id: None
        }
    }
}


pub struct UserNote {
    note: Note,
    username: String,
}

fn get_single_note(note_id: i32) -> Option<UserNote> {
    use db::schema::notes::dsl::*;
    use db::schema::users::dsl::*;
    use db::schema;
     let note = notes.inner_join(users)
    .filter(schema::notes::id.eq(note_id))
    .first::<(Note, User)>(&POOL.get().unwrap()).unwrap();
    Some(UserNote{note: note.0, username: note.1.username})
}

/// We have to do a join here
fn get_notes(params: GetPostsParams) -> Result<Vec<UserNote>, diesel::result::Error> {
    use db::schema::notes::dsl::*;
    use db::schema::users::dsl::*;
    use db::schema as s;
    const PAGE_SIZE: i64 = 250;
    let results = notes.inner_join(users)
        .order(s::notes::id.desc())
        .limit(PAGE_SIZE)
        .offset((params.page_num - 1) * PAGE_SIZE)
        .load::<(Note, User)>(&POOL.get().unwrap()).unwrap(); // TODO get rid of unwrap
    Ok(results.into_iter().map(|a| UserNote{note: a.0, username: a.1.username}).collect())
}

fn render_timeline(auth_user: Option<User>, params:GetPostsParams, url_path: FullPath) -> impl Reply {
    // no session -- anonymous
    // pulls a bunch of data i dont really need
    let header = Global::create(auth_user, url_path.as_str());
    let notes = get_notes(params);
    match notes {
        Ok(n) => render_template(&TimelineTemplate{
            global: header,
            notes: n,
        }),
        _ => render_template(&ErrorTemplate{global: header, error_message: "Could not fetch notes", ..Default::default()})
    }

}

#[derive(Template)]
#[template(path = "server_info.html")]
struct ServerInfoTemplate<'a> {
    global: Global<'a>,
}

#[derive(Template)]
#[template(path = "error.html")] 
struct ErrorTemplate<'a> {
    global: Global<'a>,
    error_message: &'a str
}

impl<'a>  Default for ErrorTemplate<'a> {
    fn default() -> Self {
        Self {
            global: Global::default(),
            error_message: "An error occured. Please report to site admin."
        }
    }
}

#[derive(Template)]
#[template(path = "user.html")] 
struct UserTemplate<'a>{
    global: Global<'a>,
    page: &'a str,
    notes: Vec<UserNote>,
    user: User,
} 

#[derive(Template)]
#[template(path = "note.html")] 
struct NoteTemplate<'a> {
    global: Global<'a>,
    page: &'a str,
    note: &'a UserNote,
    // thread
}

fn server_info_page(auth_user: Option<User>) -> impl Reply {
    render_template(&ServerInfoTemplate{global: Global::create(auth_user, "/server")})
}

fn note_page(auth_user: Option<User>, note_id: i32, path: FullPath) -> impl Reply {
    use db::schema::notes::dsl::*;
    use db::schema::users::dsl::*;
    use db::schema;
    let conn = &POOL.get().unwrap();
    let note = get_single_note(note_id);
   if let Some(n) = note {
        render_template(&NoteTemplate{global: Global::create(auth_user, path.as_str()), note: &n, page: &note_id.to_string()})
    }
    else {
        render_template(&ErrorTemplate{global: Global::create(auth_user, path.as_str()), error_message: "Note not found"})
    }
    // TODO -- fetch replies
}

fn user_page(auth_user: Option<User>, user_name: String, params: GetPostsParams, path: FullPath) -> impl Reply {
    let header = Global::create(auth_user, path.as_str());  // maybe if i'm clever i can abstract this away
    use db::schema::users::dsl::*;
    let conn = &POOL.get().unwrap();
    let user: Option<User> = users
        .filter(username.eq(user_name))
        .first::<User>(conn)
        .ok();
    if let Some(u) = user {
        let notes = get_notes(params).unwrap();
        render_template(&UserTemplate{
            global: header,
            page: &u.username,
            user: u.clone(), // TODO stop cloning
            notes: notes
        })
    }
    else {
        render_template(&ErrorTemplate{global: header, error_message: "User not found", ..Default::default()})
    }
}


#[derive(Template)]
#[template(path = "notifications.html")] 
struct NotificationTemplate<'a> {
    // user: Global<'a>,
    page: &'a str,
    // notifications: Vec<Notifications>,
    // thread
}

// fn notification_page(auth_user: &User) -> impl Reply {
//     let global = Global::from_session(session):
//     render_template(&NotificationTemplate{global: global, page: "notifications"}
// }

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    Ok(render_template(&ErrorTemplate{global: Global::create(None, "error"), error_message: "You do not have access to this page, it does not exist, or something went wrong."}))
}


// Url query
#[derive(Deserialize)]
struct Page {
    page_num: i32
}

// TODO -- move this into separate module
#[derive(Debug)]
struct LoggedOut;
impl Reject for LoggedOut {}

pub async fn run_server() {
    env_logger::init();
    // cors filters etc
    
    // NOT TESTED YET
    let public = false; // std::env::var("PUBLIC").unwrap_or("false");
    let session_filter = move || session::create_session_filter(public).clone();

    use warp::{filters::cookie, path, body::json, body::form, filters::query::query};

    // we have to pass the full paths for redirect to work without javascript
    let home = warp::path::end()
        .and(session_filter())
        .and(query())
        .and(path::full())
        .map(render_timeline);

    let user_page = session_filter()
        .and(path!("user" / String))
        .and(form())
        .and(path::full())
        .map(user_page);

    let note_page = session_filter()
        .and(path!("note" / i32))
        .and(path::full())
        .map(note_page);

    let server_info_page = session_filter()
        .and(path("server"))
        .map(server_info_page);

    // auth functions
    let register_page = path("register")
        .and(query())
        .map(register_page);

    let do_register = path("register")
        .and(form())
        .and(query())
        .map(do_register);

    let login_page = path("login")
        .map(|| login_page());

    // TODO redirect these login pages
    let do_login = path("login")
        .and(form())
        .map(do_login);

    let do_logout = path("logout")
        .and(cookie::cookie("EXAUTH"))
        .map(do_logout);

    // CRUD actions
    let create_note = path("create_note")
        .and(session_filter())
        .and(form())
        // Verbose -- see if you can refactor
        .map(|u: Option<User>, f: NewNoteRequest| match u {
            Some(u) => {
                new_note(u, &f.note_input).unwrap(); // TODO fix unwrap
                let red_url: http::Uri = f.redirect_url.parse().unwrap();
                redirect(red_url)},
            None => redirect(warp::http::Uri::from_static("error"))});

    let delete_note = path("delete_note")
        .and(session_filter())
        .and(form())
        .map(|u: Option<User>, f: DeleteNoteRequest | match u {
            Some(u) => {
                delete_note(f.note_id).unwrap(); // TODO fix unwrap
                let red_url: http::Uri = f.redirect_url.parse().unwrap();
                redirect(red_url)},
            None => redirect(warp::http::Uri::from_static("error"))});


    let static_files = warp::path("static")
        .and(warp::fs::dir("./static"));

    // activityPub stuff
    //
    // POST
    let post_user_inbox = path!("user" / String / "inbox.json" )
        .and(json())
        .map(ap::post_user_inbox);

    let post_user_outbox = path!("user" / String / "outbox.json" )
        .and(json())
        .map(ap::post_user_outbox);

    let get_user_outbox = path!("user" / String / "outbox.json" )
        .map(ap::get_user_outbox);

    // let get_user_inbox = path!("user" / String / "outbox.json" )
    //     .and(json())
    //     .map(ap::post_user_outbox);

    let user_followers = path!("user" / String / "followers.json" )
        .map(ap::user_followers);

    let user_following = path!("user" / String / "following.json" )
        .map(ap::user_following);

    // https://github.com/seanmonstar/warp/issues/42 -- how to set up diesel
    // TODO set content length limit 
    // TODO redirect via redirect in request
    // TODO secure against xss
        // used for api based authentication
    // let api_filter = session::create_session_filter(&POOL.get());
    let html_renders = home.or(login_page).or(register_page).or(user_page).or(note_page).or(server_info_page);
    let forms = do_register.or(do_login).or(do_logout).or(create_note).or(delete_note);
    // let api
    // catch all for any other paths

    let routes = warp::get().and(html_renders)
        .or(
            warp::post()
            .and(warp::body::content_length_limit(1024 * 32))
            .and(forms))
        .or(static_files)
        .with(warp::log("server"))
        .recover(handle_rejection);

    match std::env::var("GOURAMI_ENV").unwrap().as_str() {
        "PROD" => warp::serve(routes)
            .tls()
            .cert_path(&std::env::var("CERT_PATH").unwrap())
            .key_path(&std::env::var("KEY_PATH").unwrap())
            .run(([0, 0, 0, 0], 443))
            .await ,
        _ => warp::serve(routes).run(([127,0,0,1], 3030)).await
    }
}
