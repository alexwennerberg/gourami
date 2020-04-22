use crate::*;
use db::user::User;
use log::{debug, error};
use rand::thread_rng;
use rand::Rng;
use rand::distributions::Alphanumeric;
use diesel::sqlite::SqliteConnection;
use warp::filters::{cookie, BoxedFilter};


pub struct Session {
    // dbpool maybe
    pub id: i32,
    pub user: User
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
            let session_id = sessions.select(id)
                .filter(cookie.eq(&secret))
                .first::<i32>(conn);
            if let Ok(s_id) = result {
                // self.id = Some(s_id as i32);
                // self.user = Some(user);
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
    pub fn from_key(sess: Option<String>) -> Option<Self> {
        if let Some(sessionkey) = sess {
            use db::schema::sessions::dsl as s;
            use db::schema::users::dsl as u;
            let result  = u::users
                        .inner_join(s::sessions)
                        .select((s::id, (u::id, u::username, u::email, u::bio, u::created_time, u::password))) // TODO figure out how to not select pw
                        .filter(s::cookie.eq(sessionkey))
                        .first::<(i32, User)>(&POOL.get().unwrap())
                        .ok();
            if let Some(r) = result {
                Some(Self {id: r.0, user: r.1})
            }
            else {
                None
            }
        }
        else {
            // so we don't have to query db when key isnt present
            None
        }

    }
}

pub fn create_session_filter(optional: bool) -> BoxedFilter<(Option<Session>,)> {
    if optional {
        cookie::optional("EXAUTH")
        .map(move |key: Option<String>| {Session::from_key(key)})
        .boxed()
    } else {
        cookie::cookie("EXAUTH")
        .and_then(|key: String| async move {
            let s = Session::from_key(Some(key));
            if s.is_none() {
                Err(warp::reject::reject())
            }
            else {
                Ok(Some(s.unwrap()))
            }
            })
        .boxed()
    }
}
