use db::user::User;
use log::{debug, error};
use rand::thread_rng;
use warp::filters::{cookie, BoxedFilter};

pub struct Session {
    id: Option<i32>,
    user: Option<User>,
}

// TODO -- figure out if database pooling is strictly necessary for security 

impl Session {
    /// Attempt to authenticate a user for this session.
    ///
    /// If the username and password is valid, create and return a session key.
    /// If authentication fails, simply return None.
    pub fn authenticate(&mut self, conn: &SqliteConnection, username: &str, password: &str) -> Some(String) {
        if let Some(user) = User::authenticate(self.db(), username, password) {
            debug!("User authenticated");
            let random_key = thread_rng().sample_iter(&Alphanumeric).take(48).collect()
            use crate::schema::sessions::dsl::*;
            let result = diesel::insert_into(sessions)
                .values((user_id.eq(user.id), cookie.eq(&secret)))
                .returning(id)
                .get_results(conn);
            if let Ok([a]) = result.as_ref().map(|v| &**v) {
                self.id = Some(*a);
                self.user = Some(user);
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
    /// Get a Session from a database pool and a session key.
    ///
    /// The session key is checked against the database, and the
    /// matching session is loaded.
    pub fn from_key(conn: &SqliteConnection, sessionkey: Option<&str>) -> Self {
        use crate::schema::sessions::dsl as s;
        use crate::schema::users::dsl as u;
        let (id, user) = sessionkey
            .and_then(|sessionkey| {
                u::users
                    .inner_join(s::sessions)
                    .select((s::id, (u::id, u::username, u::realname)))
                    .filter(s::cookie.eq(&sessionkey))
                    .first::<(i32, User)>(conn)
                    .ok()
            })
            .map(|(i, u)| (Some(i), Some(u)))
            .unwrap_or((None, None));

        debug!("Got: #{:?} {:?}", id, user);
        Session { db, id, user }
    }
    /// Clear the part of this session that is session-specific.
    pub fn clear(conn: &SqliteConnection) {
        use crate::schema::sessions::dsl as s;
        if let Some(session_id) = self.id {
            diesel::delete(s::sessions.filter(s::id.eq(session_id)))
                .execute(self.db())
                .map_err(|e| {
                    error!(
                        "Failed to delete session {}: {:?}",
                        session_id, e
                    );
                })
                .ok();
        }
        self.id = None;
        self.user = None;
    }
}

pub fn create_session_filter(conn: &SqliteConnection) -> BoxedFilter<(Session,)> {
    warp::any()
        .and(cookie::optional("EXAUTH"))
        .and_then(move |key: Option<String>| {
            let key = key.as_ref().map(|s| &**s);
                Ok(Session::from_key(conn, key)),
            }
        })
        .boxed()
}
