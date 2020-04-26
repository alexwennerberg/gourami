use crate::*;
use db::user::User;
use log::{debug, error};
use rand::thread_rng;
use rand::Rng;
use rand::distributions::Alphanumeric;
use diesel::sqlite::SqliteConnection;
use warp::filters::{cookie, BoxedFilter};


#[derive(Queryable)]
pub struct Session {
    // dbpool maybe
    id: i32,
    cookie: String,
    user_id: i32,
    created_time: String,
}

// TODO -- figure out if database pooling is strictly necessary for security 

impl Session {
    /// Attempt to authenticate a user for this session.
    ///
    /// If the username and password is valid, create and return a session key.
    /// If authentication fails, simply return None.
    pub fn authenticate(conn: &SqliteConnection, username: &str, password: &str) -> Option<String> {
        if let Some(user) = User::authenticate(conn, username, password) {
            debug!("User authenticated");
            let secret = thread_rng().sample_iter(&Alphanumeric).take(48).collect();
            use crate::db::schema::sessions::dsl::*;
            let result = diesel::insert_into(sessions)
                .values((user_id.eq(user.id), cookie.eq(&secret)))
                .execute(conn);
            if let Ok(_) = result {
                return Some(secret);
            } else {
                error!(
                    "Failed to create session for {}: {:?}",
                    user.username, result,
                );
            }
        }
        None
    }
    pub fn from_key(sess: Option<String>) -> Option<User> {
        if let Some(sessionkey) = sess {
            use db::schema::sessions::dsl as s;
            use db::schema::users::dsl as u;
            let result  = u::users
                    .inner_join(s::sessions)
                    .filter(s::cookie.eq(sessionkey))
                    .first::<(User,Session)>(&POOL.get().unwrap())
                        .ok();
            match result {
                Some(r) => Some(r.0),
                None => None
            }
        }
        else {
            // so we don't have to query db when key isnt present
            None
        }

    }
}

pub fn create_session_filter(optional: bool) -> BoxedFilter<(Option<User>,)> {
    if optional {
        cookie::optional("EXAUTH")
        .map(move |key: Option<String>| {Session::from_key(key)})
        .boxed()
    } else {
        cookie::cookie("EXAUTH")
        .and_then(|key: String| async move {
            let s = Session::from_key(Some(key));
            if s.is_none() {
                Err(warp::reject::reject()) // todo -- add custom rejection
            }
            else {
                Ok(Some(s.unwrap()))
            }
            })
        .boxed()
    }
}
