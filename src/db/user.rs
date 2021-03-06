use super::schema::users;
use crate::ap::SERVER;
use bcrypt;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Default, Queryable, Deserialize)]
pub struct RegistrationKey {
    value: String,
}

impl RegistrationKey {
    pub fn is_valid(conn: &SqliteConnection, key: &str) -> bool {
        use crate::db::schema::registration_keys::dsl::*;
        let key: Option<String> = registration_keys
            .select(value)
            .filter(value.eq(key))
            .first(conn)
            .ok();
        match key {
            Some(_) => true,
            None => false,
        }
    }

    pub fn clear_key(conn: &SqliteConnection, key: &str) {
        use crate::db::schema::registration_keys::dsl::*;
        diesel::delete(registration_keys.filter(value.eq(key)))
            .execute(conn)
            .ok();
    }
}

// a hack
#[derive(QueryableByName)]
#[table_name = "users"]
pub struct UserNameId {
    pub username: String,
    pub id: i32,
}

#[derive(Debug, Clone, Default, Queryable, QueryableByName, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: Option<String>, // TODO option
    pub website: String,       // TODO option
    pub bio: String,
    pub created_time: String,
    pub password: Option<String>,
    pub admin: bool,
    pub show_email: bool,
    pub remote_user: bool,
    pub can_invite: bool,
}

impl User {
    pub fn get_url(&self) -> String {
        format!("{}/?user_id={}", SERVER.global_id, self.id)
        // remote url?
    }
    pub fn authenticate(conn: &SqliteConnection, user: &str, pass: &str) -> Option<Self> {
        use crate::db::schema::users::dsl::*;
        // TODO -- allow email login as well
        debug!("Authenticating user {}", user);
        let user = match users.filter(username.eq(user)).first::<Self>(conn) {
            Ok(user) => user,
            Err(e) => {
                error!("Failed to load hash for {:?}: {:?}", user, e);
                return None;
            }
        };

        let u_pass = match &user.password {
            Some(p) => p,
            None => return None,
        };

        match bcrypt::verify(&pass, &u_pass) {
            Ok(true) => Some(user),
            Ok(false) => None,
            Err(e) => {
                error!("Verify failed for {:?}: {:?}", user, e);
                None
            }
        }
    }
}

#[derive(Insertable, Deserialize)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str,
}

#[derive(Insertable, Deserialize)]
#[table_name = "users"]
pub struct NewRemoteUser {
    pub username: String,
    pub remote_user: bool,
}
