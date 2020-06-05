// on for development work
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use serde_json::Value;
use std::collections::BTreeMap;
use std::convert::Infallible;
use zxcvbn::zxcvbn;

use warp::filters::path::FullPath;
use warp::http;
use warp::hyper::Body;
use warp::redirect::redirect;
use warp::{Filter, Rejection, Reply};

use askama::Template;
pub use db::conn::POOL;
use db::note;
use db::note::{Note, NoteInput};
use db::server_mutuals::ServerMutual;
use db::user::{NewUser, RegistrationKey, User, UserNameId};
use diesel::insert_into;
use diesel::prelude::*;
use hyper;
use serde::{Deserialize, Serialize};
use session::Session;

use serde_urlencoded;

pub mod ap;
mod db;
pub mod error;
pub mod routes;
mod session;

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate<'a> {
    global: Global<'a>,
    error_message: &'a str,
}

#[derive(Template)]
#[template(path = "edit_user.html")]
struct UserEditTemplate<'a> {
    global: Global<'a>,
    user: User,
}

// TODO split into separate templates. not sure how
#[derive(Template)]
#[template(path = "timeline.html")]
struct TimelineTemplate<'a> {
    global: Global<'a>,
    notes: Vec<UserNote>,
    params: &'a GetPostsParams,
    user: Option<User>
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate<'a> {
    login_failed: bool, // required for redirects.
    global: Global<'a>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate<'a> {
    keyed: bool,
    key: &'a str,
    global: Global<'a>,
}

#[derive(Template)]
#[template(path = "server_info.html")]
struct ServerInfoTemplate<'a> {
    global: Global<'a>,
    users: Vec<User>,
    server_mutuals: Vec<ServerMutual>,
}

const PAGE_SIZE: i64 = 50;

struct Global<'a> {
    // variables used on all pages w header
    title: &'a str,
    page: &'a str,
    page_num: i64,
    page_title: &'a str,
    me: User,
    has_more: bool,
    logged_in: bool,
}

impl<'a> Global<'a> {
    fn create(user: Option<User>, page: &'a str) -> Self {
        use diesel::dsl::count;
        match user {
            Some(u) => {
                let conn = POOL.get().unwrap();
                Self {
                    me: u,
                    page: page,             // remove leading slash
                    page_title: &page[1..], // remove leading slash
                    logged_in: true,
                    ..Default::default()
                }
            }
            None => Self {
                page: page,
                ..Default::default()
            },
        }
    }
}
impl<'a> Default for Global<'a> {
    fn default() -> Self {
        Global {
            title: "gourami", // todo set with config
            me: User::default(),
            page: "",
            page_num: 1,
            page_title: "",
            logged_in: false,
            has_more: false,
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
    redirect_url: String,
}

fn delete_note(note_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    use db::schema::notes::dsl::*;
    diesel::delete(notes.filter(id.eq(note_id))).execute(&POOL.get()?)?;
    Ok(())
}

#[derive(Deserialize)]
struct NewNoteRequest {
    note_input: String, // has to be String
    redirect_url: String,
    neighborhood: Option<String>, // "on" TODO -- add a custom serialization here
}
use tokio::sync::mpsc::UnboundedSender;

async fn handle_new_note_form(
    u: Option<User>,
    f: NewNoteRequest,
    sender: UnboundedSender<(Value, Vec<String>)>,
) -> Result<impl Reply, Rejection> {
    match u {
        Some(u) => {
            let n = new_note(&u, &f.note_input, f.neighborhood.is_some()).unwrap();
            if n.neighborhood {
                let nj = ap::new_note_to_ap_message(&n, &u);
                let destinations = ap::get_connected_remotes()
                    .into_iter()
                    .map(|s| s.inbox_url)
                    .collect();
                sender.send((nj.unwrap(), destinations)).ok();
            }
            let red_url: http::Uri = f.redirect_url.parse().unwrap();
            Ok(redirect(red_url))
        }
        None => Ok(redirect(http::Uri::from_static("error"))),
    }
}

pub fn new_note(
    auth_user: &User,
    note_input: &str,
    neighborhood: bool,
) -> Result<Note, Box<dyn std::error::Error>> {
    use db::schema::notes::dsl as notes;
    use db::schema::users::dsl as u;
    // create activitypub activity object
    // TODO -- micropub?
    // if its in reply to something
    let conn = &POOL.get()?;
    let reply = note::get_reply(note_input);
    let mentions = note::get_mentions(note_input);
    let parsed_note_text = note::parse_note_text(note_input);
    let new_note = NoteInput {
        user_id: auth_user.id,
        in_reply_to: reply,
        content: parsed_note_text,
        neighborhood: neighborhood,
    };
    let mut inserted_note: Note = conn.transaction(|| {
        insert_into(notes::notes).values(&new_note).execute(conn)?;
        notes::notes.order(notes::id.desc()).first(conn)
    })?;
    // add note url
    diesel::update(notes::notes.filter(notes::id.eq(inserted_note.id)))
        .set(notes::remote_id.eq(note::get_url(inserted_note.id)))
        .execute(conn)?;
    inserted_note.remote_id = Some(note::get_url(inserted_note.id));
    Ok(inserted_note)
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
        render_template(&RegisterTemplate {
            keyed: keyed,
            key: key_str,
            global: global,
        })
    } else {
        render_template(&RegisterTemplate {
            keyed: keyed,
            key: key_str,
            global: global,
        })
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
            return Err("Please come up with a more secure password.");
        }
        Ok(())
    }
}

