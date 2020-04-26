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
use db::note::{NoteInput, Note};
use db::note;
use db::user::{RegistrationKey, User, NewUser};
use db::notification::{NewNotification, NewNotificationViewer, Notification, NotificationViewer};
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
pub mod routes;


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
    unread_notifications: i64, // db query on every page
}

impl<'a> Global<'a> {
    fn create(user: Option<User>, page: &'a str) -> Self {
        use db::schema::notification_viewers::dsl::*;
        use diesel::dsl::count;
        match user { 
        Some(u) => {
            let conn = POOL.get().unwrap();
            let unread: i64 = notification_viewers
                .select(count(user_id))
                .filter(user_id.eq(u.id))
                .filter(viewed.eq(false))
                .first(&conn).unwrap();
            Self {
            me: u,
            page: page, // remove leading slash
            page_title: &page[1..], // remove leading slash
            logged_in: true,
            unread_notifications: unread,
            ..Default::default()
        }},
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
            unread_notifications: 0,
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
    use db::schema::notes::dsl as notes;
    // create activitypub activity object
    // TODO -- micropub?
    // if its in reply to something
    let conn = &POOL.get()?;
    let reply = note::get_reply(note_input);
    let parsed_note_text = note::parse_note_text(note_input);
    let new_note = NoteInput{
        user_id: auth_user.id,
        in_reply_to: reply,
        content: parsed_note_text
    };
    insert_into(notes::notes).values(new_note).execute(conn)?;
    // notify person u reply to
    if let Some(r_id) = reply {
        use db::schema::notifications::dsl as notifs;
        use db::schema::notification_viewers::dsl as nv;
        // create reply notification
        let message = format!("@{} created a note in reply to 📝{}", auth_user.username, r_id);
        let new_notification = NewNotification {
        // reusing the same parser for now. rename maybe
            notification_html: note::parse_note_text(&message),
            server_message: false
        };
        insert_into(notifs::notifications).values(new_notification).execute(conn)?;
        // I thinks this may work but worry about multithreading
        let notif_id = notifs::notifications
            .order(notifs::id.desc())
            .select(notifs::id)
            .first(conn).unwrap();
        let user_id = notes::notes
            .select(notes::user_id)
            .find(r_id)
            .first(conn)
            .unwrap(); // TODO 
               // TODO -- notify all members of the thread
        // Mark notes as read
       let new_nv = NewNotificationViewer {
           notification_id: notif_id,
           user_id: user_id,
           viewed: false
        };
        
        insert_into(nv::notification_viewers).values(new_nv).execute(conn)?;

    }
    // generate activitypub object from post request
    // send to outbox
    // add notification
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

// thread. hm
fn get_single_note(note_id: i32) -> Option<Vec<UserNote>> {
    use db::schema::notes::dsl as n;
    use db::schema::users::dsl as u;
     let results = n::notes.inner_join(u::users)
    .filter(n::id.eq(note_id).or(n::in_reply_to.eq(note_id)))
    .load::<(Note, User)>(&POOL.get().unwrap()).unwrap();
    Some(results.into_iter().map(|a| UserNote{note: a.0, username: a.1.username}).collect())
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

#[derive(Template)]
#[template(path = "notifications.html")] 
struct NotificationTemplate<'a>{
    notifs: Vec<RenderedNotif>, // required for redirects.
    global: Global<'a>,
} 

struct RenderedNotif {
    notif: Notification,
    viewed: bool
}
fn render_notifications(auth_user: Option<User>) -> impl Reply {
    use db::schema::notifications::dsl as n;
    use db::schema::notification_viewers::dsl as nv;
    let global = Global::create(auth_user.clone(), "/notifications");
    let conn = &POOL.get().unwrap();
    let my_id = auth_user.unwrap().id;
    let notifs = n::notifications.inner_join(nv::notification_viewers)
        .order(n::id.desc())
        .filter(nv::user_id.eq(my_id))
        .limit(1000) // arbitrary TODO cleanup / paginate
        .load::<(Notification, NotificationViewer)>(conn).unwrap()
        .into_iter()
        .map(|(n, nv)| RenderedNotif{notif: n, viewed: nv.viewed}).collect();
    // mark notifications as read
    diesel::update(
        nv::notification_viewers
        .filter(nv::user_id.eq(my_id))
        .filter(nv::viewed.eq(false)))
        .set(nv::viewed.eq(true)
    ).execute(conn).unwrap();
    render_template(&NotificationTemplate{global: global, notifs: notifs})
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
    note_thread: Vec<UserNote>,
    // thread
}

fn server_info_page(auth_user: Option<User>) -> impl Reply {
    render_template(&ServerInfoTemplate{global: Global::create(auth_user, "/server")})
}

fn note_page(auth_user: Option<User>, note_id: i32, path: FullPath) -> impl Reply {
    let note_thread = get_single_note(note_id);
   if let Some(n) = note_thread {
        render_template(&NoteTemplate{global: Global::create(auth_user, path.as_str()), note_thread: n, page: &note_id.to_string()})
    }
    else {
        render_template(&ErrorTemplate{global: Global::create(auth_user, path.as_str()), error_message: "Note not found"})
    }
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
#[template(path = "edit_user.html")] 
struct UserEditTemplate<'a> {
    global: Global<'a>,
    user: User,
}

fn render_user_edit_page(user: Option<User>, user_name: String) -> impl Reply {
    let u = user.clone().unwrap();
    let global = Global::create(user, "/edit");
    if u.username == user_name || u.admin {
        render_template(&UserEditTemplate{global: global, user: u})
    }
    else {
        render_template(&ErrorTemplate{global: global, error_message: "You don't have permission to edit this page", ..Default::default()})
    }
}


#[derive(Deserialize)]
struct EditForm {
    redirect_url: String,
    bio: Option<String>,
}

fn edit_user(user: Option<User>, user_name: String, f: EditForm) -> impl Reply {
    let u = user.clone().unwrap();
    let conn = &POOL.get().unwrap();
    if u.username == user_name || u.admin {
        use db::schema::users::dsl::*;
        diesel::update(
            users
            .find(u.id))
            .set(bio.eq(&f.bio.unwrap_or(String::new())))
            .execute(conn).unwrap();
    }
    let red: http::Uri = f.redirect_url.parse().unwrap();
    redirect(red)
}

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
