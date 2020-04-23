use diesel::sqlite::SqliteConnection;
use diesel::deserialize::{Queryable};
use super::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use bcrypt;

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
            None => false
        }
    }

    pub fn clear_key(conn: &SqliteConnection, key: &str) {
        use crate::db::schema::registration_keys::dsl::*;
            diesel::delete(
                registration_keys
                .filter(value.eq(key)))
                .execute(conn).ok();
    }
}

#[derive(Debug, Clone, Default, Queryable, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub bio: String,
    pub created_time: String,
    pub password: String, // is this OK? hashed
}

// TODO -- default "anonymous" user

impl User {
    pub fn authenticate(
        conn: &SqliteConnection,
        user: &str,
        pass: &str,
    ) -> Option<Self> {
        use crate::db::schema::users::dsl::*;
        let user = match users
            .filter(username.eq(user))
            .first::<User>(conn)
        {
            Ok(user) => user,
            Err(e) => {
                error!("Failed to load hash for {:?}: {:?}", user, e);
                return None;
            }
        };

        match bcrypt::verify(&pass, &user.password) {
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
#[table_name="users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str,
}

// impl validate
fn validate_username() {
}

fn validate_password() {
}

fn validate_email() {
}
