use diesel::sqlite::SqliteConnection;
use super::schema::server_mutuals;

#[derive(Queryable, PartialEq, Debug)]
pub struct ServerMutual {
    id: i32,
    actor_id: String,
    accepted: bool,
    followed_back: bool,
    inbox_url: String,
    outbox_url: String, // not implemented yet
}

#[derive(Insertable)]
#[table_name="server_mutuals"]
pub struct NewServerMutual {
    pub actor_id: String,
    pub inbox_url: String,
}
