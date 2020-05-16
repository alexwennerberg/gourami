use diesel::sqlite::SqliteConnection;
use super::schema::server_mutuals;

#[derive(Queryable, PartialEq, Debug)]
pub struct ServerMutual {
    pub id: i32,
    pub actor_id: String,
    pub inbox_url: String,
    pub accepted: bool,
    pub followed_back: bool,
    pub outbox_url: Option<String>, // not implemented yet
}

#[derive(Insertable)]
#[table_name="server_mutuals"]
pub struct NewServerMutual {
    pub actor_id: String,
    pub inbox_url: String,
}
