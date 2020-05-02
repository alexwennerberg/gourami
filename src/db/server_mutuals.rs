use diesel::sqlite::SqliteConnection;

#[derive(Queryable, Debug)]
struct ServerMutuals {
    id: i32,
    inbox_url: String,
    outbox_url: String,
}
