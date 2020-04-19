use chrono;
use activitystreams::object::streams;
use diesel::sqlite::SqliteConnection;
use diesel::deserialize::{Queryable};
use super::schema::notes;
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

#[derive(Insertable, Clone)]
#[table_name = "notes"]
pub struct NoteInput {
  //pub id: i32, //unsigned?
  pub creator_id: i32,
  pub parent_id: Option<i32>,
  pub content: String, // can we make this a slice?
  pub published: String,
  // pub published: chrono::NaiveDateTime,
}
