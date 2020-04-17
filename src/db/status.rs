use chrono;
use activitystreams::object::streams::Note;
use diesel::sqlite::SqliteConnection;
use diesel::deserialize::{Queryable};
use super::schema::status;

// Statuses are note activitystream object

#[derive(Queryable)]
pub struct Status {
  pub id: i32,
  pub creator_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  pub published: chrono::NaiveDateTime,
}

impl Status {
    fn get_for_user(&self, conn: &SqliteConnection, user_id: &str) {
    }
}
#[derive(Insertable, Clone)]
#[table_name = "status"]
pub struct StatusInput {
  pub id: i32, //unsigned?
  pub creator_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  // pub published: chrono::NaiveDateTime,
}
