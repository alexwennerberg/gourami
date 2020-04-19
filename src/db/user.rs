use activitystreams::object::streams;
use diesel::sqlite::SqliteConnection;
use diesel::deserialize::{Queryable};
use super::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use bcrypt;

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

// impl<'a> NewUser<'a> {
// fn validate_and_insert() -> Result<Ok(()), Err> {
// }
//
// }