// TODO move all authentication
fn do_register(form: RegisterForm, query_params: serde_json::Value) -> impl Reply {
    let conn = &POOL.get().unwrap();
    use db::schema::users::dsl::*;
    if form.validate().is_err() {
        // TODO catch better
        return do_login(LoginForm {
            username: form.username,
            password: form.password,
        });
    }
    if let Some(k) = query_params.get("key") {
        let k_string = &k.as_str().unwrap();
        let keyed = RegistrationKey::is_valid(conn, k_string);
        RegistrationKey::clear_key(conn, k_string);
        if keyed {
            let hash = bcrypt::hash(&form.password, bcrypt::DEFAULT_COST).unwrap();
            let new_user = NewUser {
                username: &form.username,
                password: &hash,
                email: &form.email,
            };
            insert_into(users).values(new_user).execute(conn).unwrap();
        }
    }
    // database
    // not good
    do_login(LoginForm {
        username: form.username,
        password: form.password,
    })
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

fn login_page() -> impl Reply {
    // dont let you access this page if logged in
    render_template(&LoginTemplate {
        login_failed: false,
        global: Global {
            page: "login",
            ..Default::default()
        },
    })
}

fn do_login(form: LoginForm) -> impl Reply {
    if let Some(cookie) =
        Session::authenticate(&POOL.get().unwrap(), &form.username, &form.password)
    {
        // 1 year cookie expiration
        http::Response::builder()
            .status(http::StatusCode::FOUND)
            .header(http::header::LOCATION, "/")
            .header(
                http::header::SET_COOKIE,
                format!(
                    "EXAUTH={}; MAX-AGE=31536000; SameSite=Strict; HttpOpnly",
                    cookie
                ),
            )
            .body(Body::empty())
            .unwrap()
    } else {
        render_template(&LoginTemplate {
            login_failed: true,
            global: Global {
                page: "login",
                ..Default::default()
            },
        })
    }
}

fn do_logout(cook: String) -> impl Reply {
    use db::schema::sessions::dsl::*;
    diesel::delete(sessions.filter(cookie.eq(cook)))
        .execute(&POOL.get().unwrap())
        .unwrap();
    redirect(warp::http::Uri::from_static("/"))
}

#[derive(Serialize, Deserialize)]
struct GetPostsParams {
    #[serde(default = "default_page")]
    page: i64,
    neighborhood: Option<bool>,
    all: Option<bool>,
    search_string: Option<String>,
    user_id: Option<i32>,
    username: Option<String>,
    note_id: Option<i32>,
}

fn default_page() -> i64 {
    1
}

pub struct UserNote {
    note: Note,
    username: String,
    user_id: i32,
}

fn get_single_note(note_id: i32) -> Option<Vec<UserNote>> {
    // Get note and all children, recursively
    let conn = &POOL.get().unwrap();
    let results: Vec<(Note, UserNameId)> = diesel::sql_query(format!(
        r"with recursive tc( p )
      as ( values({})
          union select id from notes, tc
               where notes.in_reply_to = tc.p
                     )
                     select notes.*, users.username, users.id from notes 
                     join users on notes.user_id = users.id 
                     where notes.id in tc",
        note_id
    ))
    .load(conn)
    .unwrap();
    Some(
        results
            .into_iter()
            .map(|a| {
                // the ids are swapped for some reason
                UserNote {
                    note: a.0,
                    username: a.1.username,
                    user_id: a.1.id,
                }
            })
            .collect(),
    )
}

fn get_local_users() -> Result<Vec<User>, diesel::result::Error> {
    use db::schema::users::dsl as u;
    let conn = &POOL.get().unwrap();
    let users = u::users.filter(u::remote_user.eq(false)).load(conn);
    users
}

/// We have to do a join here
fn get_notes(logged_in: bool, params: &GetPostsParams) -> Result<Vec<UserNote>, diesel::result::Error> {
    use db::schema::notes::dsl as n;
    use db::schema::users::dsl as u;
    // TODO -- add whether this is complete so i can page properly
    if logged_in { 
        if let Some(n_id) = params.note_id {
            return Ok(get_single_note(n_id).unwrap()) // TODO filter replies logged out
        }
    }
    let mut query = n::notes
        .inner_join(u::users)
        .order(n::id.desc())
        .limit(PAGE_SIZE)
        .offset((params.page - 1) * PAGE_SIZE)
        .into_boxed();
    if let Some(u_id) = params.user_id {
        query = query.filter(u::id.eq(u_id));
    }
    if let Some(search) = params.search_string.clone() {
        query = query.filter(n::content.like(format!("%{}%", search)));
    }
    // this is wonky
    if !logged_in {
        query = query.filter(n::neighborhood.eq(true))
    }
    if !params.all.is_some() {
        match params.neighborhood {
            Some(true) => query = query.filter(n::neighborhood.eq(true)),
            _ => query = query.filter(n::is_remote.eq(false)),
            // maybe somethign with replies
        }
    }
    let results = query.load::<(Note, User)>(&POOL.get().unwrap()).unwrap(); // TODO get rid of unwrap
    Ok(results
        .into_iter()
        .map(|a| UserNote {
            note: a.0,
            username: a.1.username,
            user_id: a.1.id,
        })
        .collect())
}

fn render_timeline(
    auth_user: Option<User>,
    params: &GetPostsParams,
    url_path: FullPath,
    notes: Result<Vec<UserNote>, diesel::result::Error>,
) -> impl Reply {
    // no session -- anonymous
    // pulls a bunch of data i dont really need
    let url_with_params = &format!(
        "{}?{}",
        url_path.as_str(),
        serde_urlencoded::to_string(params).unwrap()
    );
    let mut header = Global::create(auth_user, url_with_params);
    header.page_title = "";
    // wonky
    use db::schema::users::dsl as u;
    let user = match params.user_id {
        Some(u_id) => u::users.filter(u::id.eq(u_id)).first(&POOL.get().unwrap()).ok(),
        None => None,
    };
    header.page_num = params.page;
    // TODO -- ignore neighborhood replies
    match notes {
        Ok(n) => {
            // NOTE -- breaks when  exactly 50 notes
            if n.len() == PAGE_SIZE as usize {
                header.has_more = true;
            }
            render_template(&TimelineTemplate {
                global: header,
                notes: n,
                user: user,
                params: params
            })
        }
        _ => render_template(&ErrorTemplate {
            global: header,
            error_message: "Could not fetch notes",
            ..Default::default()
        }),
    }
}

impl<'a> Default for ErrorTemplate<'a> {
    fn default() -> Self {
        Self {
            global: Global::default(),
            error_message: "An error occured. Please report to site admin.",
        }
    }
}

fn server_info_page(auth_user: Option<User>) -> impl Reply {
    let users = get_local_users().unwrap();
    let server_mutuals = ap::get_connected_remotes();
    render_template(&ServerInfoTemplate {
        global: Global::create(auth_user, "/server"),
        users: users,
        server_mutuals: server_mutuals,
    })
}

fn render_user_edit_page(user: Option<User>, user_name: String) -> impl Reply {
    let u = user.clone().unwrap();
    let global = Global::create(user, "/edit");
    if u.username == user_name || u.admin {
        render_template(&UserEditTemplate {
            global: global,
            user: u,
        })
    } else {
        render_template(&ErrorTemplate {
            global: global,
            error_message: "You don't have permission to edit this page",
            ..Default::default()
        })
    }
}

pub fn get_outbox() {}

use warp::Buf;

pub async fn post_inbox(
    buf: impl Buf,
    headers: http::header::HeaderMap,
) -> Result<impl Reply, Infallible> {
    // TODO check if it is a create note message
    let message: Value = serde_json::from_slice(buf.bytes()).unwrap(); // TODO error handling
    debug!("received request {:?}", message);
    let mut headersbtree: BTreeMap<String, String> = BTreeMap::new();
    // convert to btree
    for (k, v) in headers.iter() {
        headersbtree.insert(k.as_str().to_owned(), v.to_str().unwrap().to_owned());
    }
    let domain = ap::verify_ap_message("POST", "/inbox", headersbtree)
        .await
        .unwrap(); // get the domain -- we use this for forgery checking
    let msg_type = message.get("type").unwrap().as_str().unwrap();
    debug!("Received ActivityPub message of type {}", msg_type); // TODO improve logging
    match msg_type {
        "Create" => ap::process_create_note(message, &domain).unwrap(),
        "Follow" => ap::process_follow(message, &domain).await.unwrap(),
        "Accept" => ap::process_accept(message, &domain).await.unwrap(),
        _ => (),
    }
    // thtas it!
    Ok("ok!")
}

#[derive(Deserialize)]
struct EditForm {
    redirect_url: String,
    bio: String,
    website: String,
    show_email: Option<String>,
    email: String,
}

fn edit_user(user: Option<User>, user_name: String, f: EditForm) -> impl Reply {
    let u = user.clone().unwrap();
    let conn = &POOL.get().unwrap();
    if u.username == user_name || u.admin {
        use db::schema::users::dsl::*;
        diesel::update(users.find(u.id))
            .set((
                bio.eq(&f.bio),
                email.eq(&f.email),
                website.eq(&f.website),
                show_email.eq(&f.show_email.is_some()),
            ))
            .execute(conn)
            .unwrap();
    }
    let red: http::Uri = f.redirect_url.parse().unwrap();
    redirect(red)
}

async fn handle_rejection(_: Rejection) -> Result<impl Reply, Infallible> {
    Ok(render_template(&ErrorTemplate {
        global: Global::create(None, "error"),
        error_message:
            "You do not have access to this page, it does not exist, or something went wrong.",
    }))
}
