use chrono;
use activitystreams::object::streams;
use diesel::sqlite::SqliteConnection;
use diesel::deserialize::{Queryable};
use super::schema::note;
use super::schema::note::dsl::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Statuses are note activitystream object

#[derive(Queryable, Clone, Deserialize, Serialize)]
pub struct Note {
  pub id: i32,
  pub creator_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  pub published: String,
}

impl Note {
    pub fn get_for_user(conn: &SqliteConnection, user_id: i32) -> Vec<Self> {
        let results = note
        .filter(creator_id.eq(user_id))
        .limit(5)
        .load::<Self>(conn)
        .expect("Error loading posts");
        results
    }
}
#[derive(Insertable, Clone)]
#[table_name = "note"]
pub struct NoteInput {
  //pub id: i32, //unsigned?
  pub creator_id: i32,
  pub parent_id: Option<i32>,
  pub content: String, // can we make this a slice?
  pub published: String,
  // pub published: chrono::NaiveDateTime,
}
